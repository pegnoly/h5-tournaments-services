use std::str::FromStr;

use poise::serenity_prelude::{ChannelId, ComponentInteraction, ComponentInteractionDataKind, Context, EventHandler, GuildId, Interaction, Message, MessageId, MessageInteractionMetadata};
use shuttle_runtime::async_trait;
use tokio::sync::RwLock;
use uuid::{uuid, Uuid};

use crate::{api_connector::service::ApiConnectionService, builders};

pub struct MainEventHandler {
    api: ApiConnectionService
}

impl MainEventHandler {
    pub fn new(client: reqwest::Client) -> Self {
        MainEventHandler { api: ApiConnectionService::new(client) }
    }

    async fn dispatch_buttons(&self, context: &Context, interaction: &ComponentInteraction, id: &String, channel: u64, user: u64) {
        match id.as_str() {
            "create_report_button" => {
                builders::report_message::initial_build(context, &self.api, &interaction, id, channel, user).await.unwrap();
            },
            "start_report" => {
                let response_message = builders::report_message::build_games_message(
                    context, &self.api, interaction.message.id.get()).await.unwrap();
                
                interaction.create_response(context, poise::serenity_prelude::CreateInteractionResponse::UpdateMessage(response_message)).await.unwrap();
            }
            _=> {}
        }
    }

    async fn dispatch_selection(&self, context: &Context, interaction: &ComponentInteraction, message_id: u64, component_id: &String, selected: &String) -> Result<(), crate::Error> {
        let api = &self.api;
        let match_data = api.get_match(None, None, Some(message_id.to_string())).await?;
        match component_id.as_str() {
            "games_count_selector" => {
                let value = i32::from_str_radix(&selected, 10).unwrap();
                if let Some(existing_match) = match_data {
                    let res = api.update_match(existing_match.id, None, Some(value as i64), None).await?;
                    let rebuilt_message = builders::report_message::rebuild_initial(existing_match.id, &self.api).await?;
                    tracing::info!("Match was updated with games_count of {} with reply {}", value, res);
                    interaction.create_response(context, poise::serenity_prelude::CreateInteractionResponse::UpdateMessage(rebuilt_message)).await?;
                }
            },
            "opponent_selector" => {
                if let Some(existing_match) = match_data {
                    let res = api.update_match(existing_match.id, None, None, Some(Uuid::from_str(&selected).unwrap())).await?;
                    let rebuilt_message = builders::report_message::rebuild_initial(existing_match.id, &self.api).await?;
                    tracing::info!("Match was updated with selected user of {} with reply {}", selected, res);
                    interaction.create_response(context, poise::serenity_prelude::CreateInteractionResponse::UpdateMessage(rebuilt_message)).await?;
                }
            }
            _=> {}
        }
        Ok(())
    }

    async fn dispatch_message_created_by_interaction(&self, context: &Context, message_id: u64, interaction_id: u64) -> Result<(), crate::Error> {
        let api = &self.api;
        let existing_match = api.get_match(None, Some(interaction_id.to_string()), None).await?;
        if let Some(existing_match) = existing_match {
            let id = existing_match.id;
            let res = api.update_match(id, Some(message_id.to_string()), None, None).await?;
            tracing::info!("Match {} was updated with data_message with id {} in response of {}", id, message_id, res);
        }

        Ok(())
    }
}

#[async_trait]
impl EventHandler for MainEventHandler {
    async fn interaction_create(&self, context: Context, interaction: Interaction) {
        if let Some(component_interaction) =  interaction.message_component() {
            let channel = component_interaction.channel_id;
            let user = &component_interaction.user;
            tracing::info!("Component interaction in channel {} by user {}", channel.name(&context.http).await.unwrap(), user.name);

            match component_interaction.data.kind {
                ComponentInteractionDataKind::Button => {
                    let id = &component_interaction.data.custom_id;
                    self.dispatch_buttons(&context, &component_interaction, id, channel.get(), user.id.get()).await;
                },
                ComponentInteractionDataKind::StringSelect { ref values } => {
                    let id = &component_interaction.data.custom_id;
                    let selected_value = values.first();
                    let message = component_interaction.message.id.get();
                    self.dispatch_selection(&context, &component_interaction, message, id, selected_value.unwrap()).await.unwrap();
                }
                _=> {}
            }
        }
    }

    async fn message_delete(&self, context: Context, channel_id: ChannelId, deleted_message_id: MessageId, guild_id: Option<GuildId>) {
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