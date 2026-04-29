use poise::serenity_prelude::{Colour, CreateEmbed};

use crate::{Context, Error};

#[poise::command(prefix_command, slash_command)]
pub async fn ping(ctx: Context<'_>) -> Result<(), Error> {
    ctx.send(
        poise::CreateReply::default().embed(
            CreateEmbed::new()
                .colour(Colour::from_rgb(255, 220, 12))
                .title("Pong!")
                .description("NeruBot is online and responding."),
        ),
    )
    .await?;

    Ok(())
}
