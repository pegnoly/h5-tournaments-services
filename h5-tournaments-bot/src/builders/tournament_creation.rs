use std::{collections::HashMap, str::FromStr};

use crate::{
    builders::types::GameType, event_handler::LocalSyncBuilder, graphql::queries::{
        get_organizer::GetOrganizerOrganizer,
        get_tournaments::GetTournamentsTournaments,
    }, operations::administration::{
        BargainsColorUsageType, BargainsUsageType, ForeignHeroesUsageType,
    }, services::{
        challonge::service::ChallongeService,
        h5_tournaments::{
            payloads::GetOrganizerPayload,
            service::H5TournamentsService,
        },
    }, types::payloads::GetTournament
};
use h5_tournaments_api::prelude::ModType;
/// Contains all methods to build discord elements for tournaments creation and administration.
use poise::serenity_prelude::*;
use strum::IntoEnumIterator;
use tokio::sync::{RwLock, RwLockReadGuard};
use uuid::Uuid;

use super::types::{TournamentBuildState, TournamentBuilder};

/// Called when user with organizer permissions requests new tournament creation.
pub async fn build_tournament_creation_interface(
    interaction: &ComponentInteraction,
    context: &Context,
    tournament_builders: &RwLock<HashMap<u64, RwLock<TournamentBuilder>>>
) -> Result<(), crate::Error> {
    let builder = RwLock::new(TournamentBuilder::default());
    let builder_locked = builder.read().await;
    let base_interface = build_base_interface(&builder_locked).await;
    let current_interface = build_current_state_interface(&builder_locked).await;
    let mut components = vec![base_interface];
    for row in current_interface {
        components.push(row);
    }
    drop(builder_locked);
    let message = CreateInteractionResponseMessage::new()
        .ephemeral(true)
        .content("**Создание турнира. Эти данные относятся ТОЛЬКО к взаимодействию с ботом и созданный турнир должен быть синхронизирован с Challonge турниром**")
        .components(components);

    interaction
        .create_response(context, CreateInteractionResponse::Message(message))
        .await?;
    let message_new = interaction.get_response(context).await?;
    let mut builders_locked = tournament_builders.write().await;
    builders_locked.insert(message_new.id.get(), builder);
    drop(builders_locked);
    Ok(())
}

/// Called when user with organizer permissions makes any changes with existing tournament builder
pub async fn rebuild_tournament_creation_interface(
    builder: &RwLockReadGuard<'_, TournamentBuilder>,
) -> CreateInteractionResponseMessage {
    let base_interface = build_base_interface(builder).await;
    let current_interface = build_current_state_interface(builder).await;
    let mut components = vec![base_interface];
    for row in current_interface {
        components.push(row);
    }
    CreateInteractionResponseMessage::new()
        .ephemeral(true)
        .content("**Создание турнира. Эти данные относятся ТОЛЬКО к взаимодействию с ботом и созданный турнир должен быть синхронизирован с Challonge турниром**")
        .components(components)
}

/// Creates modal for tournament name input
pub async fn build_tournament_name_modal(current_name: Option<String>) -> CreateModal {
    CreateModal::new("tournament_creation_name_modal", "Укажите название турнира").components(vec![
        CreateActionRow::InputText(
            CreateInputText::new(
                InputTextStyle::Short,
                "Название турнира",
                "tournament_creation_name_input",
            )
            .value(if current_name.is_some() {
                current_name.unwrap()
            } else {
                String::new()
            }),
        ),
    ])
}

/// Builds default buttons for tournament creation interface
async fn build_base_interface(
    builder: &RwLockReadGuard<'_, TournamentBuilder>
) -> CreateActionRow {
    CreateActionRow::Buttons(vec![
        CreateButton::new("setup_tournament_base_data_button")
            .label("Указать базовые данные турнира")
            .style(if builder.edit_state == TournamentBuildState::BaseData {
                ButtonStyle::Success
            } else {
                ButtonStyle::Secondary
            })
            .disabled(builder.edit_state == TournamentBuildState::BaseData),
        CreateButton::new("setup_tournament_channels_button")
            .label("Указать связанные с турниром каналы")
            .style(if builder.edit_state == TournamentBuildState::ChannelsData {
                ButtonStyle::Success
            } else {
                ButtonStyle::Secondary
            })
            .disabled(builder.edit_state == TournamentBuildState::ChannelsData),
        CreateButton::new("setup_tournament_reports_button")
            .label("Указать параметры отчетов турнира")
            .style(if builder.edit_state == TournamentBuildState::ReportsData {
                ButtonStyle::Success
            } else {
                ButtonStyle::Secondary
            })
            .disabled(builder.edit_state == TournamentBuildState::ReportsData),
        CreateButton::new("submit_tournament_creation_button")
            .label("Зарегистрировать турнир")
            .style(ButtonStyle::Secondary)
            .disabled(
                builder.name.is_none() ||
                builder.register_channel.is_none() || 
                builder.reports_channel.is_none() || 
                builder.role.is_none() || 
                builder.use_bargains.is_none() || 
                builder.use_bargains_color.is_none() || 
                builder.use_foreign_heroes.is_none() ||
                builder.game_type.is_none() ||
                builder.mod_type.is_none()
            ),
    ])
}

/// Builds interface for current state of tournament creation
async fn build_current_state_interface(
    builder: &RwLockReadGuard<'_, TournamentBuilder>,
) -> Vec<CreateActionRow> {
    match &builder.edit_state {
        TournamentBuildState::ChannelsData => {
            build_channels_selection_interface(builder).await
        }
        TournamentBuildState::ReportsData => {
            build_reports_data_selection_interface(builder).await
        }
        TournamentBuildState::BaseData => {
            build_base_data_interface(builder).await
        }
    }
}

async fn build_base_data_interface(
    builder: &RwLockReadGuard<'_, TournamentBuilder>
) -> Vec<CreateActionRow> {
    vec![
        CreateActionRow::Buttons(vec![
            CreateButton::new("enter_tournament_name_button").style(ButtonStyle::Primary).label("Указать название турнира")
        ]),
        CreateActionRow::SelectMenu(CreateSelectMenu::new("tournament_mod_type_selector", CreateSelectMenuKind::String { options: Vec::from_iter(
            ModType::iter().map(|m| {
                CreateSelectMenuOption::new(m.to_string(), m.to_string())
                    .default_selection(builder.mod_type.is_some() && *builder.mod_type.as_ref().unwrap() == m)
            })
        )}).placeholder("Укажите мод, на основе которого проводится турнир")),
        CreateActionRow::SelectMenu(CreateSelectMenu::new("tournament_game_type_selector", CreateSelectMenuKind::String { options: vec![
            CreateSelectMenuOption::new("Турнир по RMG режиму", GameType::Rmg.to_string())
                .default_selection(builder.game_type.is_some() && *builder.game_type.as_ref().unwrap() == GameType::Rmg),
            CreateSelectMenuOption::new("Турнир по симулятору финалок", GameType::Arena.to_string())
                .default_selection(builder.game_type.is_some() && *builder.game_type.as_ref().unwrap() == GameType::Arena)
        ] }).placeholder("Укажите тип игр в турнире")),
    ]
}

async fn build_channels_selection_interface(
    builder: &RwLockReadGuard<'_, TournamentBuilder>,
) -> Vec<CreateActionRow> {
    vec![
        CreateActionRow::SelectMenu(
            CreateSelectMenu::new(
                "registration_channel_selector",
                CreateSelectMenuKind::Channel {
                    channel_types: Some(vec![ChannelType::Text]),
                    default_channels: if builder.register_channel.is_some() {
                        Some(vec![ChannelId::from(
                            builder.register_channel.unwrap() as u64
                        )])
                    } else {
                        None
                    },
                },
            )
            .placeholder("Укажите канал для регистрации юзеров"),
        ),
        CreateActionRow::SelectMenu(
            CreateSelectMenu::new(
                "reports_channel_selector",
                CreateSelectMenuKind::Channel {
                    channel_types: Some(vec![ChannelType::Text]),
                    default_channels: if builder.reports_channel.is_some() {
                        Some(vec![ChannelId::from(
                            builder.reports_channel.unwrap() as u64
                        )])
                    } else {
                        None
                    },
                },
            )
            .placeholder("Укажите канал для заполнения отчетов"),
        ),
        CreateActionRow::SelectMenu(
            CreateSelectMenu::new(
                "tournament_role_selector",
                CreateSelectMenuKind::Role {
                    default_roles: if builder.role.is_some() {
                        Some(vec![RoleId::from(builder.role.unwrap() as u64)])
                    } else {
                        None
                    },
                },
            )
            .placeholder("Укажите роль для участника турнира"),
        ),
    ]
}

async fn build_reports_data_selection_interface(
    builder: &RwLockReadGuard<'_, TournamentBuilder>,
) -> Vec<CreateActionRow> {
    vec![
        CreateActionRow::SelectMenu(
            CreateSelectMenu::new(
                "tournament_bargains_usage_selector",
                CreateSelectMenuKind::String {
                    options: vec![
                        CreateSelectMenuOption::new(
                            "Использовать торги",
                            BargainsUsageType::UseBargains.to_string(),
                        )
                        .default_selection(
                            builder.use_bargains.is_some() && builder.use_bargains.unwrap() == true,
                        ),
                        CreateSelectMenuOption::new(
                            "Не спользовать торги",
                            BargainsUsageType::DontUseBargains.to_string(),
                        )
                        .default_selection(
                            builder.use_bargains.is_some()
                                && builder.use_bargains.unwrap() == false,
                        ),
                    ],
                },
            )
            .placeholder("Укажите статус использования торгов"),
        ),
        CreateActionRow::SelectMenu(
            CreateSelectMenu::new(
                "tournament_bargains_color_usage_selector",
                CreateSelectMenuKind::String {
                    options: vec![
                        CreateSelectMenuOption::new(
                            "Указывать цвет торга",
                            BargainsColorUsageType::UseBargainsColor.to_string(),
                        )
                        .default_selection(
                            builder.use_bargains_color.is_some()
                                && builder.use_bargains_color.unwrap() == true,
                        ),
                        CreateSelectMenuOption::new(
                            "Не указывать цвет торга",
                            BargainsColorUsageType::DontUseBargainsColor.to_string(),
                        )
                        .default_selection(
                            builder.use_bargains_color.is_some()
                                && builder.use_bargains_color.unwrap() == false,
                        ),
                    ],
                },
            )
            .placeholder("Укажите статус использования цвета торгов"),
        ),
        CreateActionRow::SelectMenu(
            CreateSelectMenu::new(
                "tournament_foreign_heroes_usage_selector",
                CreateSelectMenuKind::String {
                    options: vec![
                        CreateSelectMenuOption::new(
                            "Можно использовать неродных героев",
                            ForeignHeroesUsageType::UseForeignHeroes.to_string(),
                        )
                        .default_selection(
                            builder.use_foreign_heroes.is_some()
                                && builder.use_foreign_heroes.unwrap() == true,
                        ),
                        CreateSelectMenuOption::new(
                            "Нельзя использовать неродных героев",
                            ForeignHeroesUsageType::DontUseForeignHeroes.to_string(),
                        )
                        .default_selection(
                            builder.use_foreign_heroes.is_some()
                                && builder.use_foreign_heroes.unwrap() == false,
                        ),
                    ],
                },
            )
            .placeholder("Укажите статус использования неродных героев"),
        ),
    ]
}

pub async fn build_registration_interface(
    context: &Context,
    channel_id: u64
) -> Result<(), crate::Error> {
    
    let register_message =
    CreateMessage::new().components(vec![CreateActionRow::Buttons(vec![
        CreateButton::new("register_user_button")
            .label("Зарегистрироваться в турнире")
            .style(ButtonStyle::Success),
        CreateButton::new("unregister_user_button")
            .label("Отменить регистрацию")
            .style(ButtonStyle::Danger),
        CreateButton::new("update_user_data_button")
            .label("Редактировать данные")
            .style(ButtonStyle::Secondary),
    ])]);
    
    let register_channel = ChannelId::from(channel_id);
    register_channel
        .send_message(context, register_message)
        .await?;
    Ok(())
}

pub async fn build_reports_interface(
    context: &Context,
    channel_id: u64
) -> Result<(), crate::Error> {
    let reports_message = CreateMessage::new().button(
        CreateButton::new("create_report_button")
            .label("Написать отчет")
            .disabled(false),
    );
    let reports_channel = ChannelId::from(channel_id);
    reports_channel
        .send_message(context, reports_message)
        .await?;
    Ok(())
}

pub async fn build_sync_interface(
    _context: &Context,
    interaction: &ComponentInteraction,
    tournaments_service: &H5TournamentsService,
    challonge_service: &ChallongeService,
    sync_builders: &tokio::sync::RwLock<HashMap<u64, LocalSyncBuilder>>,
) -> Result<CreateInteractionResponseMessage, crate::Error> {
    let payload = GetOrganizerPayload::default().with_discord_id(interaction.user.id.get() as i64);
    match tournaments_service.get_organizer(payload).await? {
        Some(existing_organizer) => {
            let api_key = existing_organizer.challonge;
            let challonge_tournaments = challonge_service.get_tournaments(&api_key).await?;
            let discord_tournaments = tournaments_service
                .get_all_tournaments(existing_organizer.id)
                .await?;
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
        }
        _ => Ok(CreateInteractionResponseMessage::new()
            .ephemeral(true)
            .content(
                "Вы не являетесь организатором турниров и не можете использовать данную систему.",
            )),
    }
}

pub async fn build_manage_interface(
    _context: &Context,
    interaction: &ComponentInteraction,
    tournaments_service: &H5TournamentsService,
    challonge_service: &ChallongeService,
    managed_tournaments: &tokio::sync::RwLock<HashMap<u64, Uuid>>,
) -> Result<CreateInteractionResponseMessage, crate::Error> {
    match tournaments_service
        .get_organizer(
            GetOrganizerPayload::default().with_discord_id(interaction.user.id.get() as i64),
        )
        .await?
    {
        Some(organizer) => {
            let managed_tournaments_locked = managed_tournaments.read().await;
            let current_managed_tournament =
                managed_tournaments_locked.get(&interaction.message.id.get());
            let tournaments = tournaments_service
                .get_all_tournaments(organizer.id)
                .await?;
            let sync_tournaments = tournaments
                .iter()
                .filter(|t| t.challonge_id.is_some())
                .collect::<Vec<&GetTournamentsTournaments>>();
            if sync_tournaments.len() == 0 {
                Ok(CreateInteractionResponseMessage::new()
                    .ephemeral(true)
                    .content("Нет синхронизированных турниров"))
            } else {
                Ok(CreateInteractionResponseMessage::new()
                .ephemeral(true)
                .embed(
                    CreateEmbed::new()
                        .title("Настройка параметров синхронизированного турнира")
                        .description("Здесь можно настроить некоторые параметры уже существующих турниров(временная мера, пока все не автоматизировано)")
                        .fields(build_managed_tournament_fields(
                            current_managed_tournament, organizer, tournaments_service, challonge_service
                        ).await?)
                )
                .components(build_managed_tournament_components(current_managed_tournament, sync_tournaments).await?)
            )
            }
        }
        _ => Ok(CreateInteractionResponseMessage::new()
            .ephemeral(true)
            .content("Вы не зарегистрированы, как организатор турниров")),
    }
}

async fn build_managed_tournament_fields(
    id: Option<&Uuid>,
    organizer: GetOrganizerOrganizer,
    tournaments_service: &H5TournamentsService,
    challonge_service: &ChallongeService,
) -> Result<Vec<(String, String, bool)>, crate::Error> {
    if id.is_none() {
        Ok(vec![])
    } else {
        let tournament_data = tournaments_service
            .get_tournament_data(GetTournament::default().with_id(*id.unwrap()))
            .await?
            .unwrap();
        let users_data = tournaments_service
            .get_tournament_users(*id.unwrap())
            .await?;
        let challonge_participants = challonge_service
            .get_participants(
                &organizer.challonge,
                tournament_data.challonge_id.as_ref().unwrap(),
            )
            .await?;
        let participants_count = users_data.len();
        let challonge_participants_count = challonge_participants.len();
        Ok(vec![
            (
                "Число участников: дискорд - ".to_string(),
                participants_count.to_string(),
                false,
            ),
            (
                "Число участников: Challonge - ".to_string(),
                challonge_participants_count.to_string(),
                false,
            ),
        ])
    }
}

async fn build_managed_tournament_components(
    id: Option<&Uuid>,
    sync_tournaments: Vec<&GetTournamentsTournaments>,
) -> Result<Vec<CreateActionRow>, crate::Error> {
    let mut components = vec![];
    components.push(CreateActionRow::SelectMenu(CreateSelectMenu::new(
        "tournament_to_manage_selector",
        CreateSelectMenuKind::String {
            options: Vec::from_iter(sync_tournaments.iter().map(|t| {
                CreateSelectMenuOption::new(t.name.clone(), t.id)
                    .default_selection(id.is_some() && *id.unwrap() == t.id)
            })),
        },
    )));
    if id.is_some() {
        components.push(CreateActionRow::Buttons(vec![
            CreateButton::new("sync_participants_button")
                .label("Синхронизировать участников турнира")
                .style(ButtonStyle::Primary),
        ]));
    }
    Ok(components)
}
