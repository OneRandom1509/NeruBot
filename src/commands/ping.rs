use crate::ShardManagerContainer;
use serenity::all::{Colour, Context, CreateEmbed, CreateMessage, Message};

pub async fn ping(ctx: &Context, msg: Message) -> Result<(), serenity::Error> {
    let data = ctx.data.read().await;

    let shard_manager = match data.get::<ShardManagerContainer>() {
        Some(v) => v,
        None => {
            msg.reply(ctx, "There was a problem getting the shard manager")
                .await?;

            return Ok(());
        }
    };

    let runners = shard_manager.runners.lock().await;

    let runner = match runners.get(&ctx.shard_id) {
        Some(runner) => runner,
        None => {
            msg.reply(ctx, "No shard found").await?;

            return Ok(());
        }
    };

    if let Some(latency) = runner.latency {
        msg.channel_id
            .send_message(
                ctx,
                CreateMessage::new().embed(
                    CreateEmbed::new()
                        .colour(Colour::from_rgb(255, 220, 12))
                        .title("Pong!")
                        .description(format!("The shard latency is {:?}ms", latency.as_millis())),
                ),
            )
            .await?;
    } else {
        msg.reply(ctx, "No ping for you rn").await?;
    }

    Ok(())
}
