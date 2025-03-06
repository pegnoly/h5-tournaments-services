use std::{collections::HashMap, str::FromStr};

use crate::{
    builders::{self, tournament_creation::rebuild_tournament_creation_interface, types::{GameType, TournamentBuildState, TournamentBuilder}},
    event_handler::LocalSyncBuilder,
    graphql::queries::update_participants_bulk::UpdateParticipant,
    services::{
        challonge::{
            payloads::ChallongeParticipantAttributes,
            service::ChallongeService,
        },
        h5_tournaments::{
            payloads::{
                CreateOrganizerPayload, CreateTournamentPayload, GetOperatorPayload, GetOrganizerPayload, GetTournamentBuilderPayload, UpdateTournamentPayload
            },
            service::H5TournamentsService,
        },
    },
    types::payloads::GetTournament,
};
use h5_tournaments_api::prelude::ModType;
use poise::serenity_prelude::*;
use strum::{Display, EnumString};
use tokio::sync::RwLock;
use uuid::Uuid;

pub async fn start_admin_registration(
    context: &Context,
    interaction: &ComponentInteraction,
    service: &H5TournamentsService,
) -> Result<(), crate::Error> {
    match service
        .get_organizer(
            GetOrganizerPayload::default().with_discord_id(interaction.user.id.get() as i64),
        )
        .await?
    {
        Some(_existing_organizer) => {
            interaction
                .create_response(
                    context,
                    CreateInteractionResponse::Message(
                        CreateInteractionResponseMessage::new()
                            .ephemeral(true)
                            .content("Вы уже зарегистрированы как организатор турниров"),
                    ),
                )
                .await?;
        }
        _ => {
            interaction
                .create_response(
                    context,
                    CreateInteractionResponse::Modal(
                        CreateModal::new(
                            "tournament_admin_challonge_key_modal",
                            "Укажите свой Challonge.com ключ API",
                        )
                        .components(vec![CreateActionRow::InputText(
                            CreateInputText::new(
                                InputTextStyle::Short,
                                "Введите свой ключ API",
                                "tournament_admin_challonge_key_input",
                            ),
                        )]),
                    ),
                )
                .await?;
        }
    }
    Ok(())
}

pub async fn process_admin_registration_modal(
    context: &Context,
    interaction: &ModalInteraction,
    service: &H5TournamentsService,
) -> Result<(), crate::Error> {
    let user = interaction.user.id.get();
    for row in &interaction.data.components {
        for component in &row.components {
            match component {
                ActionRowComponent::InputText(text) => {
                    if text.custom_id.as_str() == "tournament_admin_challonge_key_input" {
                        service
                            .create_organizer(CreateOrganizerPayload::new(
                                user,
                                text.value.as_ref().unwrap().to_owned(),
                            ))
                            .await?;
                        break;
                    }
                }
                _ => {}
            }
        }
    }
    interaction
        .create_response(
            context,
            CreateInteractionResponse::Message(
                CreateInteractionResponseMessage::new()
                    .ephemeral(true)
                    .content("Вы успешно зарегистрированы, как организатор турниров."),
            ),
        )
        .await?;
    Ok(())
}

pub async fn start_tournament_name_creation(
    context: &Context,
    interaction: &ComponentInteraction,
    service: &H5TournamentsService,
) -> Result<(), crate::Error> {
    let message = interaction.message.id.get();
    let payload = GetTournamentBuilderPayload::default().with_message(message);
    match service.get_tournament_builder(payload).await? {
        Some(existing_builder) => {
            interaction
                .create_response(
                    context,
                    CreateInteractionResponse::Modal(
                        builders::tournament_creation::build_tournament_name_modal(
                            existing_builder.name,
                        )
                        .await,
                    ),
                )
                .await?;
        }
        _ => {
            let _new_builder = service.create_tournament_builder(message).await?;
            interaction
                .create_response(
                    context,
                    CreateInteractionResponse::Modal(
                        builders::tournament_creation::build_tournament_name_modal(None).await,
                    ),
                )
                .await?;
        }
    }

    Ok(())
}

pub async fn process_tournament_name_creation_modal(
    context: &Context,
    interaction: &ModalInteraction,
    tournament_builders: &RwLock<HashMap<u64, RwLock<TournamentBuilder>>>
) -> Result<(), crate::Error> {
    let message = interaction.message.as_ref().unwrap().id.get();
    for row in &interaction.data.components {
        for component in &row.components {
            match component {
                ActionRowComponent::InputText(text) => {
                    if text.custom_id.as_str() == "tournament_creation_name_input" {
                        let builders_locked = tournament_builders.read().await;
                        if let Some(builder) = builders_locked.get(&message) {
                            let mut builder_locked = builder.write().await;
                            builder_locked.name = Some(text.value.clone().unwrap_or(String::new()));
                            let response_message = rebuild_tournament_creation_interface(&builder_locked.downgrade()).await;
                            interaction.create_response(context, CreateInteractionResponse::UpdateMessage(response_message)).await?;
                        }
                        return Ok(());
                    }
                }
                _ => {}
            }
        }
    }
    Ok(())
}

pub async fn process_tournament_game_type_selection(
    context: &Context,
    interaction: &ComponentInteraction,
    tournament_builders: &RwLock<HashMap<u64, RwLock<TournamentBuilder>>>,
    selected_value: &String
) -> Result<(), crate::Error> {
    let message = interaction.message.id.get();
    let builders_locked = tournament_builders.read().await;
    if let Some(builder) = builders_locked.get(&message) {
        let mut builder_locked = builder.write().await;
        builder_locked.game_type = Some(GameType::from_str(&selected_value)?);
        let response_message = rebuild_tournament_creation_interface(&builder_locked.downgrade()).await;
        interaction.create_response(context, CreateInteractionResponse::UpdateMessage(response_message)).await?;
    }
    Ok(())
}

pub async fn process_tournament_mod_type_selection(
    context: &Context,
    interaction: &ComponentInteraction,
    tournament_builders: &RwLock<HashMap<u64, RwLock<TournamentBuilder>>>,
    selected_value: &String
) -> Result<(), crate::Error> {
    let message = interaction.message.id.get();
    let builders_locked = tournament_builders.read().await;
    if let Some(builder) = builders_locked.get(&message) {
        let mut builder_locked = builder.write().await;
        builder_locked.mod_type = Some(ModType::from_str(&selected_value)?);
        let response_message = rebuild_tournament_creation_interface(&builder_locked.downgrade()).await;
        interaction.create_response(context, CreateInteractionResponse::UpdateMessage(response_message)).await?;
    }
    Ok(())
}

pub async fn process_tournament_builder_state_change(
    context: &Context,
    interaction: &ComponentInteraction,
    tournament_builders: &RwLock<HashMap<u64, RwLock<TournamentBuilder>>>,
    new_state: TournamentBuildState
) -> Result<(), crate::Error> {
    let message = interaction.message.id.get();
    let builders_locked = tournament_builders.read().await;
    if let Some(builder) = builders_locked.get(&message) {
        let mut builder_locked = builder.write().await;
        builder_locked.edit_state = new_state;
        let response_message = rebuild_tournament_creation_interface(&builder_locked.downgrade()).await;
        interaction.create_response(context, CreateInteractionResponse::UpdateMessage(response_message)).await?;
    }
    Ok(())
}

pub async fn select_tournament_builder_register_channel(
    context: &Context,
    interaction: &ComponentInteraction,
    tournament_builders: &RwLock<HashMap<u64, RwLock<TournamentBuilder>>>,
    selected_value: u64,
) -> Result<(), crate::Error> {
    let message = interaction.message.id.get();
    let builders_locked = tournament_builders.read().await;
    if let Some(builder) = builders_locked.get(&message) {
        let mut builder_locked = builder.write().await;
        builder_locked.register_channel = Some(selected_value);
        let response_message = rebuild_tournament_creation_interface(&builder_locked.downgrade()).await;
        interaction.create_response(context, CreateInteractionResponse::UpdateMessage(response_message)).await?;
    }
    Ok(())
}

pub async fn select_tournament_builder_reports_channel(
    context: &Context,
    interaction: &ComponentInteraction,
    tournament_builders: &RwLock<HashMap<u64, RwLock<TournamentBuilder>>>,
    selected_value: u64,
) -> Result<(), crate::Error> {
    let message = interaction.message.id.get();
    let builders_locked = tournament_builders.read().await;
    if let Some(builder) = builders_locked.get(&message) {
        let mut builder_locked = builder.write().await;
        builder_locked.reports_channel = Some(selected_value);
        let response_message = rebuild_tournament_creation_interface(&builder_locked.downgrade()).await;
        interaction.create_response(context, CreateInteractionResponse::UpdateMessage(response_message)).await?;
    }
    Ok(())
}

pub async fn select_tournament_builder_role(
    context: &Context,
    interaction: &ComponentInteraction,
    tournament_builders: &RwLock<HashMap<u64, RwLock<TournamentBuilder>>>,
    selected_value: u64,
) -> Result<(), crate::Error> {
    let message = interaction.message.id.get();
    let builders_locked = tournament_builders.read().await;
    if let Some(builder) = builders_locked.get(&message) {
        let mut builder_locked = builder.write().await;
        builder_locked.role = Some(selected_value);
        let response_message = rebuild_tournament_creation_interface(&builder_locked.downgrade()).await;
        interaction.create_response(context, CreateInteractionResponse::UpdateMessage(response_message)).await?;
    }
    Ok(())
}

#[derive(Debug, EnumString, Display)]
pub enum BargainsUsageType {
    UseBargains,
    DontUseBargains,
}

impl Into<bool> for BargainsUsageType {
    fn into(self) -> bool {
        match self {
            BargainsUsageType::UseBargains => true,
            BargainsUsageType::DontUseBargains => false,
        }
    }
}

pub async fn select_tournament_builder_bargains_usage(
    context: &Context,
    interaction: &ComponentInteraction,
    tournament_builders: &RwLock<HashMap<u64, RwLock<TournamentBuilder>>>,
    selected_value: &String,
) -> Result<(), crate::Error> {
    let message = interaction.message.id.get();
    let builders_locked = tournament_builders.read().await;
    if let Some(builder) = builders_locked.get(&message) {
        let mut builder_locked = builder.write().await;
        builder_locked.use_bargains = Some(BargainsUsageType::from_str(&selected_value)?.into());
        let response_message = rebuild_tournament_creation_interface(&builder_locked.downgrade()).await;
        interaction.create_response(context, CreateInteractionResponse::UpdateMessage(response_message)).await?;
    }
    Ok(())
}

#[derive(Debug, EnumString, Display)]
pub enum BargainsColorUsageType {
    UseBargainsColor,
    DontUseBargainsColor,
}

impl Into<bool> for BargainsColorUsageType {
    fn into(self) -> bool {
        match self {
            BargainsColorUsageType::UseBargainsColor => true,
            BargainsColorUsageType::DontUseBargainsColor => false,
        }
    }
}

pub async fn select_tournament_builder_bargains_color_usage(
    context: &Context,
    interaction: &ComponentInteraction,
    tournament_builders: &RwLock<HashMap<u64, RwLock<TournamentBuilder>>>,
    selected_value: &String,
) -> Result<(), crate::Error> {
    let message = interaction.message.id.get();
    let builders_locked = tournament_builders.read().await;
    if let Some(builder) = builders_locked.get(&message) {
        let mut builder_locked = builder.write().await;
        builder_locked.use_bargains_color = Some(BargainsColorUsageType::from_str(&selected_value)?.into());
        let response_message = rebuild_tournament_creation_interface(&builder_locked.downgrade()).await;
        interaction.create_response(context, CreateInteractionResponse::UpdateMessage(response_message)).await?;
    }
    Ok(())
}

#[derive(Debug, EnumString, Display)]
pub enum ForeignHeroesUsageType {
    UseForeignHeroes,
    DontUseForeignHeroes,
}

impl Into<bool> for ForeignHeroesUsageType {
    fn into(self) -> bool {
        match self {
            ForeignHeroesUsageType::UseForeignHeroes => true,
            ForeignHeroesUsageType::DontUseForeignHeroes => false,
        }
    }
}

pub async fn select_tournament_builder_foreign_heroes_usage(
    context: &Context,
    interaction: &ComponentInteraction,
    tournament_builders: &RwLock<HashMap<u64, RwLock<TournamentBuilder>>>,
    selected_value: &String,
) -> Result<(), crate::Error> {
    let message = interaction.message.id.get();
    let builders_locked = tournament_builders.read().await;
    if let Some(builder) = builders_locked.get(&message) {
        let mut builder_locked = builder.write().await;
        builder_locked.use_foreign_heroes = Some(ForeignHeroesUsageType::from_str(&selected_value)?.into());
        let response_message = rebuild_tournament_creation_interface(&builder_locked.downgrade()).await;
        interaction.create_response(context, CreateInteractionResponse::UpdateMessage(response_message)).await?;
    }
    Ok(())
}

pub async fn finalize_tournament_creation(
    context: &Context,
    interaction: &ComponentInteraction,
    tournament_builders: &RwLock<HashMap<u64, RwLock<TournamentBuilder>>>,
    service: &H5TournamentsService,
) -> Result<(), crate::Error> {
    let message = interaction.message.id.get();
    let guild = interaction.guild_id.unwrap().get();
    let discord_user = interaction.user.id.get();
    let operator_data = service.get_operator_data(GetOperatorPayload::default().with_server_id(guild)).await?;
    let organizer_payload = GetOrganizerPayload::default().with_discord_id(discord_user as i64);
    if let Some(organizer) = service.get_organizer(organizer_payload).await? {
        let builders_locked = tournament_builders.read().await;
        if let Some(builder) = builders_locked.get(&message) {
            let builder_locked = builder.read().await;
            let payload = CreateTournamentPayload {
                name: builder_locked.name.clone().unwrap(),
                operator_id: operator_data.id,
                channel_id: builder_locked.reports_channel.unwrap().to_string(),
                register_channel: builder_locked.register_channel.unwrap().to_string(),
                role: builder_locked.role.unwrap().to_string(),
                use_bargains: builder_locked.use_bargains.unwrap(),
                use_bargains_color: builder_locked.use_bargains_color.unwrap(),
                use_foreign_heroes: builder_locked.use_foreign_heroes.unwrap(),
                organizer: organizer.id,
                game_type: builder_locked.game_type.unwrap(),
                mod_type: builder_locked.mod_type.unwrap()
            };
            service.create_tournament(payload).await?;
            interaction
                .create_response(
                    context,
                    CreateInteractionResponse::Message(
                        CreateInteractionResponseMessage::new()
                            .ephemeral(true)
                            .content("Турнир успешно создан. Обязательно синхронизируйте его с Challonge турниром для корректной работы"),
                    ),
                )
                .await?;
            builders::tournament_creation::build_registration_interface(context, builder_locked.register_channel.unwrap()).await?;
            builders::tournament_creation::build_reports_interface(context, builder_locked.reports_channel.unwrap()).await?;
            drop(builder_locked);
            drop(builders_locked);
            let mut builders_writable = tournament_builders.write().await;
            builders_writable.remove(&message);
            drop(builders_writable); // heh x4
        }
    }
    Ok(())
}

pub async fn select_sync_challonge_id(
    context: &Context,
    interaction: &ComponentInteraction,
    tournaments_service: &H5TournamentsService,
    challonge_service: &ChallongeService,
    sync_builders: &tokio::sync::RwLock<HashMap<u64, LocalSyncBuilder>>,
    selected_value: &String,
) -> Result<(), crate::Error> {
    let mut sync_builder_locked = sync_builders.write().await;
    match sync_builder_locked.get_mut(&interaction.message.id.get()) {
        Some(sync_builder) => {
            sync_builder.challonge_id = Some(selected_value.clone());
        }
        _ => {
            sync_builder_locked.insert(
                interaction.message.id.get(),
                LocalSyncBuilder {
                    challonge_id: Some(selected_value.clone()),
                    discord_id: None,
                },
            );
        }
    }
    drop(sync_builder_locked); // heh
    let response_message = builders::tournament_creation::build_sync_interface(
        context,
        interaction,
        tournaments_service,
        challonge_service,
        sync_builders,
    )
    .await?;
    interaction
        .create_response(
            context,
            CreateInteractionResponse::UpdateMessage(response_message),
        )
        .await?;
    Ok(())
}

pub async fn select_sync_discord_id(
    context: &Context,
    interaction: &ComponentInteraction,
    tournaments_service: &H5TournamentsService,
    challonge_service: &ChallongeService,
    sync_builders: &tokio::sync::RwLock<HashMap<u64, LocalSyncBuilder>>,
    selected_value: &String,
) -> Result<(), crate::Error> {
    let mut sync_builder_locked = sync_builders.write().await;
    match sync_builder_locked.get_mut(&interaction.message.id.get()) {
        Some(sync_builder) => {
            sync_builder.discord_id = Some(selected_value.clone());
        }
        _ => {
            sync_builder_locked.insert(
                interaction.message.id.get(),
                LocalSyncBuilder {
                    challonge_id: None,
                    discord_id: Some(selected_value.clone()),
                },
            );
        }
    }
    drop(sync_builder_locked);
    let response_message = builders::tournament_creation::build_sync_interface(
        context,
        interaction,
        tournaments_service,
        challonge_service,
        sync_builders,
    )
    .await?;
    interaction
        .create_response(
            context,
            CreateInteractionResponse::UpdateMessage(response_message),
        )
        .await?;
    Ok(())
}

pub async fn select_tournament_to_manage(
    context: &Context,
    interaction: &ComponentInteraction,
    tournaments_service: &H5TournamentsService,
    challonge_service: &ChallongeService,
    managed_tournaments: &tokio::sync::RwLock<HashMap<u64, Uuid>>,
    selected_value: &String,
) -> Result<(), crate::Error> {
    let message = interaction.message.id.get();
    let mut managed_tournaments_locked = managed_tournaments.write().await;
    match managed_tournaments_locked.get_mut(&message) {
        Some(managed_tournament) => {
            *managed_tournament = Uuid::from_str(&selected_value)?;
        }
        _ => {
            let id = Uuid::from_str(&selected_value)?;
            tracing::info!("Inserting managed tournament: {} = {}", message, id);
            managed_tournaments_locked.insert(message, id);
        }
    }
    drop(managed_tournaments_locked); // heh x2 
    let response_message = builders::tournament_creation::build_manage_interface(
        context,
        interaction,
        tournaments_service,
        challonge_service,
        managed_tournaments,
    )
    .await?;
    interaction
        .create_response(
            context,
            CreateInteractionResponse::UpdateMessage(response_message),
        )
        .await?;
    Ok(())
}

pub async fn start_synchronization(
    context: &Context,
    interaction: &ComponentInteraction,
    service: &H5TournamentsService,
    sync_builders: &tokio::sync::RwLock<HashMap<u64, LocalSyncBuilder>>,
) -> Result<(), crate::Error> {
    let message = &interaction.message.id.get();
    let sync_builders_locked = sync_builders.read().await;
    if let Some(sync_builder) = sync_builders_locked.get(message) {
        if sync_builder.challonge_id.is_none() || sync_builder.discord_id.is_none() {
            interaction.create_response(context, CreateInteractionResponse::Message(
                CreateInteractionResponseMessage::new()
                    .content("Один из параметров не указан, укажите оба турнира, чтобы совершить синхронизацию")
                    .ephemeral(true)
            )).await?;
        } else {
            let challonge_id = sync_builder.challonge_id.as_ref().unwrap();
            let discord_id = sync_builder.discord_id.as_ref().unwrap();
            service
                .update_tournament(
                    UpdateTournamentPayload::new(Uuid::from_str(discord_id)?)
                        .with_challonge_id(challonge_id.clone()),
                )
                .await?;
            interaction
                .create_response(
                    context,
                    CreateInteractionResponse::Message(
                        CreateInteractionResponseMessage::new()
                            .content("Турниры успешно синхронизированы")
                            .ephemeral(true),
                    ),
                )
                .await?;
        }
    }

    Ok(())
}

pub async fn start_participants_syncronization(
    context: &Context,
    interaction: &ComponentInteraction,
    tournaments_service: &H5TournamentsService,
    challonge_service: &ChallongeService,
    managed_tournaments: &tokio::sync::RwLock<HashMap<u64, Uuid>>,
) -> Result<(), crate::Error> {
    let message = interaction.message.id.get();
    let user = interaction.user.id.get();
    if let Some(organizer) = tournaments_service
        .get_organizer(GetOrganizerPayload::default().with_discord_id(user as i64))
        .await?
    {
        tracing::info!("Trying to get managed_tournament for message {}", message);
        let managed_tournaments_locked = managed_tournaments.read().await;
        match managed_tournaments_locked.get(&message) {
            Some(current_managed_tournament) => {
                let tournament_data = tournaments_service
                    .get_tournament_data(
                        GetTournament::default().with_id(*current_managed_tournament),
                    )
                    .await?
                    .unwrap();
                let users_data = tournaments_service
                    .get_tournament_users(*current_managed_tournament)
                    .await?;
                let challonge_participants_data = challonge_service
                    .get_participants(
                        &organizer.challonge,
                        tournament_data.challonge_id.as_ref().unwrap(),
                    )
                    .await?;
                let payload = users_data
                    .iter()
                    .filter_map(|u| {
                        if !challonge_participants_data.iter().any(|p| {
                            p.attributes.misc.is_some()
                                && *p.attributes.misc.as_ref().unwrap() == u.id.to_string()
                        }) {
                            Some(ChallongeParticipantAttributes {
                                name: u.nickname.clone(),
                                seed: Some(1),
                                misc: Some(u.id.to_string()),
                                email: Some(String::new()),
                                username: Some(String::new()),
                            })
                        } else {
                            None
                        }
                    })
                    .collect::<Vec<ChallongeParticipantAttributes>>();

                let add_result_data = challonge_service
                    .participants_bulk_add(
                        &organizer.challonge,
                        tournament_data.challonge_id.as_ref().unwrap().clone(),
                        payload,
                    )
                    .await?;

                let participants_to_update = add_result_data
                    .iter()
                    .map(|p| UpdateParticipant {
                        tournament_id: *current_managed_tournament,
                        user_id: Uuid::from_str(p.attributes.misc.as_ref().unwrap()).unwrap(),
                        challonge_id: p.id.clone(),
                    })
                    .collect::<Vec<UpdateParticipant>>();
                tournaments_service
                    .update_participants_bulk(participants_to_update)
                    .await?;
                interaction.create_response(context, CreateInteractionResponse::Message(
                CreateInteractionResponseMessage::new()
                    .ephemeral(true)
                    .content("Зарегистрированные участники турнира успешно загружены на Challonge.com")
            )).await?;
            }
            _ => {
                tracing::info!(
                    "No tournament found here: {:?}",
                    &managed_tournaments_locked
                );
                interaction
                    .create_response(context, CreateInteractionResponse::Acknowledge)
                    .await?;
            }
        }
    }
    Ok(())
}
