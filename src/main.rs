use std::sync::Arc;

use commands::{http_cat, ping, verify};
use serenity::all::ShardManager;
use serenity::model::prelude::*;
use serenity::prelude::*;

pub mod commands;

pub type SerenityResult = Result<(), SerenityError>;

async fn message(ctx: &Context, msg: Message) -> SerenityResult {
    let channel_id = msg.channel_id;
    // the default guild id is 1234 (which is not a real guild)
    let guild_id = msg.guild_id.unwrap_or(GuildId::new(1234));

    if let Some(args) = msg.content.strip_prefix("!") {
        let command: Vec<&str> = args.split(' ').collect();
        match *command.get(0).unwrap() {
            "verify" => {
                verify::verify(ctx, msg, channel_id, guild_id).await?;
            }
            "ping" => {
                ping::ping(ctx, msg).await?;
            }
            "http" => {
                if let Some(number) = command.get(1) {
                    http_cat::http(ctx, msg.clone(), number.parse::<u32>().unwrap_or(400)).await?;
                } else {
                    msg.reply_ping(ctx, "Please specify an http status code :3")
                        .await?;
                }
            }
            _ => {
                msg.reply_ping(ctx, "There's no command like that idiot!")
                    .await?;
            }
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

pub struct ShardManagerContainer;
impl TypeMapKey for ShardManagerContainer {
    type Value = Arc<ShardManager>;
}

#[tokio::main]
async fn main() {
    let token = std::env::var("DISCORD_TOKEN").expect("Expected a token in the environment");
    let intents = GatewayIntents::all();

    let mut client = Client::builder(token, intents)
        .event_handler(Handler)
        .await
        .unwrap();

    {
        let mut data = client.data.write().await;
        data.insert::<ShardManagerContainer>(Arc::clone(&client.shard_manager));
    }

    if let Err(why) = client.start().await {
        println!("Err with client: {:?}", why);
    }
}
