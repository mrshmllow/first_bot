#![warn(clippy::str_to_string)]

mod commands;
mod db;

use anyhow::Result;
use dotenvy_macro::dotenv;
use log::{error, info};
use poise::serenity_prelude::{self as serenity, FullEvent};
use sqlx::{PgPool, Pool, Postgres};
use std::env::var;

type Error = Box<dyn std::error::Error + Send + Sync>;
type Context<'a> = poise::Context<'a, Data, Error>;

// Me in your software :3

pub struct Data {
    pool: Pool<Postgres>,
}

async fn on_error(error: poise::FrameworkError<'_, Data, Error>) {
    match error {
        poise::FrameworkError::Setup { error, .. } => panic!("Failed to start bot: {:?}", error),
        poise::FrameworkError::Command { error, ctx, .. } => {
            error!("Error in command `{}`: {:?}", ctx.command().name, error,);
        }
        error => {
            if let Err(e) = poise::builtins::on_error(error).await {
                error!("Error while handling error: {}", e)
            }
        }
    }
}

#[tokio::main]
async fn main() -> Result<()> {
    env_logger::init();

    let pool = PgPool::connect(dotenv!("DATABASE_URL")).await?;

    sqlx::migrate!().run(&pool).await?;

    let options = poise::FrameworkOptions {
        commands: vec![
            commands::change_timezone(),
            commands::timezone(),
            commands::help(),
        ],
        prefix_options: poise::PrefixFrameworkOptions {
            prefix: Some("!".into()),
            ..Default::default()
        },
        on_error: |error| Box::pin(on_error(error)),
        skip_checks_for_owners: false,
        event_handler: |_ctx, event, _framework, _data| {
            Box::pin(async move {
                let message = match event {
                    FullEvent::Message { new_message }
                        if new_message.content.to_lowercase().starts_with("first") =>
                    {
                        new_message
                    }
                    _ => return Ok(()),
                };

                commands::first(message, _data).await?;

                Ok(())
            })
        },
        ..Default::default()
    };

    let framework = poise::Framework::builder()
        .setup(move |ctx, _ready, framework| {
            Box::pin(async move {
                info!("Logged in as {}", _ready.user.name);
                poise::builtins::register_globally(ctx, &framework.options().commands).await?;
                Ok(Data { pool: pool.into() })
            })
        })
        .options(options)
        .build();

    let token = dotenv!("DISCORD_TOKEN");
    let intents =
        serenity::GatewayIntents::non_privileged() | serenity::GatewayIntents::MESSAGE_CONTENT;

    let client = serenity::ClientBuilder::new(token, intents)
        .framework(framework)
        .await;

    client?.start().await?;

    Ok(())
}
