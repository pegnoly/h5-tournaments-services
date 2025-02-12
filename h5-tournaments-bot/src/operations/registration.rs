use poise::serenity_prelude::*;
use uuid::Uuid;

use crate::{api_connector::service::ApiConnectionService, graphql::queries::get_tournament_query::GetTournamentQueryTournament};

pub async fn try_register_in_tournament(
    interaction: &ComponentInteraction,
    context: &Context,
    api: &ApiConnectionService
) -> Result<(), crate::Error> {
    let channel = interaction.channel_id;
    let guild = interaction.guild_id.unwrap();
    let user = &interaction.user;
    let tournament = api.get_tournament_data(None, None, Some(channel.get().to_string())).await?.unwrap();
                
    if let Some(existing_user) = api.get_user(None, Some(user.id.get().to_string())).await? {
        if let Some(_existing_participant) = api.get_participant(tournament.id, existing_user.id).await? {
            interaction.create_response(context, CreateInteractionResponse::Message(
                CreateInteractionResponseMessage::new().ephemeral(true).content("Вы уже зарегистрированы в этом турнире")
            )).await?;
        }
        else {
            if existing_user.registered {
                register_participant(channel, guild, user, &tournament, existing_user.id, context, api).await?;
                interaction.create_response(context, CreateInteractionResponse::Acknowledge).await?;
            }
            else {
                build_registration_modal(interaction, context).await?;
            }
        }
    } else {
        build_registration_modal(interaction, context).await?;
    }

    Ok(())
}

pub async fn process_registration_modal(
    interaction: &ModalInteraction,
    context: &Context,
    api: &ApiConnectionService
) -> Result<(), crate::Error> {
    let user = interaction.user.id.get();
    let channel = interaction.channel.as_ref().unwrap().id.get();
    let guild = interaction.guild_id.unwrap();
    let mut nickname = "";
    for row in &interaction.data.components {
        for component in &row.components {
            match component {
                ActionRowComponent::InputText(text) => {
                    if text.custom_id.as_str() == "user_lobby_nickname_input" {
                        nickname = text.value.as_ref().unwrap();
                        let new_user_id = api.create_user(nickname.to_string(), user.to_string(), true).await?;
                        let tournament = api.get_tournament_data(None, None, Some(channel.to_string())).await?.unwrap();
                        register_participant(
                            ChannelId::from(channel), 
                            guild, 
                            &interaction.user, 
                            &tournament, 
                            new_user_id, 
                            context, 
                            api
                        ).await?;
                    }
                },
                _=> {}
            }
        }
    }
    interaction.create_response(context, CreateInteractionResponse::Acknowledge).await?;
    Ok(())
}

pub async fn try_remove_registration(
    interaction: &ComponentInteraction,
    context: &Context,
    api: &ApiConnectionService
) -> Result<(), crate::Error> {
    let channel = interaction.channel_id;
    let user = &interaction.user;
    let guild = interaction.guild_id.unwrap();
    let tournament = api.get_tournament_data(None, None, Some(channel.get().to_string())).await?.unwrap();
                
    if let Some(existing_user) = api.get_user(None, Some(user.id.get().to_string())).await? {
        if let Some(_participant) = api.get_participant(tournament.id, existing_user.id).await? {
            api.delete_participant(tournament.id, existing_user.id).await?;
            let mut roles_to_update = vec![];
            for role in guild.roles(context).await? {
                if user.has_role(context, guild, role.0).await? && role.0 != RoleId::from(tournament.role as u64) {
                    roles_to_update.push(role.0);
                }
            }
            guild.edit_member(context, user.id, EditMember::new().roles(roles_to_update)).await?;
            interaction.create_response(context, CreateInteractionResponse::Message(
                CreateInteractionResponseMessage::new().content("Вы успешно отменили регистрацию на турнир.").ephemeral(true)
            )).await?;
        } else {
            interaction.create_response(context, CreateInteractionResponse::Message(
                CreateInteractionResponseMessage::new().content("Вы не являетесь участником этого турнира.").ephemeral(true)
            )).await?; 
        }
    }
    else {
        interaction.create_response(context, CreateInteractionResponse::Message(
            CreateInteractionResponseMessage::new().content("Вы не зарегистрированы в системе.").ephemeral(true)
        )).await?;
    }

    Ok(())
}

pub async fn try_update_user_data(
    interaction: &ComponentInteraction,
    context: &Context,
    api: &ApiConnectionService
) -> Result<(), crate::Error> {
    let user = &interaction.user;
    if let Some(_existing_user) = api.get_user(None, Some(user.id.get().to_string())).await? {
        build_update_user_modal(interaction, context).await?;
    } else {
        interaction.create_response(context, CreateInteractionResponse::Message(
            CreateInteractionResponseMessage::new().ephemeral(true).content("Вы не зарегистрированы в системе и не можете менять свои данные")
        )).await?;
    }

    Ok(())
}

pub async fn process_user_update_modal(
    interaction: &ModalInteraction,
    context: &Context,
    api: &ApiConnectionService
) -> Result<(), crate::Error> {
    let user = interaction.user.id.get();
    let user_data = api.get_user(None, Some(user.to_string())).await?.unwrap();
    let mut new_nickname = "";
    for row in &interaction.data.components {
        for component in &row.components {
            match component {
                ActionRowComponent::InputText(text) => {
                    if text.custom_id.as_str() == "user_update_nickname_input" {
                        new_nickname = text.value.as_ref().unwrap();
                        api.update_user(user_data.id, Some(new_nickname.to_string()), None).await?;
                    }
                },
                _=> {}
            }
        }
    }
    interaction.create_response(context, CreateInteractionResponse::Acknowledge).await?;
    Ok(())
}

async fn build_registration_modal(
    interaction: &ComponentInteraction,
    context: &Context
) -> Result<(), crate::Error> {
    interaction.create_response(context, CreateInteractionResponse::Modal(
        CreateModal::new("user_lobby_nickname_modal", "Укажите свой никнейм в лобби")
            .components(vec![
                CreateActionRow::InputText(
                    CreateInputText::new(InputTextStyle::Short, "Укажите свой никнейм в лобби", "user_lobby_nickname_input")
                )
            ])
    )).await?; 
    Ok(())
}

async fn build_update_user_modal(
    interaction: &ComponentInteraction,
    context: &Context
) -> Result<(), crate::Error> {
    interaction.create_response(context, CreateInteractionResponse::Modal(
        CreateModal::new("user_update_nickname_modal", "Здесь вы можете изменить свой никнейм в лобби")
            .components(vec![
                CreateActionRow::InputText(
                    CreateInputText::new(InputTextStyle::Short, "Укажите новый никнейм", "user_update_nickname_input")
                )
            ])
    )).await?; 
    Ok(())
}

async fn register_participant(
    channel: ChannelId,
    guild: GuildId,
    discord_user: &User,
    tournament: &GetTournamentQueryTournament,
    tournament_user_id: Uuid,
    context: &Context,
    api: &ApiConnectionService
) -> Result<(), crate::Error> {
    let count = api.create_participant(tournament.id, tournament_user_id, 0).await?;
    let registered_message = CreateMessage::new()
        .content(format!("<@{}> зарегистрировался в турнире! Всего регистраций: **{}**", discord_user.id.get(), count));
    channel.send_message(context, registered_message).await?;
    let mut existing_roles = vec![];
    for role in guild.roles(context).await? {
        if discord_user.has_role(context, guild, role.0).await? {
            tracing::info!("User has {} role ", role.1.to_string());
            existing_roles.push(role.0);
        }
    }
    let tournament_role = RoleId::from(tournament.role as u64);
    if !existing_roles.contains(&tournament_role) {
        existing_roles.push(tournament_role);
        guild.edit_member(context, discord_user.id, EditMember::new().roles(existing_roles)).await?;
    }
    Ok(())
}