use crate::{Context, Error};

#[poise::command(prefix_command, slash_command)]
pub async fn http(ctx: Context<'_>, status_code: Option<u32>) -> Result<(), Error> {
    let status_code = status_code.unwrap_or(400);
    ctx.say(format!("https://http.cat/{status_code}")).await?;
    Ok(())
}
