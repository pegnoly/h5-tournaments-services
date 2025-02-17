use std::str::FromStr;

use poise::serenity_prelude::*;
use strum::{Display, EnumString};
use crate::{builders, graphql::queries::update_tournament_builder, services::h5_tournaments::{payloads::{CreateOrganizerPayload, CreateTournamentPayload, GetOrganizerPayload, GetTournamentBuilderPayload, UpdateTournamentBuilderPayload}, service::H5TournamentsService}};

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