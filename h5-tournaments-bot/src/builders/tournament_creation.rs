use std::collections::HashMap;

/// Contains all methods to build discord elements for tournaments creation and administration.

use poise::serenity_prelude::*;
use crate::{event_handler::LocalSyncBuilder, graphql::queries::{get_tournament_builder::{self, GetTournamentBuilderTournamentBuilder}, update_tournament_builder::{self, UpdateTournamentBuilderUpdateTournamentBuilder}}, operations::administration::{BargainsColorUsageType, BargainsUsageType, ForeignHeroesUsageType}, services::{challonge::service::ChallongeService, h5_tournaments::{payloads::{GetOrganizerPayload, GetTournamentBuilderPayload}, service::H5TournamentsService}}};

/// Called when user with organizer permissions requests new tournament creation.
pub async fn build_tournament_creation_interface(
    interaction: &ComponentInteraction,
    context: &Context,
    builder: Option<&UpdateTournamentBuilderUpdateTournamentBuilder>
) -> Result<(), crate::Error> {
    let base_interface = build_base_interface(builder).await;
    let message = CreateInteractionResponseMessage::new()
        .ephemeral(true)
        .content("Создание турнира. Эти данные относятся ТОЛЬКО к взаимодействию с ботом и созданный турнир должен быть синхронизирован с Challonge турниром")
        .components(vec![
            base_interface
        ]);
    
    interaction.create_response(context, CreateInteractionResponse::Message(message)).await?;
    Ok(())
}

/// Called when user with organizer permissions makes any changes with existing tournament builder
pub async fn rebuild_tournament_creation_interface(
    builder: &UpdateTournamentBuilderUpdateTournamentBuilder
) -> CreateInteractionResponseMessage {
    let base_interface = build_base_interface(Some(&builder)).await;
    let current_interface = build_current_state_interface(&builder).await;
    let mut components = vec![base_interface];
    for row in current_interface {
        components.push(row);
    }
    CreateInteractionResponseMessage::new()
        .ephemeral(true)
        .content("Создание турнира. Эти данные относятся ТОЛЬКО к взаимодействию с ботом и созданный турнир должен быть синхронизирован с Challonge турниром")
        .components(components)
}

/// Creates modal for tournament name input
pub async fn build_tournament_name_modal(current_name: Option<String>) -> CreateModal {
    CreateModal::new("tournament_creation_name_modal", "Укажите название турнира")
        .components(vec![
            CreateActionRow::InputText(
                CreateInputText::new(InputTextStyle::Short, "Название турнира", "tournament_creation_name_input")
                    .value(if current_name.is_some() { current_name.unwrap() } else { String::new() } )
            )
        ])
}

/// Builds default buttons for tournament creation interface
async fn build_base_interface(
    builder: Option<&UpdateTournamentBuilderUpdateTournamentBuilder>
) -> CreateActionRow {
    CreateActionRow::Buttons(vec![
        CreateButton::new("setup_tournament_name_button")
            .label("Указать имя турнира")
            .style(ButtonStyle::Success)
            .disabled(builder.is_some()),
        CreateButton::new("setup_tournament_channels_button")
            .label("Указать связанные с турниром каналы")
            .style(ButtonStyle::Secondary)
            .disabled(builder.is_none() || (builder.is_some() && *builder.unwrap().edit_state.as_ref().unwrap() == update_tournament_builder::TournamentEditState::CHANNELS_DATA)),
        CreateButton::new("setup_tournament_reports_button")
            .label("Указать параметры отчетов турнира")
            .style(ButtonStyle::Secondary)
            .disabled(builder.is_none() || (builder.is_some() && *builder.unwrap().edit_state.as_ref().unwrap() == update_tournament_builder::TournamentEditState::REPORTS_DATA)),
        CreateButton::new("submit_tournament_creation_button")
            .label("Зарегистрировать турнир")
            .style(ButtonStyle::Secondary)
            .disabled(
                builder.is_none() || 
                builder.unwrap().register_channel.is_none() || 
                builder.unwrap().reports_channel.is_none() ||
                builder.unwrap().role.is_none() ||
                builder.unwrap().use_bargains.is_none() ||
                builder.unwrap().use_bargains_color.is_none() ||
                builder.unwrap().use_foreign_heroes.is_none()
            )
    ])
}

/// Builds interface for current state of tournament creation
async fn build_current_state_interface(
    builder: &UpdateTournamentBuilderUpdateTournamentBuilder
) -> Vec<CreateActionRow> {
    match &builder.edit_state.as_ref().unwrap() {
        update_tournament_builder::TournamentEditState::CHANNELS_DATA => {
            build_channels_selection_interface(builder).await
        },
        update_tournament_builder::TournamentEditState::REPORTS_DATA => {
            build_reports_data_selection_interface(builder).await
        },
        _=> {vec![]}
    }
}

async fn build_channels_selection_interface(
    builder: &UpdateTournamentBuilderUpdateTournamentBuilder
) -> Vec<CreateActionRow> {
    vec![
        CreateActionRow::SelectMenu(
            CreateSelectMenu::new("registration_channel_selector", 
                CreateSelectMenuKind::Channel {
                    channel_types: Some(vec![ChannelType::Text]), 
                    default_channels: if builder.register_channel.is_some() {
                        Some(vec![ChannelId::from(builder.register_channel.unwrap() as u64)])
                    } else {
                        None
                    }
                }
            )
            .placeholder("Укажите канал для регистрации юзеров")
        ),
        CreateActionRow::SelectMenu(
            CreateSelectMenu::new("reports_channel_selector",
                CreateSelectMenuKind::Channel { 
                    channel_types: Some(vec![ChannelType::Text]), 
                    default_channels: if builder.reports_channel.is_some() {
                        Some(vec![ChannelId::from(builder.reports_channel.unwrap() as u64)])
                    } else {
                        None
                    } 
                }
            )
            .placeholder("Укажите канал для заполнения отчетов")
        ),
        CreateActionRow::SelectMenu(
            CreateSelectMenu::new("tournament_role_selector", 
                CreateSelectMenuKind::Role { 
                    default_roles: if builder.role.is_some() {
                        Some(vec![RoleId::from(builder.role.unwrap() as u64)])
                    } else {
                        None
                    } 
                }
            )
            .placeholder("Укажите роль для участника турнира")
        )
    ]
}

async fn build_reports_data_selection_interface(
    builder: &UpdateTournamentBuilderUpdateTournamentBuilder
) -> Vec<CreateActionRow> {
    vec![
        CreateActionRow::SelectMenu(
            CreateSelectMenu::new("tournament_bargains_usage_selector", 
                CreateSelectMenuKind::String { options: vec![
                    CreateSelectMenuOption::new("Использовать торги", BargainsUsageType::UseBargains.to_string())
                        .default_selection(builder.use_bargains.is_some() && builder.use_bargains.unwrap() == true),
                    CreateSelectMenuOption::new("Не спользовать торги", BargainsUsageType::DontUseBargains.to_string())
                        .default_selection(builder.use_bargains.is_some() && builder.use_bargains.unwrap() == false)
                ]}
            )
            .placeholder("Укажите статус использования торгов")
        ),
        CreateActionRow::SelectMenu(
            CreateSelectMenu::new("tournament_bargains_color_usage_selector",
                CreateSelectMenuKind::String { options: vec![
                    CreateSelectMenuOption::new("Указывать цвет торга", BargainsColorUsageType::UseBargainsColor.to_string())
                        .default_selection(builder.use_bargains_color.is_some() && builder.use_bargains_color.unwrap() == true),
                    CreateSelectMenuOption::new("Не указывать цвет торга", BargainsColorUsageType::DontUseBargainsColor.to_string())
                        .default_selection(builder.use_bargains_color.is_some() && builder.use_bargains_color.unwrap() == false)
                ]}
            )
            .placeholder("Укажите статус использования цвета торгов")
        ),
        CreateActionRow::SelectMenu(
            CreateSelectMenu::new("tournament_foreign_heroes_usage_selector",
                CreateSelectMenuKind::String { options: vec![
                    CreateSelectMenuOption::new("Можно использовать неродных героев", ForeignHeroesUsageType::UseForeignHeroes.to_string())
                        .default_selection(builder.use_foreign_heroes.is_some() && builder.use_foreign_heroes.unwrap() == true),
                    CreateSelectMenuOption::new("Нельзя использовать неродных героев", ForeignHeroesUsageType::DontUseForeignHeroes.to_string())
                        .default_selection(builder.use_foreign_heroes.is_some() && builder.use_foreign_heroes.unwrap() == false)
                ]}
            )
            .placeholder("Укажите статус использования неродных героев")
        )
    ]
}

pub async fn build_sync_interface(
    _context: &Context,
    interaction: &ComponentInteraction,
    tournaments_service: &H5TournamentsService,
    challonge_service: &ChallongeService,
    sync_builders: &tokio::sync::RwLock<HashMap<u64, LocalSyncBuilder>>
) -> Result<CreateInteractionResponseMessage, crate::Error> {
    let payload = GetOrganizerPayload::default().with_discord_id(interaction.user.id.get() as i64);
    if let Some(existing_organizer) = tournaments_service.get_organizer(payload).await? {
        let api_key = existing_organizer.challonge;
        let challonge_tournaments = challonge_service.get_tournaments(api_key).await?;
        let discord_tournaments = tournaments_service.get_all_tournaments(existing_organizer.id).await?;
        let sync_builders_locked = sync_builders.read().await;
        let sync_builder = sync_builders_locked.get(&interaction.message.id.get());
        Ok(CreateInteractionResponseMessage::new()
            .embed(
                CreateEmbed::new()
                    .title("Синхронизация турниров")
                    .description("Позволяет установить связь между турнирами, созданными ботом и Challonge.com турнирами, что разрешит боту делать запросы к турниру на Challonge.")
            )
            .select_menu(CreateSelectMenu::new("challonge_tournaments_selector", CreateSelectMenuKind::String { options: Vec::from_iter(
                challonge_tournaments.iter().map(|t| {
                    CreateSelectMenuOption::new(t.attributes.name.clone(), t.id.clone())
                        .default_selection(
                            sync_builder.is_some() &&  
                            sync_builder.unwrap().challonge_id.is_some() && 
                            *sync_builder.unwrap().challonge_id.as_ref().unwrap() == t.id)
                })
            ) }).placeholder("Укажите имя турнира на Challonge.com"))
            .select_menu(CreateSelectMenu::new("discord_tournaments_selector", CreateSelectMenuKind::String { options: Vec::from_iter(
                discord_tournaments.iter().map(|t| {
                    CreateSelectMenuOption::new(t.name.clone(), t.id)
                        .default_selection(
                            sync_builder.is_some() &&  
                            sync_builder.unwrap().discord_id.is_some() && 
                            *sync_builder.unwrap().discord_id.as_ref().unwrap() == t.id.to_string())
                })
            ) }).placeholder("Укажите имя турнира, созданного через бота"))
            .button(CreateButton::new("sync_tournaments_button").label("Синхронизировать выбранные турниры").style(ButtonStyle::Success))
            .ephemeral(true))
    } else {
        Ok(CreateInteractionResponseMessage::new()
            .ephemeral(true)
            .content("Вы не являетесь организатором турниров и не можете использовать данную систему."))
    }
}