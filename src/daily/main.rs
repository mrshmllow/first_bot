use anyhow::Result;
use chrono::{DateTime, Utc};
use chrono_tz::Tz;
use dotenvy_macro::dotenv;
use log::{error, info};
use sqlx::{PgPool, Pool, Postgres};
use std::{collections::HashMap, env::var};

#[tokio::main]
async fn main() -> Result<()> {
    env_logger::init();

    let pool = PgPool::connect(dotenv!("DATABASE_URL")).await?;

    #[derive(Debug)]
    struct Query {
        user_id: i64,
        bet_time: chrono::DateTime<Utc>,
        timezone: String,
    }

    let query = sqlx::query_as!(
        Query,
        "SELECT bets.bet_time, bets.user_id, users.timezone FROM bets JOIN users ON bets.user_id = users.id"
    )
    .fetch_all(&pool)
    .await?;

    let user_bet_map: HashMap<i64, DateTime<Tz>> = query
        .iter()
        .map(|query| {
            (
                query.user_id,
                query.bet_time.with_timezone(
                    &query
                        .timezone
                        .parse::<Tz>()
                        .expect("Datbase timezone to be valid"),
                ),
            )
        })
        .collect();

    let winner = user_bet_map.iter().min_by_key(|(id, bet_time)| {
        let midnight_local = bet_time.date().and_hms_nano_opt(0, 0, 0, 0).unwrap();

        let duration = bet_time.signed_duration_since(midnight_local);

        println!("{}", duration.num_hours());

        duration.num_seconds()
    });

    println!("{query:?}");
    println!("{user_bet_map:?}");
    println!("{winner:?}");

    Ok(())
}
