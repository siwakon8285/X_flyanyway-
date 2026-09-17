use chrono::{DateTime, NaiveDate, Utc};
use sqlx::{Executor, Postgres};

pub async fn origin_local_today<'e, E>(
    executor: E,
    authoritative_now: DateTime<Utc>,
    origin_time_zone: &str,
) -> Result<NaiveDate, sqlx::Error>
where
    E: Executor<'e, Database = Postgres>,
{
    sqlx::query_scalar("SELECT ($1::timestamptz AT TIME ZONE $2::text)::date")
        .bind(authoritative_now)
        .bind(origin_time_zone)
        .fetch_one(executor)
        .await
}
