use poise::serenity_prelude::*;
use shuttle_runtime::async_trait;
use std::{collections::HashMap, sync::Arc};
use tokio::sync::RwLock;
use uuid::Uuid;

use crate::{
    builders::{
        self,
        types::{GameBuilder, GameBuilderContainer, MatchBuilder, TournamentBuilder},
    },
    graphql::queries::update_tournament_builder,
    operations,
    services::{
        challonge::service::ChallongeService, h5_tournaments::service::H5TournamentsService,
    },
};

pub struct LocalSyncBuilder {
    pub challonge_id: Option<String>,
    pub discord_id: Option<String>,
}

pub struct MainEventHandler {
    tournaments_service: Arc<H5TournamentsService>,
    challonge_service: Arc<ChallongeService>,
    tournament_builders: RwLock<HashMap<u64, RwLock<TournamentBuilder>>>,
    sync_builders: RwLock<HashMap<u64, LocalSyncBuilder>>,
    managed_tournaments: RwLock<HashMap<u64, Uuid>>,
    match_builders: RwLock<HashMap<u64, RwLock<MatchBuilder>>>,
    game_builders: RwLock<HashMap<u64, RwLock<GameBuilderContainer>>>,
}

impl MainEventHandler {
    pub fn new(
        tournaments_service: Arc<H5TournamentsService>,
        challonge_service: Arc<ChallongeService>,
    ) -> Self {
        MainEventHandler {
            tournaments_service: tournaments_service,
            challonge_service: challonge_service,
            tournament_builders: RwLock::new(HashMap::new()),
            sync_builders: RwLock::new(HashMap::new()),
            managed_tournaments: RwLock::new(HashMap::new()),
            match_builders: RwLock::new(HashMap::new()),
            game_builders: RwLock::new(HashMap::new()),
        }
    }

    async fn dispatch_buttons(
        &self,
        context: &Context,
        interaction: &ComponentInteraction,
        component_id: &String,
        channel: u64,
        user: u64,
    ) -> Result<(), crate::Error> {
        match component_id.as_str() {
            "create_report_button" => {
                builders::report_message::collect_match_creation_data(
                    context,
                    interaction,
                    &self.tournaments_service,
                    &self.challonge_service,
                    &self.match_builders,
                )
                .await?;
            }
            "start_report" => {
                operations::report_creation::finish_match_creation(
                    context,
                    interaction,
                    &self.tournaments_service,
                    &self.match_builders,
                    &self.game_builders,
                )
                .await?;
            }
            "bargains_data_button" => {
                operations::report_creation::switch_to_edition_state(
                    interaction,
                    context,
                    &self.tournaments_service,
                    &self.game_builders,
                    builders::types::GameBuilderState::BargainsData,
                )
                .await?;
            }
            "opponent_data_button" => {
                operations::report_creation::switch_to_edition_state(
                    interaction,
                    context,
                    &self.tournaments_service,
                    &self.game_builders,
                    builders::types::GameBuilderState::OpponentData,
                )
                .await?;
            }
            "player_data_button" => {
                operations::report_creation::switch_to_edition_state(
                    interaction,
                    context,
                    &self.tournaments_service,
                    &self.game_builders,
                    builders::types::GameBuilderState::PlayerData,
                )
                .await?;
            }
            "result_data_button" => {
                operations::report_creation::switch_to_edition_state(
                    interaction,
                    context,
                    &self.tournaments_service,
                    &self.game_builders,
                    builders::types::GameBuilderState::ResultData,
                )
                .await?;
            }
            "next_game_button" => {
                operations::report_creation::switch_games(
                    interaction,
                    context,
                    &self.tournaments_service,
                    &self.game_builders,
                    1,
                )
                .await?;
            }
            "previous_game_button" => {
                operations::report_creation::switch_games(
                    interaction,
                    context,
                    &self.tournaments_service,
                    &self.game_builders,
                    -1,
                )
                .await?;
            }
            "submit_report" => {
                operations::report_creation::generate_final_report_message(
                    context,
                    interaction,
                    &self.tournaments_service,
                    &self.challonge_service,
                    &self.game_builders,
                )
                .await?;
            }
            "register_user_button" => {
                operations::registration::try_register_in_tournament(
                    interaction,
                    context,
                    &self.tournaments_service,
                    &self.challonge_service,
                )
                .await?;
            }
            "unregister_user_button" => {
                operations::registration::try_remove_registration(
                    interaction,
                    context,
                    &self.tournaments_service,
                    &self.challonge_service,
                )
                .await?;
            }
            "update_user_data_button" => {
                operations::registration::try_update_user_data(
                    interaction,
                    context,
                    &self.tournaments_service,
                )
                .await?;
            }
            "tournament_creation_button" => {
                builders::tournament_creation::build_tournament_creation_interface(
                    interaction,
                    context,
                    &self.tournament_builders,
                )
                .await?;
            },
            "setup_tournament_base_data_button" => {
                operations::administration::process_tournament_builder_state_change(
                    context,
                    interaction,
                    &self.tournament_builders,
                    builders::types::TournamentBuildState::BaseData
                )
                .await?;
            },
            "setup_tournament_channels_button" => {
                operations::administration::process_tournament_builder_state_change(
                    context,
                    interaction,
                    &self.tournament_builders,
                    builders::types::TournamentBuildState::ChannelsData
                )
                .await?;
            }
            "setup_tournament_reports_button" => {
                operations::administration::process_tournament_builder_state_change(
                    context,
                    interaction,
                    &self.tournament_builders,
                    builders::types::TournamentBuildState::ReportsData
                )
                .await?;
            }
            "admin_registration_button" => {
                operations::administration::start_admin_registration(
                    context,
                    interaction,
                    &self.tournaments_service,
                )
                .await?;
            }
            "tournament_sync_button" => {
                let message = builders::tournament_creation::build_sync_interface(
                    context,
                    interaction,
                    &self.tournaments_service,
                    &self.challonge_service,
                    &self.sync_builders,
                )
                .await?;
                interaction
                    .create_response(context, CreateInteractionResponse::Message(message))
                    .await?;
            }
            "enter_tournament_name_button" => {
                operations::administration::start_tournament_name_creation(
                    context,
                    interaction,
                    &self.tournaments_service,
                )
                .await?;
            }
            "submit_tournament_creation_button" => {
                operations::administration::finalize_tournament_creation(
                    context,
                    interaction,
                    &self.tournament_builders,
                    &self.tournaments_service,
                )
                .await?;
            }
            "sync_tournaments_button" => {
                operations::administration::start_synchronization(
                    context,
                    interaction,
                    &self.tournaments_service,
                    &self.sync_builders,
                )
                .await?;
            }
            "administrate_tournament_button" => {
                let message = builders::tournament_creation::build_manage_interface(
                    context,
                    interaction,
                    &self.tournaments_service,
                    &self.challonge_service,
                    &self.managed_tournaments,
                )
                .await?;
                interaction
                    .create_response(context, CreateInteractionResponse::Message(message))
                    .await?;
            }
            "sync_participants_button" => {
                operations::administration::start_participants_syncronization(
                    context,
                    interaction,
                    &self.tournaments_service,
                    &self.challonge_service,
                    &self.managed_tournaments,
                )
                .await?;
            },
            "bargains_amount_button" => {
                operations::report_creation::show_bargains_modal(interaction, context, &self.game_builders).await?;
            }
            _ => {}
        }
        Ok(())
    }

    async fn dispatch_string_selection(
        &self,
        context: &Context,
        interaction: &ComponentInteraction,
        _message_id: u64,
        component_id: &String,
        selected: &String,
    ) -> Result<(), crate::Error> {
        match component_id.as_str() {
            "games_count_selector" => {
                operations::report_creation::select_games_count(
                    context,
                    interaction,
                    &self.match_builders,
                    selected,
                )
                .await?;
            }
            "opponent_selector" => {
                operations::report_creation::select_opponent(
                    context,
                    interaction,
                    &self.match_builders,
                    selected,
                )
                .await?;
            }
            "player_race_selector" => {
                operations::report_creation::select_player_race(
                    interaction,
                    context,
                    &self.tournaments_service,
                    &self.game_builders,
                    selected,
                )
                .await?;
            }
            "opponent_race_selector" => {
                operations::report_creation::select_opponent_race(
                    interaction,
                    context,
                    &self.tournaments_service,
                    &self.game_builders,
                    selected,
                )
                .await?;
            }
            "player_hero_selector" => {
                operations::report_creation::select_player_hero(
                    interaction,
                    context,
                    &self.tournaments_service,
                    &self.game_builders,
                    selected,
                )
                .await?;
            }
            "opponent_hero_selector" => {
                operations::report_creation::select_opponent_hero(
                    interaction,
                    context,
                    &self.tournaments_service,
                    &self.game_builders,
                    selected,
                )
                .await?;
            }
            "game_result_selector" => {
                operations::report_creation::select_game_result(
                    interaction,
                    context,
                    &self.tournaments_service,
                    &self.game_builders,
                    selected,
                )
                .await?;
            }
            "tournament_bargains_usage_selector" => {
                operations::administration::select_tournament_builder_bargains_usage(
                    context,
                    interaction,
                    &self.tournament_builders,
                    selected,
                )
                .await?
            }
            "tournament_mod_type_selector" => {
                operations::administration::process_tournament_mod_type_selection(                    
                    context,
                    interaction,
                    &self.tournament_builders,
                    selected,
                )
                .await?
            }
            "tournament_game_type_selector" => {
                operations::administration::process_tournament_game_type_selection(                    
                    context,
                    interaction,
                    &self.tournament_builders,
                    selected,
                )
                .await?
            }
            "tournament_bargains_color_usage_selector" => {
                operations::administration::select_tournament_builder_bargains_color_usage(
                    context,
                    interaction,
                    &self.tournament_builders,
                    selected,
                )
                .await?
            }
            "tournament_foreign_heroes_usage_selector" => {
                operations::administration::select_tournament_builder_foreign_heroes_usage(
                    context,
                    interaction,
                    &self.tournament_builders,
                    selected,
                )
                .await?
            }
            "challonge_tournaments_selector" => {
                operations::administration::select_sync_challonge_id(
                    context,
                    interaction,
                    &self.tournaments_service,
                    &self.challonge_service,
                    &self.sync_builders,
                    selected,
                )
                .await?;
            }
            "discord_tournaments_selector" => {
                operations::administration::select_sync_discord_id(
                    context,
                    interaction,
                    &self.tournaments_service,
                    &self.challonge_service,
                    &self.sync_builders,
                    selected,
                )
                .await?;
            }
            "tournament_to_manage_selector" => {
                operations::administration::select_tournament_to_manage(
                    context,
                    interaction,
                    &self.tournaments_service,
                    &self.challonge_service,
                    &self.managed_tournaments,
                    selected,
                )
                .await?;
            },
            "player_hero_race_selector" => {
                operations::report_creation::select_player_hero_race(
                    interaction, 
                    context, 
                    &self.tournaments_service, 
                    &self.game_builders, 
                    selected
                ).await?;
            }
            "opponent_hero_race_selector" => {
                operations::report_creation::select_opponent_hero_race(
                    interaction, 
                    context, 
                    &self.tournaments_service, 
                    &self.game_builders, 
                    selected
                ).await?;
            },
            "game_outcome_selector" => {
                operations::report_creation::select_game_outcome(
                    interaction, 
                    context, 
                    &self.tournaments_service, 
                    &self.game_builders, 
                    selected
                ).await?;
            },
            "bargains_color_selector" => {
                operations::report_creation::select_bargains_color(
                    interaction, 
                    context, 
                    &self.tournaments_service, 
                    &self.game_builders, 
                    selected
                ).await?;
            },
            _ => {}
        }
        Ok(())
    }

    async fn dispatch_channel_selection(
        &self,
        context: &Context,
        interaction: &ComponentInteraction,
        _message_id: u64,
        component_id: &String,
        selected: u64,
    ) -> Result<(), crate::Error> {
        match component_id.as_str() {
            "registration_channel_selector" => {
                operations::administration::select_tournament_builder_register_channel(
                    context,
                    interaction,
                    &self.tournament_builders,
                    selected,
                )
                .await?
            }
            "reports_channel_selector" => {
                operations::administration::select_tournament_builder_reports_channel(
                    context,
                    interaction,
                    &self.tournament_builders,
                    selected,
                )
                .await?
            }
            _ => {}
        }
        Ok(())
    }

    async fn dispatch_role_selection(
        &self,
        context: &Context,
        interaction: &ComponentInteraction,
        _message_id: u64,
        component_id: &String,
        selected: u64,
    ) -> Result<(), crate::Error> {
        match component_id.as_str() {
            "tournament_role_selector" => {
                operations::administration::select_tournament_builder_role(
                    context,
                    interaction,
                    &self.tournament_builders,
                    selected,
                )
                .await?
            }
            _ => {}
        }
        Ok(())
    }

    async fn dispatch_modals(
        &self,
        context: &Context,
        interaction: &ModalInteraction,
    ) -> Result<(), crate::Error> {
        match interaction.data.custom_id.as_str() {
            "bargains_input_modal" => {
                operations::report_creation::process_bargains_modal(
                    interaction, 
                    context, 
                    &self.tournaments_service,
                    &self.game_builders
                ).await?;
            }
            "user_lobby_nickname_modal" => {
                operations::registration::process_registration_modal(
                    interaction,
                    &context,
                    &self.tournaments_service,
                    &self.challonge_service,
                )
                .await?;
            }
            "user_update_nickname_modal" => {
                operations::registration::process_user_update_modal(
                    interaction,
                    &context,
                    &self.tournaments_service,
                )
                .await?;
            }
            "tournament_admin_challonge_key_modal" => {
                operations::administration::process_admin_registration_modal(
                    context,
                    interaction,
                    &self.tournaments_service,
                )
                .await?;
            }
            "tournament_creation_name_modal" => {
                operations::administration::process_tournament_name_creation_modal(
                    context,
                    interaction,
                    &self.tournament_builders,
                )
                .await?;
            }
            _ => {}
        }
        Ok(())
    }

    async fn dispatch_message_created_by_interaction(
        &self,
        _context: &Context,
        _message_id: u64,
        _interaction_id: u64,
    ) -> Result<(), crate::Error> {
        Ok(())
    }
}

#[async_trait]
impl EventHandler for MainEventHandler {
    async fn interaction_create(&self, context: Context, interaction: Interaction) {
        if let Some(component_interaction) = interaction.as_message_component() {
            let channel = component_interaction.channel_id;
            let user = &component_interaction.user;
            tracing::info!(
                "Component interaction in channel {} by user {}",
                channel.name(&context.http).await.unwrap(),
                user.name
            );

            match component_interaction.data.kind {
                ComponentInteractionDataKind::Button => {
                    let id = &component_interaction.data.custom_id;
                    self.dispatch_buttons(
                        &context,
                        &component_interaction,
                        id,
                        channel.get(),
                        user.id.get(),
                    )
                    .await
                    .unwrap();
                }
                ComponentInteractionDataKind::StringSelect { ref values } => {
                    let id = &component_interaction.data.custom_id;
                    let selected_value = values.first();
                    let message = component_interaction.message.id.get();
                    self.dispatch_string_selection(
                        &context,
                        &component_interaction,
                        message,
                        id,
                        selected_value.unwrap(),
                    )
                    .await
                    .unwrap();
                }
                ComponentInteractionDataKind::ChannelSelect { ref values } => {
                    let id = &component_interaction.data.custom_id;
                    let selected_value = values.first();
                    let message = component_interaction.message.id.get();
                    self.dispatch_channel_selection(
                        &context,
                        &component_interaction,
                        message,
                        id,
                        selected_value.unwrap().get(),
                    )
                    .await
                    .unwrap();
                }
                ComponentInteractionDataKind::RoleSelect { ref values } => {
                    let id = &component_interaction.data.custom_id;
                    let selected_value = values.first();
                    let message = component_interaction.message.id.get();
                    self.dispatch_role_selection(
                        &context,
                        &component_interaction,
                        message,
                        id,
                        selected_value.unwrap().get(),
                    )
                    .await
                    .unwrap();
                }
                _ => {}
            }
        } else if let Some(modal_interaction) = interaction.as_modal_submit() {
            self.dispatch_modals(&context, modal_interaction)
                .await
                .unwrap();
        }
    }

    async fn message_delete(
        &self,
        context: Context,
        channel_id: ChannelId,
        deleted_message_id: MessageId,
        _guild_id: Option<GuildId>,
    ) {
        tracing::info!(
            "Message {} was deleted from channel {}",
            deleted_message_id.get(),
            &channel_id.name(context).await.unwrap()
        );
    }

    async fn message(&self, context: Context, new_message: Message) {
        if let Some(interaction) = new_message.interaction_metadata {
            match *interaction {
                MessageInteractionMetadata::Component(component_interaction) => {
                    let id = component_interaction.id.get();
                    tracing::info!(
                        "Message {} created as response to interaction {}",
                        new_message.id.get(),
                        id
                    );
                    self.dispatch_message_created_by_interaction(
                        &context,
                        new_message.id.get(),
                        id,
                    )
                    .await
                    .unwrap();
                }
                _ => {}
            }
        }
    }
}
