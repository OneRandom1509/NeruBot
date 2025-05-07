use commands::verify;
use serenity::model::prelude::*;
use serenity::prelude::*;

pub mod commands;

async fn message(ctx: &Context, msg: Message) -> Result<(), serenity::Error> {
    let channel_id = msg.channel_id;
    // the default guild id is 1234 (which is not a real guild)
    let guild_id = msg.guild_id.unwrap_or(GuildId::new(1234));

    if let Some(args) = msg.content.strip_prefix("!") {
        match args {
            "verify" => {
                verify::verify(ctx, msg, channel_id, guild_id).await?;
            }
            _ => {}
        }
    }
    Ok(())
}

struct Handler;
#[serenity::async_trait]
impl EventHandler for Handler {
    async fn message(&self, ctx: Context, msg: Message) {
        message(&ctx, msg).await.unwrap();
    }
}

#[tokio::main]
async fn main() {
    dotenv::dotenv().expect("Failed to load .env file");
    let token = std::env::var("DISCORD_TOKEN").expect("Expected a token in the environment");
    let intents = GatewayIntents::non_privileged() | GatewayIntents::MESSAGE_CONTENT;

    let mut client = Client::builder(token, intents)
        .event_handler(Handler)
        .await
        .unwrap();
    client.start().await.unwrap();
}
