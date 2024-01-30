use crate::{db::get_timezone, Context, Error};
use chrono_tz::Tz;

/// Show this help menu
#[poise::command(prefix_command, track_edits, slash_command)]
pub async fn help(
    ctx: Context<'_>,
    #[description = "Specific command to show help about"]
    #[autocomplete = "poise::builtins::autocomplete_command"]
    command: Option<String>,
) -> Result<(), Error> {
    poise::builtins::help(
        ctx,
        command.as_deref(),
        poise::builtins::HelpConfiguration {
            extra_text_at_bottom: "Who can say first, first?",
            ..Default::default()
        },
    )
    .await?;
    Ok(())
}

/// Change your timezone
#[poise::command(prefix_command, track_edits, slash_command)]
pub async fn change_timezone(
    ctx: Context<'_>,
    #[description = "Timezone"] timezone: String,
) -> Result<(), Error> {
    if let Err(err) = timezone.parse::<Tz>() {
        ctx.send(
            poise::CreateReply::default()
                .content(err)
                .reply(true)
                .ephemeral(true),
        )
        .await?;

        return Ok(());
    }

    let author_id: i64 = ctx.author().id.get().try_into()?;

    sqlx::query!(
        "INSERT INTO users (id, timezone)
            VALUES ($1, $2)
            ON CONFLICT (id) DO UPDATE SET timezone = $2
        ",
        author_id,
        timezone
    )
    .execute(&ctx.data().pool)
    .await?;

    ctx.send(
        poise::CreateReply::default()
            .content(format!("Registered {timezone} as your timezone"))
            .reply(true)
            .ephemeral(true),
    )
    .await?;

    Ok(())
}

/// Query your timezone
#[poise::command(prefix_command, track_edits, slash_command)]
pub async fn timezone(ctx: Context<'_>) -> Result<(), Error> {
    let author_id: i64 = ctx.author().id.get().try_into()?;

    let timezone = match get_timezone(author_id, &ctx.data().pool).await? {
        Some(timezone) => timezone,
        None => {
            ctx.send(
                poise::CreateReply::default()
                    .content("You do not have a registered timezone")
                    .reply(true)
                    .ephemeral(true),
            )
            .await?;

            return Ok(());
        }
    };

    ctx.send(
        poise::CreateReply::default()
            .content(format!("Your timezone is {timezone}"))
            .reply(true)
            .ephemeral(true),
    )
    .await?;

    Ok(())
}
