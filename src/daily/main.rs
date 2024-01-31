use anyhow::Result;
use chrono::{DateTime, Utc};
use chrono_tz::Tz;
use dotenvy_macro::dotenv;
use poise::serenity_prelude::{self as serenity, CreateMessage, UserId};
use sqlx::PgPool;
use std::collections::HashMap;

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

    let user_bet_map: HashMap<u64, DateTime<Tz>> = query
        .iter()
        .map(|query| {
            (
                query.user_id as u64,
                query.bet_time.with_timezone(
                    &query
                        .timezone
                        .parse::<Tz>()
                        .expect("Datbase timezone to be valid"),
                ),
            )
        })
        .collect();

    let winner = match user_bet_map.iter().min_by_key(|(_, bet_time)| {
        let midnight_local = bet_time.date().and_hms_nano_opt(0, 0, 0, 0).unwrap();
        let duration = bet_time.signed_duration_since(midnight_local);

        duration.num_seconds()
    }) {
        Some(winner) => winner,
        None => return Ok(()),
    };

    println!("{query:?}");
    println!("{user_bet_map:?}");
    println!("{winner:?}");

    let token = dotenv!("DISCORD_TOKEN");
    let intents =
        serenity::GatewayIntents::non_privileged() | serenity::GatewayIntents::MESSAGE_CONTENT;

    let client = serenity::ClientBuilder::new(token, intents).await?;

    let user_id = UserId::new(*winner.0);
    let user = user_id.to_user(&client.http).await?;

    user.dm(client.http, CreateMessage::new().content("You won!"))
        .await?;

    sqlx::query!("DELETE FROM users").execute(&pool).await?;

    Ok(())
}
