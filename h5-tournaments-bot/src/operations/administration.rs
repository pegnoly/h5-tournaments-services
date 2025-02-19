use std::{collections::HashMap, str::FromStr};

use poise::serenity_prelude::*;
use strum::{Display, EnumString};
use uuid::Uuid;
use crate::{builders, event_handler::LocalSyncBuilder, graphql::queries::{update_participants_bulk::UpdateParticipant, update_tournament_builder}, services::{challonge::{payloads::{ChallongeParticipantAttributes, ChallongeParticipantPayload}, service::ChallongeService}, h5_tournaments::{payloads::{CreateOrganizerPayload, CreateTournamentPayload, GetOrganizerPayload, GetTournamentBuilderPayload, UpdateTournamentBuilderPayload, UpdateTournamentPayload}, service::H5TournamentsService}}, types::payloads::GetTournament};

pub async fn start_admin_registration(
    context: &Context,
    interaction: &ComponentInteraction,
    service: &H5TournamentsService
) -> Result<(), crate::Error> {
    if let Some(_existing_organizer) = 
        service.get_organizer(GetOrganizerPayload::default().with_discord_id(interaction.user.id.get() as i64)).await? {
        
        interaction.create_response(context, CreateInteractionResponse::Message(
            CreateInteractionResponseMessage::new()
                .ephemeral(true)
                .content("Вы уже зарегистрированы как организатор турниров")
        )).await?;
    } else {
        interaction.create_response(context, CreateInteractionResponse::Modal(
            CreateModal::new("tournament_admin_challonge_key_modal", "Укажите свой Challonge.com ключ API")
                .components(vec![
                    CreateActionRow::InputText(
                        CreateInputText::new(InputTextStyle::Short, "Введите свой ключ API", "tournament_admin_challonge_key_input")
                    )
                ])
        )).await?;
    }
    Ok(())
}

pub async fn process_admin_registration_modal(
    context: &Context,
    interaction: &ModalInteraction,
    service: &H5TournamentsService
) -> Result<(), crate::Error> {
    let user = interaction.user.id.get();
    for row in &interaction.data.components {
        for component in &row.components {
            match component {
                ActionRowComponent::InputText(text) => {
                    if text.custom_id.as_str() == "tournament_admin_challonge_key_input" {
                        service.create_organizer(CreateOrganizerPayload::new(user, text.value.as_ref().unwrap().to_owned())).await?;
                        break;
                    }
                },
                _=> {}
            }
        }
    }
    interaction.create_response(context, CreateInteractionResponse::Message(
        CreateInteractionResponseMessage::new()
            .ephemeral(true)
            .content("Вы успешно зарегистрированы, как организатор турниров.")
    )).await?;
    Ok(())
}

pub async fn start_tournament_name_creation(
    context: &Context,
    interaction: &ComponentInteraction,
    service: &H5TournamentsService
) -> Result<(), crate::Error> {
    let message = interaction.message.id.get();
    let payload = GetTournamentBuilderPayload::default().with_message(message);
    if let Some(existing_builder) = service.get_tournament_builder(payload).await? {
        interaction.create_response(context, CreateInteractionResponse::Modal(
            builders::tournament_creation::build_tournament_name_modal(existing_builder.name).await
        )).await?;
    } else {
        let _new_builder = service.create_tournament_builder(message).await?;
        interaction.create_response(context, CreateInteractionResponse::Modal(
            builders::tournament_creation::build_tournament_name_modal(None).await
        )).await?;
    }

    Ok(())
}

pub async fn process_tournament_name_creation_modal(
    context: &Context,
    interaction: &ModalInteraction,
    service: &H5TournamentsService
) -> Result<(), crate::Error> {
    let message = interaction.message.as_ref().unwrap().id.get();
    for row in &interaction.data.components {
        for component in &row.components {
            match component {
                ActionRowComponent::InputText(text) => {
                    if text.custom_id.as_str() == "tournament_creation_name_input" {
                        let get_payload = GetTournamentBuilderPayload::default().with_message(message);
                        let builder = service.get_tournament_builder(get_payload).await?.unwrap();
                        let update_payload = UpdateTournamentBuilderPayload::new(builder.id).with_name(text.value.as_ref().unwrap().to_owned());
                        let updated_builder = service.update_tournament_builder(update_payload).await?;
                        let response_message = builders::tournament_creation::rebuild_tournament_creation_interface(&updated_builder).await;
                        interaction.create_response(context, CreateInteractionResponse::UpdateMessage(response_message)).await?;
                        return Ok(());
                    }
                },
                _=> {}
            }
        }
    }
    Ok(()) 
}

pub async fn process_tournament_builder_state_change(
    context: &Context,
    interaction: &ComponentInteraction,
    service: &H5TournamentsService,
    new_state: update_tournament_builder::TournamentEditState
) -> Result<(), crate::Error> {
    let get_payload = GetTournamentBuilderPayload::default().with_message(interaction.message.id.get());
    if let Some(existing_builder) = service.get_tournament_builder(get_payload).await? {
        let update_payload = UpdateTournamentBuilderPayload::new(existing_builder.id).with_edit_state(new_state);
        let updated_builder = service.update_tournament_builder(update_payload).await?;
        let response_message = builders::tournament_creation::rebuild_tournament_creation_interface(&updated_builder).await;
        interaction.create_response(context, CreateInteractionResponse::UpdateMessage(response_message)).await?;
        Ok(())
    } else {
        Err(crate::Error::from("Failed to find tournament builder to update its state"))
    }
}

pub async fn select_tournament_builder_register_channel(
    context: &Context,
    interaction: &ComponentInteraction,
    service: &H5TournamentsService,
    selected_value: u64
) -> Result<(), crate::Error> {
    let get_payload = GetTournamentBuilderPayload::default().with_message(interaction.message.id.get());
    if let Some(existing_builder) = service.get_tournament_builder(get_payload).await? {
        let update_payload = UpdateTournamentBuilderPayload::new(existing_builder.id)
            .with_register_channel(selected_value)
            .with_edit_state(existing_builder.edit_state.unwrap().into());
        let updated_builder = service.update_tournament_builder(update_payload).await?;
        let response_message = builders::tournament_creation::rebuild_tournament_creation_interface(&updated_builder).await;
        interaction.create_response(context, CreateInteractionResponse::UpdateMessage(response_message)).await?;
        Ok(())
    } else {
        Err(crate::Error::from("Failed to find tournament builder to update its register channel"))
    }
}

pub async fn select_tournament_builder_reports_channel(
    context: &Context,
    interaction: &ComponentInteraction,
    service: &H5TournamentsService,
    selected_value: u64
) -> Result<(), crate::Error> {
    let get_payload = GetTournamentBuilderPayload::default().with_message(interaction.message.id.get());
    if let Some(existing_builder) = service.get_tournament_builder(get_payload).await? {
        let update_payload = UpdateTournamentBuilderPayload::new(existing_builder.id)
            .with_reports_channel(selected_value)
            .with_edit_state(existing_builder.edit_state.unwrap().into());
        let updated_builder = service.update_tournament_builder(update_payload).await?;
        let response_message = builders::tournament_creation::rebuild_tournament_creation_interface(&updated_builder).await;
        interaction.create_response(context, CreateInteractionResponse::UpdateMessage(response_message)).await?;
        Ok(())
    } else {
        Err(crate::Error::from("Failed to find tournament builder to update its reports channel"))
    }
}

pub async fn select_tournament_builder_role(
    context: &Context,
    interaction: &ComponentInteraction,
    service: &H5TournamentsService,
    selected_value: u64
) -> Result<(), crate::Error> {
    let get_payload = GetTournamentBuilderPayload::default().with_message(interaction.message.id.get());
    if let Some(existing_builder) = service.get_tournament_builder(get_payload).await? {
        let update_payload = UpdateTournamentBuilderPayload::new(existing_builder.id)
            .with_role(selected_value)
            .with_edit_state(existing_builder.edit_state.unwrap().into());
        let updated_builder = service.update_tournament_builder(update_payload).await?;
        let response_message = builders::tournament_creation::rebuild_tournament_creation_interface(&updated_builder).await;
        interaction.create_response(context, CreateInteractionResponse::UpdateMessage(response_message)).await?;
        Ok(())
    } else {
        Err(crate::Error::from("Failed to find tournament builder to update its role"))
    }
}

#[derive(Debug, EnumString, Display)]
pub enum BargainsUsageType {
    UseBargains,
    DontUseBargains
}

impl Into<bool> for BargainsUsageType {
    fn into(self) -> bool {
        match self {
            BargainsUsageType::UseBargains => true,
            BargainsUsageType::DontUseBargains => false
        }
    }
}

pub async fn select_tournament_builder_bargains_usage(
    context: &Context,
    interaction: &ComponentInteraction,
    service: &H5TournamentsService,
    selected_value: &String
) -> Result<(), crate::Error> {
    let get_payload = GetTournamentBuilderPayload::default().with_message(interaction.message.id.get());
    if let Some(existing_builder) = service.get_tournament_builder(get_payload).await? {
        let update_payload = UpdateTournamentBuilderPayload::new(existing_builder.id)
            .with_bargains(BargainsUsageType::from_str(selected_value)?.into())
            .with_edit_state(existing_builder.edit_state.unwrap().into());
        let updated_builder = service.update_tournament_builder(update_payload).await?;
        let response_message = builders::tournament_creation::rebuild_tournament_creation_interface(&updated_builder).await;
        interaction.create_response(context, CreateInteractionResponse::UpdateMessage(response_message)).await?;
        Ok(())
    } else {
        Err(crate::Error::from("Failed to find tournament builder to update its role"))
    }
}

#[derive(Debug, EnumString, Display)]
pub enum BargainsColorUsageType {
    UseBargainsColor,
    DontUseBargainsColor
}

impl Into<bool> for BargainsColorUsageType {
    fn into(self) -> bool {
        match self {
            BargainsColorUsageType::UseBargainsColor => true,
            BargainsColorUsageType::DontUseBargainsColor => false
        }
    }
}

pub async fn select_tournament_builder_bargains_color_usage(
    context: &Context,
    interaction: &ComponentInteraction,
    service: &H5TournamentsService,
    selected_value: &String
) -> Result<(), crate::Error> {
    let get_payload = GetTournamentBuilderPayload::default().with_message(interaction.message.id.get());
    if let Some(existing_builder) = service.get_tournament_builder(get_payload).await? {
        let update_payload = UpdateTournamentBuilderPayload::new(existing_builder.id)
            .with_bargains_color(BargainsColorUsageType::from_str(selected_value)?.into())
            .with_edit_state(existing_builder.edit_state.unwrap().into());
        let updated_builder = service.update_tournament_builder(update_payload).await?;
        let response_message = builders::tournament_creation::rebuild_tournament_creation_interface(&updated_builder).await;
        interaction.create_response(context, CreateInteractionResponse::UpdateMessage(response_message)).await?;
        Ok(())
    } else {
        Err(crate::Error::from("Failed to find tournament builder to update its role"))
    }
}

#[derive(Debug, EnumString, Display)]
pub enum ForeignHeroesUsageType {
    UseForeignHeroes,
    DontUseForeignHeroes
}

impl Into<bool> for ForeignHeroesUsageType {
    fn into(self) -> bool {
        match self {
            ForeignHeroesUsageType::UseForeignHeroes => true,
            ForeignHeroesUsageType::DontUseForeignHeroes => false
        }
    }
}

pub async fn select_tournament_builder_foreign_heroes_usage(
    context: &Context,
    interaction: &ComponentInteraction,
    service: &H5TournamentsService,
    selected_value: &String
) -> Result<(), crate::Error> {
    let get_payload = GetTournamentBuilderPayload::default().with_message(interaction.message.id.get());
    if let Some(existing_builder) = service.get_tournament_builder(get_payload).await? {
        let update_payload = UpdateTournamentBuilderPayload::new(existing_builder.id)
            .with_foreign_heroes(ForeignHeroesUsageType::from_str(selected_value)?.into())
            .with_edit_state(existing_builder.edit_state.unwrap().into());
        let updated_builder = service.update_tournament_builder(update_payload).await?;
        let response_message = builders::tournament_creation::rebuild_tournament_creation_interface(&updated_builder).await;
        interaction.create_response(context, CreateInteractionResponse::UpdateMessage(response_message)).await?;
        Ok(())
    } else {
        Err(crate::Error::from("Failed to find tournament builder to update its role"))
    }
}

pub async fn finalize_tournament_creation(
    context: &Context,
    interaction: &ComponentInteraction,
    service: &H5TournamentsService
) -> Result<(), crate::Error> {
    if let Some(organizer) = service.get_organizer(GetOrganizerPayload::default().with_discord_id(interaction.user.id.get() as i64)).await? {
        let builder = service.get_tournament_builder(
            GetTournamentBuilderPayload::default()
            .with_message(interaction.message.id.get())
        ).await?.unwrap();
        let payload = CreateTournamentPayload {
            name: builder.name.unwrap(),
            operator_id: None,
            channel_id: builder.reports_channel.unwrap().to_string(),
            register_channel: builder.register_channel.unwrap().to_string(),
            role: builder.role.unwrap().to_string(),
            use_bargains: builder.use_bargains.unwrap(),
            use_bargains_color: builder.use_bargains_color.unwrap(),
            use_foreign_heroes: builder.use_foreign_heroes.unwrap(),
            organizer: organizer.id
        };
        service.create_tournament(payload).await?;
        interaction.create_response(context, CreateInteractionResponse::Message(
            CreateInteractionResponseMessage::new()
                .ephemeral(true)
                .content("Турнир успешно создан.")
        )).await?;
    }
    Ok(())
}

pub async fn select_sync_challonge_id(
    context: &Context,
    interaction: &ComponentInteraction,
    tournaments_service: &H5TournamentsService,
    challonge_service: &ChallongeService,
    sync_builders: &tokio::sync::RwLock<HashMap<u64, LocalSyncBuilder>>,
    selected_value: &String
) -> Result<(), crate::Error> {
    let mut sync_builder_locked = sync_builders.write().await;
    if let Some(sync_builder) = sync_builder_locked.get_mut(&interaction.message.id.get()) {
        sync_builder.challonge_id = Some(selected_value.clone());
    } else {
        sync_builder_locked.insert(interaction.message.id.get(), LocalSyncBuilder { challonge_id: Some(selected_value.clone()), discord_id: None });
    }
    drop(sync_builder_locked); // heh
    let response_message = builders::tournament_creation::build_sync_interface(
        context, interaction, tournaments_service, challonge_service, sync_builders).await?;
    interaction.create_response(context, CreateInteractionResponse::UpdateMessage(response_message)).await?;
    Ok(())
}

pub async fn select_sync_discord_id(
    context: &Context,
    interaction: &ComponentInteraction,
    tournaments_service: &H5TournamentsService,
    challonge_service: &ChallongeService,
    sync_builders: &tokio::sync::RwLock<HashMap<u64, LocalSyncBuilder>>,
    selected_value: &String
) -> Result<(), crate::Error> {
    let mut sync_builder_locked = sync_builders.write().await;
    if let Some(sync_builder) = sync_builder_locked.get_mut(&interaction.message.id.get()) {
        sync_builder.discord_id = Some(selected_value.clone());
    } else {
        sync_builder_locked.insert(interaction.message.id.get(), LocalSyncBuilder { challonge_id: None, discord_id: Some(selected_value.clone()) });
    }
    drop(sync_builder_locked);
    let response_message = builders::tournament_creation::build_sync_interface(
        context, interaction, tournaments_service, challonge_service, sync_builders).await?;
    interaction.create_response(context, CreateInteractionResponse::UpdateMessage(response_message)).await?;
    Ok(())
}

pub async fn select_tournament_to_manage(
    context: &Context,
    interaction: &ComponentInteraction,
    tournaments_service: &H5TournamentsService,
    challonge_service: &ChallongeService,
    managed_tournaments: &tokio::sync::RwLock<HashMap<u64, Uuid>>,
    selected_value: &String
) -> Result<(), crate::Error> {
    let message = interaction.message.id.get();
    let mut managed_tournaments_locked = managed_tournaments.write().await;
    if let Some(managed_tournament) = managed_tournaments_locked.get_mut(&message) {
        *managed_tournament = Uuid::from_str(&selected_value)?;
    } else {
        let id = Uuid::from_str(&selected_value)?;
        tracing::info!("Inserting managed tournament: {} = {}", message, id);
        managed_tournaments_locked.insert(message, id);
    }
    drop(managed_tournaments_locked); // heh x2 
    let response_message = builders::tournament_creation::build_manage_interface(
        context, interaction, tournaments_service, challonge_service, managed_tournaments).await?;
    interaction.create_response(context, CreateInteractionResponse::UpdateMessage(response_message)).await?;
    Ok(())        
}


pub async fn start_synchronization(
    context: &Context,
    interaction: &ComponentInteraction,
    service: &H5TournamentsService,
    sync_builders: &tokio::sync::RwLock<HashMap<u64, LocalSyncBuilder>>
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
        }
        else {
            let challonge_id = sync_builder.challonge_id.as_ref().unwrap();
            let discord_id = sync_builder.discord_id.as_ref().unwrap();
            service.update_tournament(UpdateTournamentPayload::new(Uuid::from_str(discord_id)?).with_challonge_id(challonge_id.clone())).await?;
            interaction.create_response(context, CreateInteractionResponse::Message(
                CreateInteractionResponseMessage::new()
                    .content("Турниры успешно синхронизированы")
                    .ephemeral(true)
            )).await?;
        }
    }

    Ok(())
}

pub async fn start_participants_syncronization(
    context: &Context,
    interaction: &ComponentInteraction,
    tournaments_service: &H5TournamentsService,
    challonge_service: &ChallongeService,
    managed_tournaments: &tokio::sync::RwLock<HashMap<u64, Uuid>>
) -> Result<(), crate::Error> {
    let message = interaction.message.id.get();
    let user = interaction.user.id.get();
    if let Some(organizer) = tournaments_service.get_organizer(GetOrganizerPayload::default().with_discord_id(user as i64)).await? {
        tracing::info!("Trying to get managed_tournament for message {}", message);
        let managed_tournaments_locked = managed_tournaments.read().await;
        if let Some(current_managed_tournament) = managed_tournaments_locked.get(&message) {
            let tournament_data = tournaments_service.get_tournament_data(GetTournament::default().with_id(*current_managed_tournament)).await?.unwrap();
            let users_data = tournaments_service.get_tournament_users(*current_managed_tournament).await?;
            let challonge_participants_data = challonge_service.get_participants(
                organizer.challonge.clone(), 
                tournament_data.challonge_id.as_ref().unwrap().clone()).await?;
            let payload = users_data.iter()
                .filter_map(|u| {
                    if !challonge_participants_data.iter().any(|p| p.attributes.misc.is_some() && *p.attributes.misc.as_ref().unwrap() == u.id.to_string()) {
                        Some(ChallongeParticipantAttributes {
                            name: u.nickname.clone(),
                            seed: Some(1),
                            misc: Some(u.id.to_string()),
                            email: Some(String::new()),
                            username: Some(String::new())
                        })
                    } else {
                        None
                    }
                })
                .collect::<Vec<ChallongeParticipantAttributes>>();
    
            let add_result_data = challonge_service.participants_bulk_add(
                organizer.challonge, 
                tournament_data.challonge_id.as_ref().unwrap().clone(), 
                payload
            ).await?;

            let participants_to_update = add_result_data.iter()
                .map(|p| {
                    UpdateParticipant {
                        tournament_id: *current_managed_tournament,
                        user_id: Uuid::from_str(p.attributes.misc.as_ref().unwrap()).unwrap(),
                        challonge_id: p.id.clone()
                    }
                })
                .collect::<Vec<UpdateParticipant>>();
            tournaments_service.update_participants_bulk(participants_to_update).await?;
            interaction.create_response(context, CreateInteractionResponse::Message(
                CreateInteractionResponseMessage::new()
                    .ephemeral(true)
                    .content("Зарегистрированные участники турнира успешно загружены на Challonge.com")
            )).await?;
        } else {
            tracing::info!("No tournament found here: {:?}", &managed_tournaments_locked);
            interaction.create_response(context, CreateInteractionResponse::Acknowledge).await?;
        }
    }
    Ok(())
}