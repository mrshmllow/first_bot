#![warn(clippy::str_to_string)]

mod commands;
mod db;

use anyhow::Context as _;
use anyhow::Result;
use log::{error, info};
use poise::serenity_prelude::{self as serenity, FullEvent};
use shuttle_runtime::CustomError;
use shuttle_secrets::SecretStore;
use shuttle_serenity::ShuttleSerenity;
use sqlx::{PgPool, Pool, Postgres};

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

#[shuttle_runtime::main]
async fn main(
    #[shuttle_secrets::Secrets] secret_store: SecretStore,
    #[shuttle_shared_db::Postgres(local_uri = "postgresql://127.0.0.1:5432/first_bot")]
    pool: sqlx::PgPool,
) -> ShuttleSerenity {
    env_logger::init();

    sqlx::migrate!()
        .run(&pool)
        .await
        .map_err(CustomError::new)?;

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

    let token = secret_store
        .get("DISCORD_TOKEN")
        .context("DISCORD_TOKEN was not found")?;

    let intents =
        serenity::GatewayIntents::non_privileged() | serenity::GatewayIntents::MESSAGE_CONTENT;

    let client = serenity::ClientBuilder::new(token, intents)
        .framework(framework)
        .await
        .map_err(CustomError::new)?;

    Ok(client.into())
}
