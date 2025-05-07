use serenity::all::{
    ChannelId, Context, CreateButton, CreateChannel, CreateEmbed, CreateInteractionResponse,
    CreateInteractionResponseMessage, CreateMessage, EmojiId, GuildId, Message,
};

pub async fn verify(
    ctx: &Context,
    msg: Message,
    channel_id: ChannelId,
    guild_id: GuildId,
) -> Result<(), serenity::Error> {
    let author = msg.author.id;
    let author_mention = msg.author;

    let msg = channel_id
        .send_message(
            ctx,
            CreateMessage::new()
                .button(CreateButton::new("button1").emoji(EmojiId::new(1369536103354466384)))
                .embed(
                    CreateEmbed::new()
                        .title("Verify Yourself!")
                        .description("Click the button to get verified!"),
                ),
        )
        .await?;
    loop {
        let button_press = msg.await_component_interaction(&ctx.shard).await;
        match button_press {
            Some(x) => {
                // do permissions
                let channel_name = format!("id-{}", author);
                let channels = guild_id.channels(&ctx.http).await?;
                let mut flag = true;
                let mut existing_channel = String::new();
                for (_, channel) in &channels {
                    if channel.name == channel_name {
                        flag = false;
                        existing_channel = channel.to_string();
                        break;
                    }
                }
                if flag {
                    let builder = CreateChannel::new(channel_name);
                    let new_channel = guild_id.create_channel(&ctx.http, builder).await?;
                    new_channel
                        .send_message(
                            &ctx.http,
                            CreateMessage::new()
                                .content(format!("Come over here {}", author_mention)),
                        )
                        .await?;
                } else {
                    x.create_response(
                        &ctx.http,
                        CreateInteractionResponse::Message(
                            CreateInteractionResponseMessage::new()
                                .content(format!(
                                    "Channel already created! go to {}",
                                    existing_channel
                                ))
                                .ephemeral(true),
                        ),
                    )
                    .await?;
                }
            }
            None => {}
        }
    }
}
