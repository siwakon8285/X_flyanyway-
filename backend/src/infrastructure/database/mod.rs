use std::time::Duration;

use async_trait::async_trait;
use chrono::{DateTime, Utc};
use sqlx::{FromRow, PgPool, Postgres, Transaction};
use thiserror::Error;
use uuid::Uuid;

use crate::domain::{
    entities::{
        CreateSeatHold, FlightSelection, SeatAvailability, SeatHold, SeatInventoryItem, SeatMap,
    },
    repositories::{SeatHoldRepository, SeatHoldRepositoryError},
    value_objects::{CabinClass, PassengerCounts, SeatNumber},
};

mod passengers;

#[derive(Debug, Error)]
pub enum DatabaseInitError {
    #[error("database migration failed")]
    Migration(#[from] sqlx::migrate::MigrateError),
    #[error("reference inventory seed failed")]
    Seed(#[from] sqlx::Error),
}

pub async fn prepare_database(pool: &PgPool) -> Result<(), DatabaseInitError> {
    sqlx::migrate!("./migrations").run(pool).await?;
    sqlx::raw_sql(include_str!("../../../seeds/demo_flight_inventory.sql"))
        .execute(pool)
        .await?;
    Ok(())
}

#[derive(Clone, Debug)]
pub struct SqlxSeatHoldRepository {
    pool: PgPool,
}

impl SqlxSeatHoldRepository {
    pub fn new(pool: PgPool) -> Self {
        Self { pool }
    }

    pub fn pool(&self) -> &PgPool {
        &self.pool
    }

    async fn ensure_inventory(
        transaction: &mut Transaction<'_, Postgres>,
        selection: &FlightSelection,
    ) -> Result<Uuid, SeatHoldRepositoryError> {
        let service = sqlx::query_as::<_, ServiceRow>(
            "SELECT id, aircraft_code FROM flight_services WHERE public_id = $1",
        )
        .bind(&selection.flight_id)
        .fetch_optional(&mut **transaction)
        .await
        .map_err(SeatHoldRepositoryError::Infrastructure)?
        .ok_or(SeatHoldRepositoryError::FlightNotFound)?;

        let cabin_is_sold: bool = sqlx::query_scalar(
            "SELECT EXISTS(
                SELECT 1 FROM flight_service_cabins
                WHERE flight_service_id = $1 AND cabin = $2
            )",
        )
        .bind(service.id)
        .bind(selection.cabin.as_str())
        .fetch_one(&mut **transaction)
        .await
        .map_err(SeatHoldRepositoryError::Infrastructure)?;
        if !cabin_is_sold {
            return Err(SeatHoldRepositoryError::CabinUnavailable);
        }

        let instance_id: Uuid = sqlx::query_scalar(
            "INSERT INTO flight_instances (flight_service_id, departure_date)
             VALUES ($1, $2)
             ON CONFLICT (flight_service_id, departure_date)
             DO UPDATE SET updated_at = flight_instances.updated_at
             RETURNING id",
        )
        .bind(service.id)
        .bind(selection.departure_date)
        .fetch_one(&mut **transaction)
        .await
        .map_err(SeatHoldRepositoryError::Infrastructure)?;

        sqlx::query(
            "INSERT INTO flight_seats (
                flight_instance_id, seat_number, row_number, column_code, cabin, position, sellable
             )
             SELECT $1, seat_number, row_number, column_code, cabin, position, sellable
             FROM aircraft_seat_templates
             WHERE aircraft_code = $2
             ON CONFLICT (flight_instance_id, cabin, seat_number) DO NOTHING",
        )
        .bind(instance_id)
        .bind(service.aircraft_code)
        .execute(&mut **transaction)
        .await
        .map_err(SeatHoldRepositoryError::Infrastructure)?;

        Ok(instance_id)
    }

    async fn server_time(
        transaction: &mut Transaction<'_, Postgres>,
    ) -> Result<DateTime<Utc>, SeatHoldRepositoryError> {
        sqlx::query_scalar("SELECT NOW()")
            .fetch_one(&mut **transaction)
            .await
            .map_err(SeatHoldRepositoryError::Infrastructure)
    }

    fn normalized_seats(
        requested: Vec<SeatNumber>,
        required_count: usize,
    ) -> Result<Vec<SeatNumber>, SeatHoldRepositoryError> {
        if requested.is_empty() || requested.len() > required_count {
            return Err(SeatHoldRepositoryError::SeatCountMismatch);
        }
        let requested_len = requested.len();
        let mut normalized = requested;
        normalized.sort();
        normalized.dedup();
        if normalized.len() != requested_len {
            return Err(SeatHoldRepositoryError::SeatCountMismatch);
        }
        Ok(normalized)
    }

    async fn locked_inventory_rows(
        transaction: &mut Transaction<'_, Postgres>,
        instance_id: Uuid,
        cabin: CabinClass,
        seats: &[SeatNumber],
        existing_hold: Option<Uuid>,
    ) -> Result<Vec<LockedSeatRow>, SeatHoldRepositoryError> {
        let seat_values: Vec<&str> = seats.iter().map(SeatNumber::as_str).collect();
        sqlx::query_as::<_, LockedSeatRow>(
            "SELECT
                seat.id,
                seat.seat_number,
                seat.cabin,
                seat.sellable,
                seat.booking_status,
                seat.hold_id,
                owner.expires_at AS owner_expires_at,
                owner.released_at AS owner_released_at,
                owner.consumed_at AS owner_consumed_at
             FROM flight_seats AS seat
             LEFT JOIN seat_holds AS owner ON owner.id = seat.hold_id
             WHERE seat.flight_instance_id = $1
               AND seat.cabin = $2
               AND (seat.seat_number = ANY($3) OR seat.hold_id = $4)
             ORDER BY seat.seat_number
             FOR UPDATE OF seat",
        )
        .bind(instance_id)
        .bind(cabin.as_str())
        .bind(&seat_values)
        .bind(existing_hold)
        .fetch_all(&mut **transaction)
        .await
        .map_err(SeatHoldRepositoryError::Infrastructure)
    }

    fn validate_requested_inventory(
        rows: &[LockedSeatRow],
        requested: &[SeatNumber],
        current_hold: Option<Uuid>,
        server_time: DateTime<Utc>,
    ) -> Result<Vec<Uuid>, SeatHoldRepositoryError> {
        let mut missing = Vec::new();
        let mut conflicts = Vec::new();
        let mut ids = Vec::with_capacity(requested.len());

        for requested_seat in requested {
            let Some(row) = rows
                .iter()
                .find(|row| row.seat_number == requested_seat.as_str())
            else {
                missing.push(requested_seat.to_string());
                continue;
            };

            let active_other_hold = row.hold_id.is_some_and(|hold_id| {
                Some(hold_id) != current_hold
                    && row
                        .owner_expires_at
                        .is_some_and(|expiry| expiry > server_time)
                    && row.owner_released_at.is_none()
                    && row.owner_consumed_at.is_none()
            });
            if !row.sellable || row.booking_status == "BOOKED" || active_other_hold {
                conflicts.push(requested_seat.to_string());
            } else {
                ids.push(row.id);
            }
        }

        if !missing.is_empty() {
            return Err(SeatHoldRepositoryError::SeatNotFound(missing));
        }
        if !conflicts.is_empty() {
            return Err(SeatHoldRepositoryError::SeatConflict(conflicts));
        }
        Ok(ids)
    }

    async fn locked_hold(
        transaction: &mut Transaction<'_, Postgres>,
        hold_id: Uuid,
        token_hash: [u8; 32],
    ) -> Result<HoldRow, SeatHoldRepositoryError> {
        let hold = sqlx::query_as::<_, HoldRow>(
            "SELECT
                hold.id,
                service.public_id AS flight_id,
                instance.departure_date,
                hold.flight_instance_id,
                hold.cabin,
                hold.adults,
                hold.children,
                hold.infants,
                hold.access_token_hash,
                hold.expires_at,
                hold.released_at,
                hold.consumed_at
             FROM seat_holds AS hold
             JOIN flight_instances AS instance ON instance.id = hold.flight_instance_id
             JOIN flight_services AS service ON service.id = instance.flight_service_id
             WHERE hold.id = $1
             FOR UPDATE OF hold",
        )
        .bind(hold_id)
        .fetch_optional(&mut **transaction)
        .await
        .map_err(SeatHoldRepositoryError::Infrastructure)?
        .ok_or(SeatHoldRepositoryError::HoldNotFound)?;

        if hold.access_token_hash.as_slice() != token_hash {
            return Err(SeatHoldRepositoryError::Unauthorized);
        }
        let server_time = Self::server_time(transaction).await?;
        if hold.consumed_at.is_some() {
            return Err(SeatHoldRepositoryError::HoldConsumed);
        }
        if hold.released_at.is_some() {
            return Err(SeatHoldRepositoryError::HoldReleased);
        }
        if hold.expires_at <= server_time {
            return Err(SeatHoldRepositoryError::HoldExpired);
        }
        Ok(hold)
    }

    async fn hold_entity(
        transaction: &mut Transaction<'_, Postgres>,
        hold: HoldRow,
    ) -> Result<SeatHold, SeatHoldRepositoryError> {
        let seat_values: Vec<String> = sqlx::query_scalar(
            "SELECT seat_number FROM flight_seats WHERE hold_id = $1 ORDER BY seat_number",
        )
        .bind(hold.id)
        .fetch_all(&mut **transaction)
        .await
        .map_err(SeatHoldRepositoryError::Infrastructure)?;
        let seats = seat_values
            .iter()
            .map(|seat| SeatNumber::parse(seat).expect("database seat constraint is valid"))
            .collect();
        let server_time = Self::server_time(transaction).await?;

        Ok(SeatHold {
            id: hold.id,
            flight_id: hold.flight_id,
            departure_date: hold.departure_date,
            cabin: hold
                .cabin
                .parse()
                .expect("database cabin constraint is valid"),
            passengers: PassengerCounts::new(
                hold.adults as u8,
                hold.children as u8,
                hold.infants as u8,
            )
            .expect("database passenger constraints are valid"),
            seats,
            expires_at: hold.expires_at,
            server_time,
        })
    }
}

#[async_trait]
impl SeatHoldRepository for SqlxSeatHoldRepository {
    async fn seat_map(
        &self,
        selection: &FlightSelection,
        owner: Option<(Uuid, [u8; 32])>,
    ) -> Result<SeatMap, SeatHoldRepositoryError> {
        let mut transaction = self
            .pool
            .begin()
            .await
            .map_err(SeatHoldRepositoryError::Infrastructure)?;
        let instance_id = Self::ensure_inventory(&mut transaction, selection).await?;
        let server_time = Self::server_time(&mut transaction).await?;

        if let Some((hold_id, token_hash)) = owner {
            Self::locked_hold(&mut transaction, hold_id, token_hash).await?;
        }

        let rows = sqlx::query_as::<_, InventoryRow>(
            "SELECT
                seat.seat_number,
                seat.row_number,
                seat.column_code,
                seat.position,
                seat.sellable,
                seat.booking_status,
                seat.hold_id,
                owner.expires_at AS owner_expires_at,
                owner.released_at AS owner_released_at,
                owner.consumed_at AS owner_consumed_at
             FROM flight_seats AS seat
             LEFT JOIN seat_holds AS owner ON owner.id = seat.hold_id
             WHERE seat.flight_instance_id = $1 AND seat.cabin = $2
             ORDER BY seat.row_number, seat.column_code",
        )
        .bind(instance_id)
        .bind(selection.cabin.as_str())
        .fetch_all(&mut *transaction)
        .await
        .map_err(SeatHoldRepositoryError::Infrastructure)?;
        let owner_id = owner.map(|(hold_id, _)| hold_id);

        let seats = rows
            .into_iter()
            .map(|row| {
                let active_hold = row
                    .owner_expires_at
                    .is_some_and(|expiry| expiry > server_time)
                    && row.owner_released_at.is_none()
                    && row.owner_consumed_at.is_none();
                let status = if !row.sellable || row.booking_status == "BOOKED" {
                    SeatAvailability::Booked
                } else if active_hold && row.hold_id == owner_id {
                    SeatAvailability::HeldByMe
                } else if active_hold {
                    SeatAvailability::Unavailable
                } else {
                    SeatAvailability::Available
                };
                SeatInventoryItem {
                    seat_number: SeatNumber::parse(&row.seat_number)
                        .expect("database seat constraint is valid"),
                    row_number: row.row_number,
                    column_code: row.column_code,
                    position: row.position,
                    status,
                }
            })
            .collect();

        transaction
            .commit()
            .await
            .map_err(SeatHoldRepositoryError::Infrastructure)?;
        Ok(SeatMap {
            flight_id: selection.flight_id.clone(),
            departure_date: selection.departure_date,
            cabin: selection.cabin,
            seats,
            server_time,
        })
    }

    async fn create_hold(
        &self,
        command: CreateSeatHold,
        ttl: Duration,
    ) -> Result<SeatHold, SeatHoldRepositoryError> {
        let requested = Self::normalized_seats(command.seats, command.passengers.required_seats())?;
        let mut transaction = self
            .pool
            .begin()
            .await
            .map_err(SeatHoldRepositoryError::Infrastructure)?;
        let instance_id = Self::ensure_inventory(&mut transaction, &command.selection).await?;
        let server_time = Self::server_time(&mut transaction).await?;
        let expires_at = server_time
            + chrono::Duration::from_std(ttl).unwrap_or_else(|_| chrono::Duration::minutes(10));
        let hold_id = Uuid::new_v4();

        sqlx::query(
            "INSERT INTO seat_holds (
                id, flight_instance_id, cabin, adults, children, infants,
                access_token_hash, expires_at
             ) VALUES ($1, $2, $3, $4, $5, $6, $7, $8)",
        )
        .bind(hold_id)
        .bind(instance_id)
        .bind(command.selection.cabin.as_str())
        .bind(i16::from(command.passengers.adults()))
        .bind(i16::from(command.passengers.children()))
        .bind(i16::from(command.passengers.infants()))
        .bind(command.token_hash.as_slice())
        .bind(expires_at)
        .execute(&mut *transaction)
        .await
        .map_err(SeatHoldRepositoryError::Infrastructure)?;

        let rows = Self::locked_inventory_rows(
            &mut transaction,
            instance_id,
            command.selection.cabin,
            &requested,
            None,
        )
        .await?;
        let seat_ids = Self::validate_requested_inventory(&rows, &requested, None, server_time)?;
        sqlx::query("UPDATE flight_seats SET hold_id = $1, updated_at = NOW() WHERE id = ANY($2)")
            .bind(hold_id)
            .bind(&seat_ids)
            .execute(&mut *transaction)
            .await
            .map_err(SeatHoldRepositoryError::Infrastructure)?;

        let entity = SeatHold {
            id: hold_id,
            flight_id: command.selection.flight_id,
            departure_date: command.selection.departure_date,
            cabin: command.selection.cabin,
            passengers: command.passengers,
            seats: requested,
            expires_at,
            server_time,
        };
        transaction
            .commit()
            .await
            .map_err(SeatHoldRepositoryError::Infrastructure)?;
        Ok(entity)
    }

    async fn replace_seats(
        &self,
        hold_id: Uuid,
        token_hash: [u8; 32],
        seats: Vec<SeatNumber>,
    ) -> Result<SeatHold, SeatHoldRepositoryError> {
        let mut transaction = self
            .pool
            .begin()
            .await
            .map_err(SeatHoldRepositoryError::Infrastructure)?;
        let hold = Self::locked_hold(&mut transaction, hold_id, token_hash).await?;
        let passengers =
            PassengerCounts::new(hold.adults as u8, hold.children as u8, hold.infants as u8)
                .expect("database passenger constraints are valid");
        let requested = Self::normalized_seats(seats, passengers.required_seats())?;
        let cabin: CabinClass = hold
            .cabin
            .parse()
            .expect("database cabin constraint is valid");
        let server_time = Self::server_time(&mut transaction).await?;
        let rows = Self::locked_inventory_rows(
            &mut transaction,
            hold.flight_instance_id,
            cabin,
            &requested,
            Some(hold_id),
        )
        .await?;
        let seat_ids =
            Self::validate_requested_inventory(&rows, &requested, Some(hold_id), server_time)?;

        sqlx::query(
            "UPDATE flight_seats SET hold_id = NULL, updated_at = NOW() WHERE hold_id = $1",
        )
        .bind(hold_id)
        .execute(&mut *transaction)
        .await
        .map_err(SeatHoldRepositoryError::Infrastructure)?;
        sqlx::query("UPDATE flight_seats SET hold_id = $1, updated_at = NOW() WHERE id = ANY($2)")
            .bind(hold_id)
            .bind(&seat_ids)
            .execute(&mut *transaction)
            .await
            .map_err(SeatHoldRepositoryError::Infrastructure)?;
        sqlx::query("UPDATE seat_holds SET updated_at = NOW() WHERE id = $1")
            .bind(hold_id)
            .execute(&mut *transaction)
            .await
            .map_err(SeatHoldRepositoryError::Infrastructure)?;

        let entity = Self::hold_entity(&mut transaction, hold).await?;
        transaction
            .commit()
            .await
            .map_err(SeatHoldRepositoryError::Infrastructure)?;
        Ok(entity)
    }

    async fn get_hold(
        &self,
        hold_id: Uuid,
        token_hash: [u8; 32],
    ) -> Result<SeatHold, SeatHoldRepositoryError> {
        let mut transaction = self
            .pool
            .begin()
            .await
            .map_err(SeatHoldRepositoryError::Infrastructure)?;
        let hold = Self::locked_hold(&mut transaction, hold_id, token_hash).await?;
        let entity = Self::hold_entity(&mut transaction, hold).await?;
        transaction
            .commit()
            .await
            .map_err(SeatHoldRepositoryError::Infrastructure)?;
        Ok(entity)
    }

    async fn release_hold(
        &self,
        hold_id: Uuid,
        token_hash: [u8; 32],
    ) -> Result<(), SeatHoldRepositoryError> {
        let mut transaction = self
            .pool
            .begin()
            .await
            .map_err(SeatHoldRepositoryError::Infrastructure)?;
        Self::locked_hold(&mut transaction, hold_id, token_hash).await?;
        sqlx::query(
            "UPDATE flight_seats SET hold_id = NULL, updated_at = NOW() WHERE hold_id = $1",
        )
        .bind(hold_id)
        .execute(&mut *transaction)
        .await
        .map_err(SeatHoldRepositoryError::Infrastructure)?;
        sqlx::query("UPDATE seat_holds SET released_at = NOW(), updated_at = NOW() WHERE id = $1")
            .bind(hold_id)
            .execute(&mut *transaction)
            .await
            .map_err(SeatHoldRepositoryError::Infrastructure)?;
        transaction
            .commit()
            .await
            .map_err(SeatHoldRepositoryError::Infrastructure)
    }
}

#[derive(FromRow)]
struct ServiceRow {
    id: Uuid,
    aircraft_code: String,
}

#[derive(FromRow)]
struct LockedSeatRow {
    id: Uuid,
    seat_number: String,
    #[allow(dead_code)]
    cabin: String,
    sellable: bool,
    booking_status: String,
    hold_id: Option<Uuid>,
    owner_expires_at: Option<DateTime<Utc>>,
    owner_released_at: Option<DateTime<Utc>>,
    owner_consumed_at: Option<DateTime<Utc>>,
}

#[derive(FromRow)]
struct InventoryRow {
    seat_number: String,
    row_number: i16,
    column_code: String,
    position: String,
    sellable: bool,
    booking_status: String,
    hold_id: Option<Uuid>,
    owner_expires_at: Option<DateTime<Utc>>,
    owner_released_at: Option<DateTime<Utc>>,
    owner_consumed_at: Option<DateTime<Utc>>,
}

#[derive(FromRow)]
struct HoldRow {
    id: Uuid,
    flight_id: String,
    departure_date: chrono::NaiveDate,
    flight_instance_id: Uuid,
    cabin: String,
    adults: i16,
    children: i16,
    infants: i16,
    access_token_hash: Vec<u8>,
    expires_at: DateTime<Utc>,
    released_at: Option<DateTime<Utc>>,
    consumed_at: Option<DateTime<Utc>>,
}
