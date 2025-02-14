use std::sync::Arc;

use poise::serenity_prelude::*;
use shuttle_runtime::async_trait;

use crate::{services::h5_tournaments::service::H5TournamentsService, builders, operations, graphql::queries::update_game_mutation::GameEditState};

pub struct MainEventHandler {
    api: Arc<H5TournamentsService>
}

impl MainEventHandler {
    pub fn new(api: Arc<H5TournamentsService>) -> Self {
        MainEventHandler { api: api }
    }

    async fn dispatch_buttons(&self, context: &Context, interaction: &ComponentInteraction, component_id: &String, channel: u64, user: u64) -> Result<(), crate::Error> {
        match component_id.as_str() {
            "create_report_button" => {
                builders::report_message::initial_build(context, &self.api, &interaction, component_id, channel, user).await?;
            },
            "start_report" => {
                let response_message = builders::report_message::build_game_message(
                    context, &self.api, interaction.message.id.get()).await.unwrap();
                interaction.create_response(context, poise::serenity_prelude::CreateInteractionResponse::UpdateMessage(response_message)).await?;
            },
            "bargains_data_button" => {
                operations::report_creation::show_bargains_modal(interaction, context).await?;
            },
            "opponent_data_button" => {
                operations::report_creation::switch_to_edition_state(interaction, context, &self.api, GameEditState::OPPONENT_DATA).await?;
            },
            "player_data_button" => {
                operations::report_creation::switch_to_edition_state(interaction, context, &self.api, GameEditState::PLAYER_DATA).await?;
            }
            "result_data_button" => {
                operations::report_creation::switch_to_edition_state(interaction, context, &self.api, GameEditState::RESULT_DATA).await?;
            },
            "next_game_button" => {
                operations::report_creation::switch_games(interaction, context, &self.api, 1).await?;
            },
            "previous_game_button" => {
                operations::report_creation::switch_games(interaction, context, &self.api, -1).await?;
            },
            "submit_report" => {
                operations::report_creation::generate_final_report_message(interaction, context, &self.api).await?;
            },
            "register_user_button" => {
                operations::registration::try_register_in_tournament(interaction, context, &self.api).await?;
            },
            "unregister_user_button" => {
                operations::registration::try_remove_registration(interaction, context, &self.api).await?;
            },
            "update_user_data_button" => {
                operations::registration::try_update_user_data(interaction, context, &self.api).await?;
            }
            _=> {}
        }
        Ok(())
    }

    async fn dispatch_selection(&self, context: &Context, interaction: &ComponentInteraction, message_id: u64, component_id: &String, selected: &String) -> Result<(), crate::Error> {
        match component_id.as_str() {
            "games_count_selector" => {
                operations::report_creation::select_games_count(interaction, context, &self.api, message_id, selected).await?;
            },
            "opponent_selector" => {
                operations::report_creation::select_opponent(interaction, context, &self.api, message_id, selected).await?;
            },
            "player_race_selector" => {
                operations::report_creation::select_player_race(interaction, context, &self.api, message_id, selected).await?;
            },
            "opponent_race_selector" => {
                operations::report_creation::select_opponent_race(interaction, context, &self.api, message_id, selected).await?;
            },
            "player_hero_selector" => {
                operations::report_creation::select_player_hero(interaction, context, &self.api, message_id, selected).await?;
            },
            "opponent_hero_selector" => {
                operations::report_creation::select_opponent_hero(interaction, context, &self.api, message_id, selected).await?;
            },
            "game_result_selector" => {
                operations::report_creation::select_game_result(interaction, context, &self.api, message_id, selected).await?;
            }
            _=> {}
        }
        Ok(())
    }

    async fn dispatch_modals(&self, context: &Context, interaction: &ModalInteraction) -> Result<(), crate::Error> {
        match interaction.data.custom_id.as_str() {
            "player_data_modal" => {
                operations::report_creation::process_bargains_modal(interaction, context, &self.api).await?;
            },
            "user_lobby_nickname_modal" => {
                operations::registration::process_registration_modal(interaction, &context, &self.api).await?;
            },
            "user_update_nickname_modal" => {
                operations::registration::process_user_update_modal(interaction, &context, &self.api).await?;
            }
            _=> {}
        }
        Ok(())
    }

    async fn dispatch_message_created_by_interaction(&self, _context: &Context, message_id: u64, interaction_id: u64) -> Result<(), crate::Error> {
        operations::report_creation::save_report_user_message(_context, &self.api, message_id, interaction_id).await?;
        Ok(())
    }
}

#[async_trait]
impl EventHandler for MainEventHandler {
    async fn interaction_create(&self, context: Context, interaction: Interaction) {
        if let Some(component_interaction) =  interaction.as_message_component() {
            let channel = component_interaction.channel_id;
            let user = &component_interaction.user;
            tracing::info!("Component interaction in channel {} by user {}", channel.name(&context.http).await.unwrap(), user.name);

            match component_interaction.data.kind {
                ComponentInteractionDataKind::Button => {
                    let id = &component_interaction.data.custom_id;
                    self.dispatch_buttons(&context, &component_interaction, id, channel.get(), user.id.get()).await.unwrap();
                },
                ComponentInteractionDataKind::StringSelect { ref values } => {
                    let id = &component_interaction.data.custom_id;
                    let selected_value = values.first();
                    let message = component_interaction.message.id.get();
                    self.dispatch_selection(&context, &component_interaction, message, id, selected_value.unwrap()).await.unwrap();
                },
                _=> {}
            }
        }
        else if let Some(modal_interaction) = interaction.as_modal_submit() {
            self.dispatch_modals(&context, modal_interaction).await.unwrap();
        }
    }

    async fn message_delete(&self, context: Context, channel_id: ChannelId, deleted_message_id: MessageId, _guild_id: Option<GuildId>) {
        tracing::info!("Message {} was deleted from channel {}", deleted_message_id.get(), &channel_id.name(context).await.unwrap());
    }

    async fn message(&self, context: Context, new_message: Message) {
        if let Some(interaction) = new_message.interaction_metadata {
            match *interaction {
                MessageInteractionMetadata::Component(component_interaction) => {
                    let id = component_interaction.id.get();
                    tracing::info!("Message {} created as response to interaction {}", new_message.id.get(), id);
                    self.dispatch_message_created_by_interaction(&context, new_message.id.get(), id).await.unwrap();
                    
                },
                _=> {}
            }
        }
    }
}