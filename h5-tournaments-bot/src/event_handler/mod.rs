use std::str::FromStr;

use itertools::Itertools;
use poise::serenity_prelude::*;
use shuttle_runtime::async_trait;
use uuid::Uuid;

use crate::{api_connector::service::ApiConnectionService, builders, graphql::queries::{get_game_query, get_games_query::{self, GetGamesQueryGames}, int_to_game_result, update_game_mutation::{self, GameResult}}};

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
                let response_message = builders::report_message::build_game_message(
                    context, &self.api, interaction.message.id.get()).await.unwrap();
                interaction.create_response(context, poise::serenity_prelude::CreateInteractionResponse::UpdateMessage(response_message)).await.unwrap();
                // interaction.create_response(context, poise::serenity_prelude::CreateInteractionResponse::Message(second_part)).await.unwrap();
            },
            "bargains_data_button" => {
                interaction.create_response(context, CreateInteractionResponse::Modal(
                    CreateModal::new("player_data_modal", "Указать размер торга")
                        .components(vec![
                            CreateActionRow::InputText(CreateInputText::new(InputTextStyle::Short, "Торг", "bargains_amount_input"))
                        ])
                )).await.unwrap();
            },
            "opponent_data_button" => {
                let message = interaction.message.id.get();
                let current_match = self.api.get_match(None, None, Some(message.to_string())).await.unwrap();
                if let Some(match_data) = current_match {
                    self.api.update_game(
                        match_data.id, 
                        match_data.current_game, 
                        Some(update_game_mutation::GameEditState::OPPONENT_DATA), 
                        None,
                        None, 
                        None, 
                        None, 
                        None,
                    None).await.unwrap();
                    let updated_message = builders::report_message::build_game_message(&context, &self.api, message).await.unwrap();
                    interaction.create_response(context, poise::serenity_prelude::CreateInteractionResponse::UpdateMessage(updated_message)).await.unwrap();
                }
            },
            "player_data_button" => {
                let message = interaction.message.id.get();
                let current_match = self.api.get_match(None, None, Some(message.to_string())).await.unwrap();
                if let Some(match_data) = current_match {
                    self.api.update_game(
                        match_data.id, 
                        match_data.current_game, 
                        Some(update_game_mutation::GameEditState::PLAYER_DATA), 
                        None,
                        None, 
                        None, 
                        None, 
                        None,
                    None).await.unwrap();
                    let updated_message = builders::report_message::build_game_message(&context, &self.api, message).await.unwrap();
                    interaction.create_response(context, poise::serenity_prelude::CreateInteractionResponse::UpdateMessage(updated_message)).await.unwrap();
                }
            },
            "result_data_button" => {
                let message = interaction.message.id.get();
                let current_match = self.api.get_match(None, None, Some(message.to_string())).await.unwrap();
                if let Some(match_data) = current_match {
                    self.api.update_game(
                        match_data.id, 
                        match_data.current_game, 
                        Some(update_game_mutation::GameEditState::RESULT_DATA), 
                        None,
                        None, 
                        None, 
                        None, 
                        None,
                    None).await.unwrap();
                    let updated_message = builders::report_message::build_game_message(&context, &self.api, message).await.unwrap();
                    interaction.create_response(context, poise::serenity_prelude::CreateInteractionResponse::UpdateMessage(updated_message)).await.unwrap();
                }
            },
            "next_game_button" => {
                let message = interaction.message.id.get();
                let current_match = self.api.get_match(None, None, Some(message.to_string())).await.unwrap();
                if let Some(match_data) = current_match {
                    let new_game = match_data.current_game + 1;
                    self.api.update_match(
                        match_data.id, 
                        None, 
                        None, 
                        None,
                    Some(new_game))
                    .await.unwrap();
                    let updated_message = builders::report_message::build_game_message(&context, &self.api, message).await.unwrap();
                    interaction.create_response(context, poise::serenity_prelude::CreateInteractionResponse::UpdateMessage(updated_message)).await.unwrap();
                }
            },
            "previous_game_button" => {
                let message = interaction.message.id.get();
                let current_match = self.api.get_match(None, None, Some(message.to_string())).await.unwrap();
                if let Some(match_data) = current_match {
                    let new_game = match_data.current_game - 1;
                    self.api.update_match(
                        match_data.id, 
                        None, 
                        None, 
                        None,
                    Some(new_game))
                    .await.unwrap();
                    let updated_message = builders::report_message::build_game_message(&context, &self.api, message).await.unwrap();
                    interaction.create_response(context, poise::serenity_prelude::CreateInteractionResponse::UpdateMessage(updated_message)).await.unwrap();
                }
            },
            "submit_report" => {
                let message = interaction.message.id.get();
                let current_match = self.api.get_match(None, None, Some(message.to_string())).await.unwrap();
                if let Some(match_data) = current_match {
                    let tournament_data = self.api.get_tournament_data(Some(match_data.tournament), None).await.unwrap().unwrap();
                    let operator_data = self.api.get_operator_data(tournament_data.operator).await.unwrap();
                    let output_channel = ChannelId::from(operator_data.generated as u64);
                    let first_user = self.api.get_user(Some(match_data.first_player), None).await.unwrap().unwrap().nickname;
                    let second_user = self.api.get_user(Some(match_data.second_player.unwrap()), None).await.unwrap().unwrap().nickname;
                    let games = self.api.get_games(match_data.id).await.unwrap();
                    let sorted_games = games.iter()
                        .sorted_by_key(|g| g.number)
                        .collect::<Vec<&GetGamesQueryGames>>();

                    let mut fields = vec![];
                    for game in &sorted_games {
                        fields.push(
                            (
                                format!("_Игра {}_", game.number),
                                format!("**{}(**_{}_**)** **{}** **{}** **(**_{}_**)** [**{}**]", 
                                &self.api.races.iter().find(|r| r.id == game.first_player_race.unwrap()).unwrap().name,
                                &self.api.get_hero(game.first_player_hero.unwrap()).await.unwrap().unwrap().name,
                                match game.result {
                                    get_games_query::GameResult::FIRST_PLAYER_WON => ">".to_string(),
                                    get_games_query::GameResult::SECOND_PLAYER_WON => "<".to_string(),
                                    _=> "?".to_string()
                                },
                                &self.api.races.iter().find(|r| r.id == game.second_player_race.unwrap()).unwrap().name,
                                &self.api.get_hero(game.second_player_hero.unwrap()).await.unwrap().unwrap().name,
                                game.bargains_amount.unwrap().to_string()
                                ),
                                false
                            ) 
                        )
                    }

                    let message_builder = CreateMessage::new()
                        .add_embed(
                            CreateEmbed::new()
                                .title(format!("**Турнир {}**", &tournament_data.name))
                                .description(format!("**{}** _VS_ **{}**", &first_user, &second_user))
                                .fields(fields)
                        );
                    
                    output_channel.send_message(context, message_builder).await.unwrap();
                    interaction.create_response(context, CreateInteractionResponse::Acknowledge).await.unwrap();
                    context.http.delete_message(ChannelId::from(tournament_data.channel as u64), MessageId::from(interaction.message.id.get()), Some("Report cleanup")).await.unwrap();
                }
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
                    let res = api.update_match(existing_match.id, None, Some(value as i64), None, None).await?;
                    let rebuilt_message = builders::report_message::rebuild_initial(existing_match.id, &self.api).await?;
                    tracing::info!("Match was updated with games_count of {} with reply {}", value, res);
                    interaction.create_response(context, poise::serenity_prelude::CreateInteractionResponse::UpdateMessage(rebuilt_message)).await?;
                }
            },
            "opponent_selector" => {
                if let Some(existing_match) = match_data {
                    let res = api.update_match(existing_match.id, None, None, Some(Uuid::from_str(&selected).unwrap()), None).await?;
                    let rebuilt_message = builders::report_message::rebuild_initial(existing_match.id, &self.api).await?;
                    tracing::info!("Match was updated with selected user of {} with reply {}", selected, res);
                    interaction.create_response(context, poise::serenity_prelude::CreateInteractionResponse::UpdateMessage(rebuilt_message)).await?;
                }
            },
            "player_race_selector" => {
                if let Some(existing_match) = match_data {
                    let selected_race = i64::from_str_radix(&selected, 10)?;
                    self.api.update_game(
                        existing_match.id, 
                        existing_match.current_game, 
                        None, 
                        Some(selected_race), 
                        None,
                        None, 
                        None,
                        None,
                    None)
                    .await?;
                    let rebuild_message = builders::report_message::build_game_message(context, &self.api, message_id).await?;
                    interaction.create_response(context, CreateInteractionResponse::UpdateMessage(rebuild_message)).await?;
                }
            },
            "opponent_race_selector" => {
                if let Some(existing_match) = match_data {
                    let selected_race = i64::from_str_radix(&selected, 10)?;
                    self.api.update_game(
                        existing_match.id, 
                        existing_match.current_game, 
                        None, 
                        None, 
                        None,
                        Some(selected_race), 
                        None,
                        None,
                    None)
                    .await?;
                    let rebuild_message = builders::report_message::build_game_message(context, &self.api, message_id).await?;
                    interaction.create_response(context, CreateInteractionResponse::UpdateMessage(rebuild_message)).await?;
                }
            },
            "player_hero_selector" => {
                if let Some(existing_match) = match_data {
                    let selected_hero = i64::from_str_radix(&selected, 10)?;
                    self.api.update_game(
                        existing_match.id, 
                        existing_match.current_game, 
                        None, 
                        None, 
                        Some(selected_hero),
                        None, 
                        None,
                        None,
                    None)
                    .await?;
                    let rebuild_message = builders::report_message::build_game_message(context, &self.api, message_id).await?;
                    interaction.create_response(context, CreateInteractionResponse::UpdateMessage(rebuild_message)).await?;
                }
            },
            "opponent_hero_selector" => {
                if let Some(existing_match) = match_data {
                    let selected_hero = i64::from_str_radix(&selected, 10)?;
                    self.api.update_game(
                        existing_match.id, 
                        existing_match.current_game, 
                        None, 
                        None, 
                        None,
                        None, 
                        Some(selected_hero),
                        None,
                    None)
                    .await?;
                    let rebuild_message = builders::report_message::build_game_message(context, &self.api, message_id).await?;
                    interaction.create_response(context, CreateInteractionResponse::UpdateMessage(rebuild_message)).await?;
                }
            },
            "game_result_selector" => {
                if let Some(existing_match) = match_data {
                    let selected_result = i32::from_str_radix(&selected, 10)?;
                    let result = int_to_game_result(selected_result);
                    self.api.update_game(
                        existing_match.id, 
                        existing_match.current_game, 
                        None, 
                        None, 
                        None,
                        None, 
                        None,
                        None,
                    Some(result))
                    .await?;
                    let rebuild_message = builders::report_message::build_game_message(context, &self.api, message_id).await?;
                    interaction.create_response(context, CreateInteractionResponse::UpdateMessage(rebuild_message)).await?;
                }
            }
            _=> {}
        }
        Ok(())
    }

    async fn dispatch_message_created_by_interaction(&self, _context: &Context, message_id: u64, interaction_id: u64) -> Result<(), crate::Error> {
        let api = &self.api;
        let existing_match = api.get_match(None, Some(interaction_id.to_string()), None).await?;
        if let Some(existing_match) = existing_match {
            let id = existing_match.id;
            let res = api.update_match(id, Some(message_id.to_string()), None, None, None).await?;
            tracing::info!("Match {} was updated with data_message with id {} in response of {}", id, message_id, res);
        }

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
                    self.dispatch_buttons(&context, &component_interaction, id, channel.get(), user.id.get()).await;
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
            match modal_interaction.data.custom_id.as_str() {
                "player_data_modal" => {
                    let message = &modal_interaction.message.as_ref().unwrap().content;
                    let mut bargains_value = 0;
                    tracing::info!("Modal was created from message: {}", message);
                    tracing::info!("Modal data: {:?}", &modal_interaction.data);
                    for row in &modal_interaction.data.components {
                        for component in &row.components {
                            match component {
                                ActionRowComponent::InputText(text) => {
                                    if text.custom_id.as_str() == "bargains_amount_input" {
                                        let value = i32::from_str_radix(&text.value.as_ref().unwrap(), 10).unwrap();
                                        bargains_value = value;
                                        tracing::info!("Bargains amount: {}", value);
                                    }
                                },
                                _=> {}
                            }
                        }
                    }
                    let match_data = self.api.get_match(
                        None, 
                        None, 
                        Some(modal_interaction.message.as_ref().unwrap().id.get().to_string())
                    ).await.unwrap();
                    if let Some(existing_match) = match_data {
                        self.api.update_game(
                            existing_match.id, 
                            existing_match.current_game, 
                            None, 
                            None,
                            None, 
                            None,
                            None,
                            Some(bargains_value as i64),
                            None
                        ).await.unwrap();
                        let rebuilt_message=  builders::report_message::build_game_message(&context, &self.api, modal_interaction.message.as_ref().unwrap().id.get()).await.unwrap();
                        modal_interaction.create_response(context, CreateInteractionResponse::UpdateMessage(rebuilt_message)).await.unwrap();
                    }
                },
                _=> {}
            }
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