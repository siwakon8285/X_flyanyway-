use async_trait::async_trait;
use chrono::{NaiveDate, NaiveTime};
use serde_json::Value;
use sqlx::{FromRow, PgPool, Postgres, Transaction};
use uuid::Uuid;

use crate::application::flight::{
    AirportReference, FlightAuditEntry, FlightDetail, FlightListFilter, FlightPage,
    FlightReferenceData, FlightRepository, PublicCabinPrice, PublicFlight, PublicFlightFilter,
};
use crate::domain::flight::{
    FlightCommand, FlightManagementError, FlightRecord, FlightStatus, ManagedCabin,
    ValidatedFlightCommand,
};

#[derive(Clone, Debug)]
pub struct SqlxFlightRepository {
    pool: PgPool,
}

impl SqlxFlightRepository {
    pub fn new(pool: PgPool) -> Self {
        Self { pool }
    }

    pub fn pool(&self) -> &PgPool {
        &self.pool
    }

    pub async fn create(
        &self,
        actor: Uuid,
        command: FlightCommand,
    ) -> Result<FlightRecord, FlightManagementError> {
        let command = command
            .validate()
            .map_err(|_| FlightManagementError::Validation)?;
        let operating_date = command
            .operating_date
            .ok_or(FlightManagementError::Validation)?;
        let mut transaction = self.pool.begin().await.map_err(infrastructure)?;
        let zones = validate_schedule(&mut transaction, &command).await?;
        let public_id = format!(
            "{}-{}",
            command
                .flight_number
                .as_str()
                .replace(' ', "-")
                .to_lowercase(),
            operating_date.format("%Y%m%d")
        );
        let id = sqlx::query_scalar::<_, Uuid>(
            "INSERT INTO flight_services (
                public_id, flight_number, origin_code, destination_code, aircraft_code,
                origin_time_zone, departure_time, arrival_time, arrival_day_offset,
                duration_minutes, stops, status, operating_date
             ) VALUES ($1,$2,$3,$4,$5,$6,$7,$8,$9,
                EXTRACT(EPOCH FROM ((($10::date + $9 * INTERVAL '1 day') + $8::time) AT TIME ZONE $11)
                    - (($10::date + $7::time) AT TIME ZONE $6))::int / 60,
                'DIRECT','SCHEDULED',$10)
             RETURNING id",
        )
        .bind(public_id)
        .bind(command.flight_number.as_str())
        .bind(&command.origin_code)
        .bind(&command.destination_code)
        .bind(&command.aircraft_code)
        .bind(&zones.origin)
        .bind(command.departure_time)
        .bind(command.arrival_time)
        .bind(command.arrival_day_offset)
        .bind(operating_date)
        .bind(&zones.destination)
        .fetch_one(&mut *transaction)
        .await
        .map_err(map_insert)?;
        write_cabins_and_templates(&mut transaction, id, &command).await?;
        let record = load_record(&mut transaction, id).await?;
        write_audit(&mut transaction, actor, id, "FLIGHT_CREATED", None, &record).await?;
        transaction.commit().await.map_err(infrastructure)?;
        Ok(record)
    }

    pub async fn update(
        &self,
        actor: Uuid,
        id: Uuid,
        expected_version: i64,
        command: FlightCommand,
    ) -> Result<FlightRecord, FlightManagementError> {
        let command = command
            .validate()
            .map_err(|_| FlightManagementError::Validation)?;
        let mut transaction = self.pool.begin().await.map_err(infrastructure)?;
        lock_service(&mut transaction, id).await?;
        let before = load_record(&mut transaction, id).await?;
        if before.version != expected_version {
            return Err(FlightManagementError::Conflict);
        }
        if before.status != FlightStatus::Scheduled {
            return Err(FlightManagementError::InvalidStatus);
        }
        let structural = is_structural_change(&before, &command);
        if structural {
            let materialized: bool = sqlx::query_scalar(
                "SELECT EXISTS(SELECT 1 FROM flight_instances WHERE flight_service_id=$1)",
            )
            .bind(id)
            .fetch_one(&mut *transaction)
            .await
            .map_err(infrastructure)?;
            if materialized {
                return Err(FlightManagementError::StructuralConflict);
            }
        }
        if structural {
            let zones = validate_schedule(&mut transaction, &command).await?;
            let operating_date = command
                .operating_date
                .ok_or(FlightManagementError::Validation)?;
            sqlx::query(
            "UPDATE flight_services SET flight_number=$2, origin_code=$3, destination_code=$4,
                aircraft_code=$5, origin_time_zone=$6, operating_date=$7, departure_time=$8,
                arrival_time=$9, arrival_day_offset=$10,
                duration_minutes=EXTRACT(EPOCH FROM (((($7::date + $10 * INTERVAL '1 day') + $9::time) AT TIME ZONE $11)
                    - (($7::date + $8::time) AT TIME ZONE $6)))::int / 60,
                version=version+1, updated_at=NOW() WHERE id=$1",
        )
        .bind(id)
        .bind(command.flight_number.as_str())
        .bind(&command.origin_code)
        .bind(&command.destination_code)
        .bind(&command.aircraft_code)
        .bind(&zones.origin)
        .bind(operating_date)
        .bind(command.departure_time)
        .bind(command.arrival_time)
        .bind(command.arrival_day_offset)
        .bind(&zones.destination)
            .execute(&mut *transaction)
            .await
            .map_err(map_insert)?;
        } else {
            sqlx::query(
                "UPDATE flight_services SET version=version+1,updated_at=NOW() WHERE id=$1",
            )
            .bind(id)
            .execute(&mut *transaction)
            .await
            .map_err(infrastructure)?;
        }
        upsert_prices(&mut transaction, id, &command).await?;
        if structural {
            sqlx::query("DELETE FROM flight_service_seat_templates WHERE flight_service_id=$1")
                .bind(id)
                .execute(&mut *transaction)
                .await
                .map_err(infrastructure)?;
            insert_templates(&mut transaction, id, &command).await?;
        }
        let after = load_record(&mut transaction, id).await?;
        write_audit(
            &mut transaction,
            actor,
            id,
            "FLIGHT_EDITED",
            Some(&before),
            &after,
        )
        .await?;
        transaction.commit().await.map_err(infrastructure)?;
        Ok(after)
    }

    pub async fn cancel(
        &self,
        actor: Uuid,
        id: Uuid,
        expected_version: i64,
    ) -> Result<FlightRecord, FlightManagementError> {
        let mut transaction = self.pool.begin().await.map_err(infrastructure)?;
        lock_service(&mut transaction, id).await?;
        let before = load_record(&mut transaction, id).await?;
        if before.version != expected_version {
            return Err(FlightManagementError::Conflict);
        }
        if before.status != FlightStatus::Scheduled {
            return Err(FlightManagementError::InvalidStatus);
        }
        sqlx::query(
            "UPDATE flight_services SET status='CANCELLED', cancelled_at=NOW(),
                version=version+1, updated_at=NOW() WHERE id=$1",
        )
        .bind(id)
        .execute(&mut *transaction)
        .await
        .map_err(infrastructure)?;
        let after = load_record(&mut transaction, id).await?;
        write_audit(
            &mut transaction,
            actor,
            id,
            "FLIGHT_CANCELLED",
            Some(&before),
            &after,
        )
        .await?;
        transaction.commit().await.map_err(infrastructure)?;
        Ok(after)
    }
}

#[async_trait]
impl FlightRepository for SqlxFlightRepository {
    async fn list(&self, filter: FlightListFilter) -> Result<FlightPage, FlightManagementError> {
        let search = filter
            .search
            .map(|value| format!("%{}%", value.to_uppercase()));
        let status = filter.status.map(|value| match value {
            FlightStatus::Scheduled => "SCHEDULED",
            FlightStatus::Cancelled => "CANCELLED",
        });
        let total: i64 = sqlx::query_scalar(
            "SELECT COUNT(*) FROM flight_services service
             WHERE ($1::text IS NULL OR service.flight_number ILIKE $1)
               AND ($2::text IS NULL OR service.origin_code=$2)
               AND ($3::text IS NULL OR service.destination_code=$3)
               AND ($4::date IS NULL OR service.operating_date=$4 OR service.operating_date IS NULL)
               AND ($5::text IS NULL OR service.status=$5)",
        )
        .bind(&search)
        .bind(&filter.origin)
        .bind(&filter.destination)
        .bind(filter.date)
        .bind(status)
        .fetch_one(&self.pool)
        .await
        .map_err(infrastructure)?;
        let rows = sqlx::query_as::<_, FlightRow>(
            "SELECT service.id,service.public_id,service.flight_number,service.origin_code,
                service.destination_code,COALESCE(service.origin_time_zone,origin.time_zone) origin_time_zone,
                destination.time_zone destination_time_zone,service.operating_date,service.departure_time,
                service.arrival_time,service.arrival_day_offset,service.aircraft_code,service.status,
                business.base_fare_amount business_price,business.currency_code business_currency,
                first_class.base_fare_amount first_price,first_class.currency_code first_currency,
                COALESCE((SELECT COUNT(*) FROM flight_service_seat_templates template WHERE template.flight_service_id=service.id AND template.cabin='business'),0) business_capacity,
                COALESCE((SELECT COUNT(*) FROM flight_service_seat_templates template WHERE template.flight_service_id=service.id AND template.cabin='first'),0) first_capacity,
                service.version,service.updated_at
             FROM flight_services service
             LEFT JOIN airports origin ON origin.code=service.origin_code
             LEFT JOIN airports destination ON destination.code=service.destination_code
             LEFT JOIN flight_service_cabins business ON business.flight_service_id=service.id AND business.cabin='business'
             LEFT JOIN flight_service_cabins first_class ON first_class.flight_service_id=service.id AND first_class.cabin='first'
             WHERE ($1::text IS NULL OR service.flight_number ILIKE $1)
               AND ($2::text IS NULL OR service.origin_code=$2)
               AND ($3::text IS NULL OR service.destination_code=$3)
               AND ($4::date IS NULL OR service.operating_date=$4 OR service.operating_date IS NULL)
               AND ($5::text IS NULL OR service.status=$5)
             ORDER BY service.operating_date DESC NULLS LAST, service.flight_number
             LIMIT $6 OFFSET $7",
        )
        .bind(&search).bind(&filter.origin).bind(&filter.destination).bind(filter.date).bind(status)
        .bind(filter.limit).bind(filter.offset)
        .fetch_all(&self.pool).await.map_err(infrastructure)?;
        let items = rows
            .into_iter()
            .map(FlightRow::into_record)
            .collect::<Result<Vec<_>, _>>()?;
        Ok(FlightPage {
            items,
            total,
            limit: filter.limit,
            offset: filter.offset,
        })
    }

    async fn detail(&self, id: Uuid) -> Result<FlightDetail, FlightManagementError> {
        let mut transaction = self.pool.begin().await.map_err(infrastructure)?;
        let flight = load_record(&mut transaction, id).await?;
        let audit = sqlx::query_as::<_, AuditRow>(
            "SELECT audit.id,audit.actor_email,audit.action,audit.before_state,audit.after_state,audit.created_at
             FROM flight_management_audit audit
             WHERE audit.flight_service_id=$1 ORDER BY audit.created_at DESC LIMIT 50",
        ).bind(id).fetch_all(&mut *transaction).await.map_err(infrastructure)?
            .into_iter().map(AuditRow::into_entry).collect();
        transaction.commit().await.map_err(infrastructure)?;
        Ok(FlightDetail { flight, audit })
    }

    async fn reference_data(&self) -> Result<FlightReferenceData, FlightManagementError> {
        let airports = sqlx::query_as::<_, AirportRow>(
            "SELECT airport.code,airport.name,airport.city,airport.country_code,
                country.name country_name,airport.time_zone
             FROM airports airport
             JOIN supported_countries country ON country.code=airport.country_code
             WHERE airport.is_primary
             ORDER BY country.name,airport.city,airport.code",
        )
        .fetch_all(&self.pool)
        .await
        .map_err(infrastructure)?
        .into_iter()
        .map(AirportRow::into_reference)
        .collect();
        let aircraft = sqlx::query_scalar(
            "SELECT DISTINCT aircraft_code FROM aircraft_seat_templates ORDER BY aircraft_code",
        )
        .fetch_all(&self.pool)
        .await
        .map_err(infrastructure)?;
        Ok(FlightReferenceData { airports, aircraft })
    }

    async fn search_public(
        &self,
        filter: PublicFlightFilter,
    ) -> Result<Vec<PublicFlight>, FlightManagementError> {
        let rows = sqlx::query_as::<_, PublicFlightRow>(
            "SELECT service.public_id,service.flight_number,service.origin_code,service.destination_code,
                service.departure_time,service.arrival_time,service.arrival_day_offset,service.duration_minutes,
                service.stops,service.aircraft_code,
                jsonb_agg(jsonb_build_object('amountThb',cabin.base_fare_amount,'cabin',cabin.cabin) ORDER BY cabin.cabin)
                    FILTER (WHERE cabin.cabin IN ('business','first') AND cabin.base_fare_amount IS NOT NULL AND cabin.currency_code='THB') cabin_prices
             FROM flight_services service
             JOIN flight_service_cabins selected ON selected.flight_service_id=service.id AND selected.cabin=$4
                 AND selected.base_fare_amount IS NOT NULL AND selected.currency_code='THB'
             JOIN flight_service_cabins cabin ON cabin.flight_service_id=service.id
             WHERE service.origin_code=$1 AND service.destination_code=$2 AND service.status='SCHEDULED'
               AND (service.operating_date IS NULL OR service.operating_date=$3)
               AND service.departure_time IS NOT NULL AND service.arrival_time IS NOT NULL
               AND service.arrival_day_offset IS NOT NULL AND service.duration_minutes IS NOT NULL
             GROUP BY service.id ORDER BY service.departure_time,service.flight_number LIMIT 100",
        )
        .bind(&filter.origin).bind(&filter.destination).bind(filter.departure).bind(filter.cabin.as_str())
        .fetch_all(&self.pool).await.map_err(infrastructure)?;
        rows.into_iter()
            .enumerate()
            .map(|(index, row)| row.into_public(index + 1))
            .collect()
    }

    async fn public_detail(
        &self,
        public_id: &str,
        departure: NaiveDate,
        cabin: crate::domain::value_objects::CabinClass,
    ) -> Result<PublicFlight, FlightManagementError> {
        let row = sqlx::query_as::<_, PublicFlightRow>(
            "SELECT service.public_id,service.flight_number,service.origin_code,service.destination_code,
                service.departure_time,service.arrival_time,service.arrival_day_offset,service.duration_minutes,
                service.stops,service.aircraft_code,
                jsonb_agg(jsonb_build_object('amountThb',prices.base_fare_amount,'cabin',prices.cabin) ORDER BY prices.cabin)
                    FILTER (WHERE prices.cabin IN ('business','first') AND prices.base_fare_amount IS NOT NULL AND prices.currency_code='THB') cabin_prices
             FROM flight_services service
             JOIN flight_service_cabins selected ON selected.flight_service_id=service.id AND selected.cabin=$3
                 AND selected.base_fare_amount IS NOT NULL AND selected.currency_code='THB'
             JOIN flight_service_cabins prices ON prices.flight_service_id=service.id
             WHERE service.public_id=$1 AND service.status='SCHEDULED'
               AND (service.operating_date IS NULL OR service.operating_date=$2)
               AND service.departure_time IS NOT NULL AND service.arrival_time IS NOT NULL
               AND service.arrival_day_offset IS NOT NULL AND service.duration_minutes IS NOT NULL
             GROUP BY service.id",
        ).bind(public_id).bind(departure).bind(cabin.as_str())
            .fetch_optional(&self.pool).await.map_err(infrastructure)?
            .ok_or(FlightManagementError::NotFound)?;
        row.into_public(1)
    }

    async fn create(
        &self,
        actor: Uuid,
        command: FlightCommand,
    ) -> Result<FlightRecord, FlightManagementError> {
        SqlxFlightRepository::create(self, actor, command).await
    }
    async fn update(
        &self,
        actor: Uuid,
        id: Uuid,
        version: i64,
        command: FlightCommand,
    ) -> Result<FlightRecord, FlightManagementError> {
        SqlxFlightRepository::update(self, actor, id, version, command).await
    }
    async fn cancel(
        &self,
        actor: Uuid,
        id: Uuid,
        version: i64,
    ) -> Result<FlightRecord, FlightManagementError> {
        SqlxFlightRepository::cancel(self, actor, id, version).await
    }
}

struct Zones {
    origin: String,
    destination: String,
}

async fn validate_schedule(
    transaction: &mut Transaction<'_, Postgres>,
    command: &ValidatedFlightCommand,
) -> Result<Zones, FlightManagementError> {
    let aircraft_exists: bool = sqlx::query_scalar(
        "SELECT EXISTS(
            SELECT 1 FROM aircraft_seat_templates WHERE aircraft_code=$1
         )",
    )
    .bind(&command.aircraft_code)
    .fetch_one(&mut **transaction)
    .await
    .map_err(infrastructure)?;
    if !aircraft_exists {
        return Err(FlightManagementError::Validation);
    }
    let rows = sqlx::query_as::<_, AirportZone>(
        "SELECT code, time_zone FROM airports WHERE code=$1 OR code=$2 ORDER BY code",
    )
    .bind(&command.origin_code)
    .bind(&command.destination_code)
    .fetch_all(&mut **transaction)
    .await
    .map_err(infrastructure)?;
    let origin = rows
        .iter()
        .find(|row| row.code == command.origin_code)
        .map(|row| row.time_zone.clone());
    let destination = rows
        .iter()
        .find(|row| row.code == command.destination_code)
        .map(|row| row.time_zone.clone());
    let (Some(origin), Some(destination)) = (origin, destination) else {
        return Err(FlightManagementError::Validation);
    };
    let operating_date = command
        .operating_date
        .ok_or(FlightManagementError::Validation)?;
    let valid: bool = sqlx::query_scalar(
        "SELECT (($1::date + $2::time) AT TIME ZONE $3)
              < (((($1::date + $4 * INTERVAL '1 day') + $5::time)) AT TIME ZONE $6)",
    )
    .bind(operating_date)
    .bind(command.departure_time)
    .bind(&origin)
    .bind(command.arrival_day_offset)
    .bind(command.arrival_time)
    .bind(&destination)
    .fetch_one(&mut **transaction)
    .await
    .map_err(|_| FlightManagementError::Validation)?;
    if !valid {
        return Err(FlightManagementError::Validation);
    }
    Ok(Zones {
        origin,
        destination,
    })
}

async fn write_cabins_and_templates(
    transaction: &mut Transaction<'_, Postgres>,
    id: Uuid,
    command: &ValidatedFlightCommand,
) -> Result<(), FlightManagementError> {
    upsert_prices(transaction, id, command).await?;
    insert_templates(transaction, id, command).await
}

async fn upsert_prices(
    transaction: &mut Transaction<'_, Postgres>,
    id: Uuid,
    command: &ValidatedFlightCommand,
) -> Result<(), FlightManagementError> {
    for (cabin, amount) in [
        ("business", command.business_price_amount),
        ("first", command.first_price_amount),
    ] {
        sqlx::query(
            "INSERT INTO flight_service_cabins (flight_service_id,cabin,base_fare_amount,currency_code)
             VALUES ($1,$2,$3,$4) ON CONFLICT (flight_service_id,cabin) DO UPDATE
             SET base_fare_amount=EXCLUDED.base_fare_amount,currency_code=EXCLUDED.currency_code",
        )
        .bind(id).bind(cabin).bind(amount).bind(&command.currency_code)
        .execute(&mut **transaction).await.map_err(infrastructure)?;
    }
    Ok(())
}

async fn insert_templates(
    transaction: &mut Transaction<'_, Postgres>,
    id: Uuid,
    command: &ValidatedFlightCommand,
) -> Result<(), FlightManagementError> {
    let business = [
        ('A', "window"),
        ('D', "aisle"),
        ('G', "aisle"),
        ('K', "window"),
    ];
    let first = [('A', "window"), ('K', "window")];
    for (cabin, capacity, columns, start_row) in [
        ("business", command.business_capacity, &business[..], 3_u16),
        ("first", command.first_capacity, &first[..], 1_u16),
    ] {
        for index in 0..capacity {
            let row = index / columns.len() as u16 + start_row;
            let (column, position) = columns[index as usize % columns.len()];
            sqlx::query(
                "INSERT INTO flight_service_seat_templates
                 (flight_service_id,seat_number,row_number,column_code,cabin,position)
                 VALUES ($1,$2,$3,$4,$5,$6)",
            )
            .bind(id)
            .bind(format!("{row}{column}"))
            .bind(row as i16)
            .bind(column.to_string())
            .bind(cabin)
            .bind(position)
            .execute(&mut **transaction)
            .await
            .map_err(infrastructure)?;
        }
    }
    Ok(())
}

async fn lock_service(
    transaction: &mut Transaction<'_, Postgres>,
    id: Uuid,
) -> Result<(), FlightManagementError> {
    sqlx::query_scalar::<_, Uuid>("SELECT id FROM flight_services WHERE id=$1 FOR UPDATE")
        .bind(id)
        .fetch_optional(&mut **transaction)
        .await
        .map_err(infrastructure)?
        .ok_or(FlightManagementError::NotFound)?;
    Ok(())
}

async fn load_record(
    transaction: &mut Transaction<'_, Postgres>,
    id: Uuid,
) -> Result<FlightRecord, FlightManagementError> {
    let row = sqlx::query_as::<_, FlightRow>(
        "SELECT service.id,service.public_id,service.flight_number,service.origin_code,
            service.destination_code,COALESCE(service.origin_time_zone,origin.time_zone) origin_time_zone,
            destination.time_zone destination_time_zone,service.operating_date,service.departure_time,
            service.arrival_time,service.arrival_day_offset,service.aircraft_code,service.status,
            business.base_fare_amount business_price,business.currency_code business_currency,
            first_class.base_fare_amount first_price,first_class.currency_code first_currency,
            (SELECT COUNT(*) FROM flight_service_seat_templates template WHERE template.flight_service_id=service.id AND template.cabin='business') business_capacity,
            (SELECT COUNT(*) FROM flight_service_seat_templates template WHERE template.flight_service_id=service.id AND template.cabin='first') first_capacity,
            service.version,service.updated_at
         FROM flight_services service
         LEFT JOIN airports origin ON origin.code=service.origin_code
         LEFT JOIN airports destination ON destination.code=service.destination_code
         LEFT JOIN flight_service_cabins business ON business.flight_service_id=service.id AND business.cabin='business'
         LEFT JOIN flight_service_cabins first_class ON first_class.flight_service_id=service.id AND first_class.cabin='first'
         WHERE service.id=$1",
    )
    .bind(id).fetch_optional(&mut **transaction).await.map_err(infrastructure)?
    .ok_or(FlightManagementError::NotFound)?;
    row.into_record()
}

fn is_structural_change(before: &FlightRecord, command: &ValidatedFlightCommand) -> bool {
    before.flight_number != command.flight_number.as_str()
        || before.origin_code != command.origin_code
        || before.destination_code != command.destination_code
        || before.operating_date != command.operating_date
        || before.departure_time != Some(command.departure_time)
        || before.arrival_time != Some(command.arrival_time)
        || before.arrival_day_offset != Some(command.arrival_day_offset)
        || before.aircraft_code != command.aircraft_code
        || before.business.capacity != command.business_capacity
        || before.first.capacity != command.first_capacity
}

async fn write_audit(
    transaction: &mut Transaction<'_, Postgres>,
    actor: Uuid,
    id: Uuid,
    action: &str,
    before: Option<&FlightRecord>,
    after: &FlightRecord,
) -> Result<(), FlightManagementError> {
    let before: Option<Value> = before
        .map(serde_json::to_value)
        .transpose()
        .map_err(|_| FlightManagementError::Infrastructure)?;
    let after = serde_json::to_value(after).map_err(|_| FlightManagementError::Infrastructure)?;
    let result = sqlx::query("INSERT INTO flight_management_audit (actor_staff_user_id,actor_email,flight_service_id,action,before_state,after_state)
         SELECT $1,staff.email,$2,$3,$4,$5 FROM staff_users staff WHERE staff.id=$1")
        .bind(actor).bind(id).bind(action).bind(before).bind(after)
        .execute(&mut **transaction).await.map_err(infrastructure)?;
    if result.rows_affected() != 1 {
        return Err(FlightManagementError::Infrastructure);
    }
    Ok(())
}

fn infrastructure(_: sqlx::Error) -> FlightManagementError {
    FlightManagementError::Infrastructure
}

fn map_insert(error: sqlx::Error) -> FlightManagementError {
    if error
        .as_database_error()
        .and_then(|value| value.code())
        .is_some_and(|code| code == "23505")
    {
        FlightManagementError::Duplicate
    } else {
        FlightManagementError::Infrastructure
    }
}

#[derive(FromRow)]
struct AirportZone {
    code: String,
    time_zone: String,
}

#[derive(FromRow)]
struct FlightRow {
    id: Uuid,
    public_id: String,
    flight_number: String,
    origin_code: String,
    destination_code: String,
    origin_time_zone: Option<String>,
    destination_time_zone: Option<String>,
    operating_date: Option<NaiveDate>,
    departure_time: Option<NaiveTime>,
    arrival_time: Option<NaiveTime>,
    arrival_day_offset: Option<i16>,
    aircraft_code: String,
    status: String,
    business_price: Option<i64>,
    business_currency: Option<String>,
    first_price: Option<i64>,
    first_currency: Option<String>,
    business_capacity: i64,
    first_capacity: i64,
    version: i64,
    updated_at: chrono::DateTime<chrono::Utc>,
}

#[derive(FromRow)]
struct AuditRow {
    id: Uuid,
    actor_email: String,
    action: String,
    before_state: Option<Value>,
    after_state: Value,
    created_at: chrono::DateTime<chrono::Utc>,
}

impl AuditRow {
    fn into_entry(self) -> FlightAuditEntry {
        FlightAuditEntry {
            id: self.id,
            actor_email: self.actor_email,
            action: self.action,
            before_state: self.before_state,
            after_state: self.after_state,
            created_at: self.created_at,
        }
    }
}

#[derive(FromRow)]
struct AirportRow {
    code: String,
    name: String,
    city: String,
    country_code: String,
    country_name: String,
    time_zone: String,
}

impl AirportRow {
    fn into_reference(self) -> AirportReference {
        AirportReference {
            code: self.code,
            name: self.name,
            city: self.city,
            country_code: self.country_code,
            country_name: self.country_name,
            time_zone: self.time_zone,
        }
    }
}

#[derive(FromRow)]
struct PublicFlightRow {
    public_id: String,
    flight_number: String,
    origin_code: String,
    destination_code: String,
    departure_time: Option<NaiveTime>,
    arrival_time: Option<NaiveTime>,
    arrival_day_offset: Option<i16>,
    duration_minutes: Option<i16>,
    stops: Option<String>,
    aircraft_code: String,
    cabin_prices: Value,
}

impl PublicFlightRow {
    fn into_public(self, recommended_rank: usize) -> Result<PublicFlight, FlightManagementError> {
        let cabin_prices: Vec<PublicCabinPrice> = serde_json::from_value(self.cabin_prices)
            .map_err(|_| FlightManagementError::Infrastructure)?;
        Ok(PublicFlight {
            id: self.public_id,
            flight_number: self.flight_number,
            origin_code: self.origin_code,
            destination_code: self.destination_code,
            departure_time: self
                .departure_time
                .ok_or(FlightManagementError::Infrastructure)?
                .format("%H:%M")
                .to_string(),
            arrival_time: self
                .arrival_time
                .ok_or(FlightManagementError::Infrastructure)?
                .format("%H:%M")
                .to_string(),
            arrival_day_offset: u8::try_from(
                self.arrival_day_offset
                    .ok_or(FlightManagementError::Infrastructure)?,
            )
            .map_err(|_| FlightManagementError::Infrastructure)?,
            duration_minutes: u16::try_from(
                self.duration_minutes
                    .ok_or(FlightManagementError::Infrastructure)?,
            )
            .map_err(|_| FlightManagementError::Infrastructure)?,
            stops: match self.stops.as_deref() {
                Some("DIRECT") => "direct",
                Some("ONE_STOP") => "one-stop",
                _ => return Err(FlightManagementError::Infrastructure),
            }
            .to_owned(),
            aircraft: self.aircraft_code,
            status: "scheduled".to_owned(),
            recommended_rank,
            cabin_prices,
        })
    }
}

impl FlightRow {
    fn into_record(self) -> Result<FlightRecord, FlightManagementError> {
        Ok(FlightRecord {
            id: self.id,
            public_id: self.public_id,
            flight_number: self.flight_number,
            origin_code: self.origin_code,
            destination_code: self.destination_code,
            origin_time_zone: self
                .origin_time_zone
                .ok_or(FlightManagementError::Infrastructure)?,
            destination_time_zone: self
                .destination_time_zone
                .ok_or(FlightManagementError::Infrastructure)?,
            operating_date: self.operating_date,
            departure_time: self.departure_time,
            arrival_time: self.arrival_time,
            arrival_day_offset: self.arrival_day_offset,
            aircraft_code: self.aircraft_code,
            status: match self.status.as_str() {
                "SCHEDULED" => FlightStatus::Scheduled,
                "CANCELLED" => FlightStatus::Cancelled,
                _ => return Err(FlightManagementError::Infrastructure),
            },
            business: ManagedCabin {
                available: self.business_price.is_some(),
                price_amount: self.business_price,
                currency_code: self.business_currency,
                capacity: u16::try_from(self.business_capacity)
                    .map_err(|_| FlightManagementError::Infrastructure)?,
            },
            first: ManagedCabin {
                available: self.first_price.is_some(),
                price_amount: self.first_price,
                currency_code: self.first_currency,
                capacity: u16::try_from(self.first_capacity)
                    .map_err(|_| FlightManagementError::Infrastructure)?,
            },
            version: self.version,
            updated_at: self.updated_at,
        })
    }
}
