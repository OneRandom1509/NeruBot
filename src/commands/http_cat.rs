use serenity::all::{Context, Message};

use crate::SerenityResult;

pub async fn http(ctx: &Context, msg: Message, status_code: u32) -> SerenityResult {
    msg.reply(ctx, format!("https://http.cat/{}", status_code))
        .await?;
    Ok(())
}
