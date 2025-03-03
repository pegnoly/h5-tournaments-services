use poise::serenity_prelude::*;
use uuid::Uuid;

use crate::{
    graphql::queries::{
        get_tournament_query::GetTournamentQueryTournament, get_user_query::GetUserQueryUser,
    },
    services::{
        challonge::{
            payloads::{ChallongeParticipantAttributes, ChallongeParticipantPayload},
            service::ChallongeService,
        },
        h5_tournaments::{
            payloads::{
                CreateParticipantPayload, CreateUserPayload, DeleteParticipantPayload,
                GetOrganizerPayload, GetParticipantPayload,
            },
            service::H5TournamentsService,
        },
    },
    types::payloads::{GetTournament, GetUser},
};

/// Invoked when user activates button of tournament registration.
pub async fn try_register_in_tournament(
    interaction: &ComponentInteraction,
    context: &Context,
    tournament_service: &H5TournamentsService,
    challonge_service: &ChallongeService,
) -> Result<(), crate::Error> {
    let channel = interaction.channel_id;
    let guild = interaction.guild_id.unwrap();
    let discord_user = &interaction.user;
    let get_tournament_payload =
        GetTournament::default().with_register_channel(channel.get().to_string());
    let get_user_payload = GetUser::default().with_discord_id(discord_user.id.get().to_string());
    let tournament = tournament_service
        .get_tournament_data(get_tournament_payload)
        .await?
        .ok_or(crate::Error::from(format!(
            "No tournament associated with register channel {}",
            channel.get()
        )))?;
    match tournament_service.get_user(get_user_payload).await? {
        Some(system_user) => {
            let get_participant_payload = GetParticipantPayload::default()
                .with_tournament(tournament.id)
                .with_user(system_user.id);
            if tournament_service
                .get_participant(get_participant_payload)
                .await?
                .is_some()
            {
                interaction
                    .create_response(
                        context,
                        CreateInteractionResponse::Message(
                            CreateInteractionResponseMessage::new()
                                .ephemeral(true)
                                .content("Вы уже зарегистрированы в этом турнире"),
                        ),
                    )
                    .await?;
            } else {
                if system_user.registered {
                    register_participant(
                        channel,
                        guild,
                        discord_user,
                        &tournament,
                        &system_user,
                        context,
                        tournament_service,
                        challonge_service,
                    )
                    .await?;
                    interaction
                        .create_response(context, CreateInteractionResponse::Acknowledge)
                        .await?;
                } else {
                    build_registration_modal(interaction, context).await?;
                }
            }
        }
        _ => {
            build_registration_modal(interaction, context).await?;
        }
    }

    Ok(())
}

pub async fn process_registration_modal(
    interaction: &ModalInteraction,
    context: &Context,
    tournament_service: &H5TournamentsService,
    challonge_service: &ChallongeService,
) -> Result<(), crate::Error> {
    let user = &interaction.user;
    let channel = interaction.channel.as_ref().unwrap().id.get();
    let guild = interaction.guild_id.unwrap();
    let mut _nickname = "";
    for row in &interaction.data.components {
        for component in &row.components {
            match component {
                ActionRowComponent::InputText(text) => {
                    if text.custom_id.as_str() == "user_lobby_nickname_input" {
                        _nickname = text.value.as_ref().unwrap();
                        let create_user_payload = CreateUserPayload::new(
                            _nickname.to_string(),
                            user.id.get(),
                            user.name.clone(),
                        );
                        let get_tournament_payload =
                            GetTournament::default().with_register_channel(channel.to_string());
                        let new_user = tournament_service.create_user(create_user_payload).await?;
                        let tournament = tournament_service
                            .get_tournament_data(get_tournament_payload)
                            .await?
                            .ok_or(crate::Error::from(format!(
                                "No tournament associated with register channel: {}",
                                channel
                            )))?;
                        register_participant(
                            ChannelId::from(channel),
                            guild,
                            &interaction.user,
                            &tournament,
                            &GetUserQueryUser::from(new_user),
                            context,
                            tournament_service,
                            challonge_service,
                        )
                        .await?;
                    }
                }
                _ => {}
            }
        }
    }
    interaction
        .create_response(context, CreateInteractionResponse::Acknowledge)
        .await?;
    Ok(())
}

pub async fn try_remove_registration(
    interaction: &ComponentInteraction,
    context: &Context,
    tournament_service: &H5TournamentsService,
    challonge_service: &ChallongeService,
) -> Result<(), crate::Error> {
    let channel = interaction.channel_id;
    let user = &interaction.user;
    let guild = interaction.guild_id.unwrap();
    let tournament = tournament_service
        .get_tournament_data(GetTournament::default().with_register_channel(channel.get().to_string()))
        .await?
        .ok_or(crate::Error::from(format!("No tournament associated with {} register channel", channel.get())))?;
    let organizer = tournament_service
        .get_organizer(GetOrganizerPayload::default().with_id(tournament.organizer))
        .await?
        .ok_or(crate::Error::from(format!("No organizer found with id {}", tournament.organizer)))?;
    match tournament_service
        .get_user(GetUser::default().with_discord_id(user.id.get().to_string()))
        .await?
    {
        Some(system_user) => {
            match tournament_service
                .get_participant(
                    GetParticipantPayload::default()
                        .with_tournament(tournament.id)
                        .with_user(system_user.id),
                )
                .await?
            {
                Some(participant) => {
                    let count = tournament_service
                        .delete_participant(
                            DeleteParticipantPayload::new(tournament.id).with_id(participant.id),
                        )
                        .await?;
                    challonge_service
                        .delete_challonge_participant(
                            &organizer.challonge,
                            &tournament.challonge_id.unwrap(),
                            &participant.challonge.unwrap(),
                        )
                        .await?;
                    let mut roles_to_update = vec![];
                    for role in guild.roles(context).await? {
                        if user.has_role(context, guild, role.0).await?
                            && role.0 != RoleId::from(tournament.role as u64)
                        {
                            roles_to_update.push(role.0);
                        }
                    }
                    guild
                        .edit_member(context, user.id, EditMember::new().roles(roles_to_update))
                        .await?;
                    let output_channel = ChannelId::new(tournament.register_channel as u64);
                    let unregister_message = CreateMessage::new().content(format!(
                        "<@{}> снялся с турнира! Текущее число регистраций: **{}**",
                        system_user.discord_id as u64, count
                    ));
                    output_channel
                        .send_message(context, unregister_message)
                        .await?;
                    interaction
                        .create_response(
                            context,
                            CreateInteractionResponse::Message(
                                CreateInteractionResponseMessage::new()
                                    .content("Вы успешно отменили регистрацию на турнир.")
                                    .ephemeral(true),
                            ),
                        )
                        .await?;
                }
                _ => {
                    interaction
                        .create_response(
                            context,
                            CreateInteractionResponse::Message(
                                CreateInteractionResponseMessage::new()
                                    .content("Вы не являетесь участником этого турнира.")
                                    .ephemeral(true),
                            ),
                        )
                        .await?;
                }
            }
        }
        _ => {
            interaction
                .create_response(
                    context,
                    CreateInteractionResponse::Message(
                        CreateInteractionResponseMessage::new()
                            .content("Вы не зарегистрированы в системе.")
                            .ephemeral(true),
                    ),
                )
                .await?;
        }
    }

    Ok(())
}

pub async fn try_update_user_data(
    interaction: &ComponentInteraction,
    context: &Context,
    api: &H5TournamentsService,
) -> Result<(), crate::Error> {
    let user = &interaction.user;
    match api
        .get_user(GetUser::default().with_discord_id(user.id.get().to_string()))
        .await?
    {
        Some(_existing_user) => {
            build_update_user_modal(interaction, context).await?;
        }
        _ => {
            interaction
                .create_response(
                    context,
                    CreateInteractionResponse::Message(
                        CreateInteractionResponseMessage::new()
                            .ephemeral(true)
                            .content(
                                "Вы не зарегистрированы в системе и не можете менять свои данные",
                            ),
                    ),
                )
                .await?;
        }
    }

    Ok(())
}

pub async fn process_user_update_modal(
    interaction: &ModalInteraction,
    context: &Context,
    api: &H5TournamentsService,
) -> Result<(), crate::Error> {
    let user = interaction.user.id.get();
    let user_data = api
        .get_user(GetUser::default().with_discord_id(user.to_string()))
        .await?
        .unwrap();
    let mut _new_nickname = "";
    for row in &interaction.data.components {
        for component in &row.components {
            match component {
                ActionRowComponent::InputText(text) => {
                    if text.custom_id.as_str() == "user_update_nickname_input" {
                        _new_nickname = text.value.as_ref().unwrap();
                        api.update_user(user_data.id, Some(_new_nickname.to_string()), None)
                            .await?;
                    }
                }
                _ => {}
            }
        }
    }
    interaction
        .create_response(context, CreateInteractionResponse::Acknowledge)
        .await?;
    Ok(())
}

async fn build_registration_modal(
    interaction: &ComponentInteraction,
    context: &Context,
) -> Result<(), crate::Error> {
    interaction
        .create_response(
            context,
            CreateInteractionResponse::Modal(
                CreateModal::new("user_lobby_nickname_modal", "Укажите свой никнейм в лобби")
                    .components(vec![CreateActionRow::InputText(CreateInputText::new(
                        InputTextStyle::Short,
                        "Укажите свой никнейм в лобби",
                        "user_lobby_nickname_input",
                    ))]),
            ),
        )
        .await?;
    Ok(())
}

async fn build_update_user_modal(
    interaction: &ComponentInteraction,
    context: &Context,
) -> Result<(), crate::Error> {
    interaction
        .create_response(
            context,
            CreateInteractionResponse::Modal(
                CreateModal::new(
                    "user_update_nickname_modal",
                    "Здесь вы можете изменить свой никнейм в лобби",
                )
                .components(vec![CreateActionRow::InputText(
                    CreateInputText::new(
                        InputTextStyle::Short,
                        "Укажите новый никнейм",
                        "user_update_nickname_input",
                    ),
                )]),
            ),
        )
        .await?;
    Ok(())
}

async fn register_participant(
    channel: ChannelId,
    guild: GuildId,
    discord_user: &User,
    tournament: &GetTournamentQueryTournament,
    user: &GetUserQueryUser,
    context: &Context,
    tournaments_service: &H5TournamentsService,
    challonge_service: &ChallongeService,
) -> Result<(), crate::Error> {
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
        guild
            .edit_member(
                context,
                discord_user.id,
                EditMember::new().roles(existing_roles),
            )
            .await?;
    } else {
        tracing::info!(
            "User {:?} who already have participant role tries to get it twice",
            discord_user
        );
        return Ok(());
    }
    let organizer = tournaments_service
        .get_organizer(GetOrganizerPayload::default().with_id(tournament.organizer))
        .await?
        .unwrap();
    let participant_data = challonge_service
        .create_challonge_participant(
            &organizer.challonge,
            &tournament.id.to_string(),
            ChallongeParticipantPayload {
                _type: crate::services::challonge::payloads::ChallongePayloadType::Participants,
                attributes: Some(ChallongeParticipantAttributes {
                    name: user.nickname.clone(),
                    seed: Some(1),
                    misc: Some(user.id.to_string()),
                    email: Some(String::new()),
                    username: Some(String::new()),
                }),
            },
        )
        .await?;
    let create_participant_payload =
        CreateParticipantPayload::new(tournament.id, user.id, participant_data.id);
    let count = tournaments_service
        .create_participant(create_participant_payload)
        .await?;
    let registered_message = CreateMessage::new().content(format!(
        "<@{}> зарегистрировался в турнире! Всего регистраций: **{}**",
        discord_user.id.get(),
        count
    ));
    channel.send_message(context, registered_message).await?;
    Ok(())
}
