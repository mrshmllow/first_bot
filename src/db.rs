use anyhow::Result;
use chrono_tz::Tz;
use sqlx::{Pool, Postgres};

pub async fn get_timezone(user_id: i64, pool: &Pool<Postgres>) -> Result<Option<Tz>> {
    #[derive(Debug)]
    struct Query {
        timezone: String,
    }

    let query = sqlx::query_as!(Query, "SELECT timezone FROM users WHERE id = $1", user_id)
        .fetch_optional(pool)
        .await?;

    return Ok(query.map(|query| query.timezone.parse::<Tz>().unwrap()));
}
