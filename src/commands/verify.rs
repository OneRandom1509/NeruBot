use std::time::Duration;

use poise::serenity_prelude::{
    ButtonStyle, ChannelType, CreateButton, CreateChannel, CreateEmbed, CreateInteractionResponse,
    CreateInteractionResponseMessage, CreateMessage, PermissionOverwrite, PermissionOverwriteType,
    Permissions, RoleId,
};

use crate::{Context, Error};

#[poise::command(slash_command, guild_only, required_permissions = "ADMINISTRATOR")]
pub async fn verify(
    ctx: Context<'_>,
    #[description = "Category name where tickets should be created"] category_name: String,
    #[description = "Message content for the ticket initializer"] initializer_message: String,
    #[description = "Message sent in the created ticket channel (the user is always pinged)"]
    ticket_ping_message: String,
    #[description = "Button label (optional)"] button_label: Option<String>,
) -> Result<(), Error> {
    let serenity_ctx = ctx.serenity_context();
    let channel_id = ctx.channel_id();
    let Some(guild_id) = ctx.guild_id() else {
        ctx.say("This command can only be used inside a server.")
            .await?;
        return Ok(());
    };

    let bot_user_id = serenity_ctx.http.get_current_user().await?.id;

    let category_id = {
        let channels = guild_id.channels(&serenity_ctx.http).await?;
        if let Some(existing_category) = channels.values().find(|channel| {
            channel.kind == ChannelType::Category
                && channel.name.eq_ignore_ascii_case(category_name.as_str())
        }) {
            existing_category.id
        } else {
            guild_id
                .create_channel(
                    &serenity_ctx.http,
                    CreateChannel::new(category_name.as_str()).kind(ChannelType::Category),
                )
                .await?
                .id
        }
    };

    let button_text = button_label.unwrap_or_else(|| "Open Ticket".to_string());
    let msg = channel_id
        .send_message(
            serenity_ctx,
            CreateMessage::new()
                .embed(
                    CreateEmbed::new()
                        .colour(poise::serenity_prelude::Colour::from_rgb(255, 220, 12))
                        .title("Ticket Panel")
                        .description(initializer_message),
                )
                .button(
                    CreateButton::new("verify_ticket_open")
                        .label(button_text)
                        .style(ButtonStyle::Primary),
                ),
        )
        .await?;

    ctx.send(
        poise::CreateReply::default()
            .content(format!(
                "Ticket panel created in this channel. Tickets will be created under category <#{}>.",
                category_id
            ))
            .ephemeral(true),
    )
    .await?;

    let serenity_ctx = serenity_ctx.clone();
    tokio::spawn(async move {
        while let Some(interaction) = msg
            .await_component_interaction(&serenity_ctx.shard)
            .timeout(Duration::from_secs(60 * 60 * 24 * 7))
            .await
        {
            let user_id = interaction.user.id;

            let open_ticket_result = async {
                let channels = guild_id.channels(&serenity_ctx.http).await?;

                if let Some(existing_channel) = channels.values().find(|channel| {
                    channel.parent_id == Some(category_id)
                        && channel.topic.as_deref()
                            == Some(&format!("ticket-owner:{}", user_id.get()))
                }) {
                    interaction
                        .create_response(
                            &serenity_ctx.http,
                            CreateInteractionResponse::Message(
                                CreateInteractionResponseMessage::new()
                                    .content(format!(
                                        "You already have an open ticket: {}",
                                        format!("<#{}>", existing_channel.id.get())
                                    ))
                                    .ephemeral(true),
                            ),
                        )
                        .await?;
                    return Ok::<(), Box<dyn std::error::Error + Send + Sync>>(());
                }

                let user_name_slug = interaction
                    .user
                    .name
                    .chars()
                    .map(|c| {
                        if c.is_ascii_alphanumeric() {
                            c.to_ascii_lowercase()
                        } else {
                            '-'
                        }
                    })
                    .collect::<String>();

                let ticket_name = format!("ticket-{user_name_slug}-{}", user_id.get());

                let new_channel = guild_id
                    .create_channel(
                        &serenity_ctx.http,
                        CreateChannel::new(ticket_name)
                            .kind(ChannelType::Text)
                            .category(category_id)
                            .topic(format!("ticket-owner:{}", user_id.get()))
                            .permissions(vec![
                                PermissionOverwrite {
                                    allow: Permissions::empty(),
                                    deny: Permissions::VIEW_CHANNEL,
                                    kind: PermissionOverwriteType::Role(RoleId::new(
                                        guild_id.get(),
                                    )),
                                },
                                PermissionOverwrite {
                                    allow: Permissions::VIEW_CHANNEL
                                        | Permissions::SEND_MESSAGES
                                        | Permissions::READ_MESSAGE_HISTORY
                                        | Permissions::ATTACH_FILES,
                                    deny: Permissions::empty(),
                                    kind: PermissionOverwriteType::Member(user_id),
                                },
                                PermissionOverwrite {
                                    allow: Permissions::VIEW_CHANNEL
                                        | Permissions::SEND_MESSAGES
                                        | Permissions::READ_MESSAGE_HISTORY
                                        | Permissions::MANAGE_CHANNELS,
                                    deny: Permissions::empty(),
                                    kind: PermissionOverwriteType::Member(bot_user_id),
                                },
                            ]),
                    )
                    .await?;

                let ticket_msg = new_channel
                    .send_message(
                        &serenity_ctx.http,
                        CreateMessage::new()
                            .embed(
                                CreateEmbed::new()
                                    .colour(poise::serenity_prelude::Colour::from_rgb(255, 220, 12))
                                    .title("Ticket")
                                    .description(ticket_ping_message.clone()),
                            )
                            .button(
                                CreateButton::new("close_ticket")
                                    .label("Close Ticket")
                                    .style(ButtonStyle::Danger),
                            ),
                    )
                    .await?;

                // Spawn a short-lived task to wait for a close-button click on the ticket message.
                let serenity_ctx_clone = serenity_ctx.clone();
                let new_channel_clone = new_channel.clone();
                tokio::spawn(async move {
                    if let Some(close_interaction) = ticket_msg
                        .await_component_interaction(&serenity_ctx_clone.shard)
                        .timeout(Duration::from_secs(60 * 60 * 24 * 7))
                        .await
                    {
                        // Only allow the ticket owner to close the channel (user who opened it)
                        if close_interaction.user.id == user_id {
                            let _ = new_channel_clone.delete(&serenity_ctx_clone.http).await;
                            let _ = close_interaction
                                .create_response(
                                    &serenity_ctx_clone.http,
                                    CreateInteractionResponse::Message(
                                        CreateInteractionResponseMessage::new()
                                            .content("Ticket closed and channel deleted.")
                                            .ephemeral(true),
                                    ),
                                )
                                .await;
                        } else {
                            let _ = close_interaction
                                .create_response(
                                    &serenity_ctx_clone.http,
                                    CreateInteractionResponse::Message(
                                        CreateInteractionResponseMessage::new()
                                            .content("Only the ticket owner can close this ticket.")
                                            .ephemeral(true),
                                    ),
                                )
                                .await;
                        }
                    }
                });

                interaction
                    .create_response(
                        &serenity_ctx.http,
                        CreateInteractionResponse::Message(
                            CreateInteractionResponseMessage::new()
                                .content(format!(
                                    "Ticket created: {}",
                                    format!("<#{}>", new_channel.id.get())
                                ))
                                .ephemeral(true),
                        ),
                    )
                    .await?;

                Ok::<(), Box<dyn std::error::Error + Send + Sync>>(())
            }
            .await;

            if let Err(err) = open_ticket_result {
                eprintln!("Error while handling ticket interaction: {err}");

                let _ = interaction
                    .create_response(
                        &serenity_ctx.http,
                        CreateInteractionResponse::Message(
                            CreateInteractionResponseMessage::new()
                                .content(
                                    "Could not create your ticket. Please contact a moderator.",
                                )
                                .ephemeral(true),
                        ),
                    )
                    .await;
            }
        }
    });

    Ok(())
}
