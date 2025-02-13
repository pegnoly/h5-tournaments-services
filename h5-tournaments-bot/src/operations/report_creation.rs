use std::str::FromStr;

use itertools::Itertools;
use poise::serenity_prelude::*;
use uuid::Uuid;

use crate::{api_connector::service::ApiConnectionService, builders, graphql::queries::{get_games_query::{self, GetGamesQueryGames}, int_to_game_result, update_game_mutation}, types::payloads::{GetMatch, GetTournament, GetUser, UpdateGame, UpdateMatch}};

pub async fn show_bargains_modal(
    interaction: &ComponentInteraction,
    context: &Context
) -> Result<(), crate::Error> {
    interaction.create_response(context, CreateInteractionResponse::Modal(
        CreateModal::new("player_data_modal", "Указать размер торга")
            .components(vec![
                CreateActionRow::InputText(CreateInputText::new(InputTextStyle::Short, "Торг", "bargains_amount_input"))
            ])
    )).await?;
    Ok(())
}

pub async fn switch_to_edition_state(
    interaction: &ComponentInteraction,
    context: &Context,
    api: &ApiConnectionService,
    new_state: update_game_mutation::GameEditState
) -> Result<(), crate::Error> {
    let message = interaction.message.id.get();
    if let Some(match_data) = api.get_match(GetMatch::default().with_message_id(interaction.message.id.get())).await? {
        api.update_game(
            UpdateGame::new(match_data.id, match_data.current_game)
                .with_edit_state(new_state)
            ).await?;
        let updated_message = builders::report_message::build_game_message(&context, &api, message).await?;
        interaction.create_response(context, poise::serenity_prelude::CreateInteractionResponse::UpdateMessage(updated_message)).await?;
    }
    Ok(())
}

pub async fn switch_games(
    interaction: &ComponentInteraction,
    context: &Context,
    api: &ApiConnectionService,
    game_change: i64
) -> Result<(), crate::Error> {
    let message = interaction.message.id.get();
    if let Some(match_data) = api.get_match(GetMatch::default().with_message_id(message)).await? {
        let new_game = match_data.current_game + game_change;
        api.update_match(UpdateMatch::new(match_data.id).with_current_game(new_game)).await?;
        let updated_message = builders::report_message::build_game_message(&context, api, message).await?;
        interaction.create_response(context, poise::serenity_prelude::CreateInteractionResponse::UpdateMessage(updated_message)).await?;
    }
    Ok(())
}

pub async fn generate_final_report_message(
    interaction: &ComponentInteraction,
    context: &Context,
    api: &ApiConnectionService
) -> Result<(), crate::Error> {
    let message = interaction.message.id.get();
    if let Some(match_data) = api.get_match(GetMatch::default().with_message_id(message)).await? {
        let tournament_data = api.get_tournament_data(GetTournament::default().with_id(match_data.tournament)).await?.unwrap();
        let operator_data = api.get_operator_data(tournament_data.operator).await?;
        let output_channel = ChannelId::from(operator_data.generated as u64);
        let first_user = api.get_user(GetUser::default().with_id(match_data.first_player)).await?.unwrap();
        let participant = api.get_participant(tournament_data.id, first_user.id).await.unwrap().unwrap();
        let second_user = api.get_user(GetUser::default().with_id(match_data.second_player.unwrap())).await?.unwrap().nickname;
        let games = api.get_games(match_data.id).await?;
        let sorted_games = games.iter()
            .sorted_by_key(|g| g.number)
            .collect::<Vec<&GetGamesQueryGames>>();

        let first_player_wins = sorted_games.iter()
            .filter(|g| g.result == get_games_query::GameResult::FIRST_PLAYER_WON)
            .count();

        let second_player_wins = sorted_games.iter()
            .filter(|g| g.result == get_games_query::GameResult::SECOND_PLAYER_WON)
            .count();

        let mut fields = vec![];
        for game in &sorted_games {
            fields.push(
                (
                    format!("_Игра {}_", game.number),
                    format!("**{},** _{}_ **{}** **{},** _{}_ [**{}**]", 
                    &api.races.iter().find(|r| r.id == game.first_player_race.unwrap()).unwrap().name,
                    &api.get_hero(game.first_player_hero.unwrap()).await.unwrap().unwrap().name,
                    match game.result {
                        get_games_query::GameResult::FIRST_PLAYER_WON => ">".to_string(),
                        get_games_query::GameResult::SECOND_PLAYER_WON => "<".to_string(),
                        _=> "?".to_string()
                    },
                    &api.races.iter().find(|r| r.id == game.second_player_race.unwrap()).unwrap().name,
                    &api.get_hero(game.second_player_hero.unwrap()).await?.unwrap().name,
                    game.bargains_amount.to_string()
                    ),
                    false
                ) 
            )
        }

        fields.push(
            (
                "_Счёт_".to_string(),
                format!("**{} - {}**", first_player_wins, second_player_wins),
                false
            )
        );

        let message_builder = CreateMessage::new()
            .add_embed(
                CreateEmbed::new()
                    .title(format!("**Турнир {}**, _групповой этап, группа_ **{}**", &tournament_data.name.to_uppercase(), participant.group))
                    .description(format!("**{}** _VS_ **{}**", &first_user.nickname, &second_user))
                    .fields(fields)
            );
        
        output_channel.send_message(context, message_builder).await?;
        interaction.create_response(context, CreateInteractionResponse::UpdateMessage(
            CreateInteractionResponseMessage::new()
                .add_embed(CreateEmbed::new().title("Отчет успешно создан, можете закрыть это сообщение."))
                .components(vec![])
        )).await?;
    }
    Ok(())
}

pub async fn select_games_count(
    interaction: &ComponentInteraction,
    context: &Context,
    api: &ApiConnectionService,
    message_id: u64,
    selected_value: &String
) -> Result<(), crate::Error> {
    let value = i64::from_str_radix(selected_value, 10).unwrap();
    if let Some(match_data) = api.get_match(GetMatch::default().with_message_id(message_id)).await? {
        let res = api.update_match(UpdateMatch::new(match_data.id).with_games_count(value)).await?;
        let rebuilt_message = builders::report_message::rebuild_initial(match_data.id, api).await?;
        tracing::info!("Match was updated with games_count of {} with reply {}", value, res);
        interaction.create_response(context, poise::serenity_prelude::CreateInteractionResponse::UpdateMessage(rebuilt_message)).await?;
    }
    Ok(())
}

pub async fn select_opponent(
    interaction: &ComponentInteraction,
    context: &Context,
    api: &ApiConnectionService,
    message_id: u64,
    selected_value: &String
) -> Result<(), crate::Error> {
    if let Some(match_data) = api.get_match(GetMatch::default().with_message_id(message_id)).await?  {
        let res = api.update_match(UpdateMatch::new(match_data.id).with_second_player(Uuid::from_str(selected_value).unwrap())).await?;
        let rebuilt_message = builders::report_message::rebuild_initial(match_data.id, api).await?;
        tracing::info!("Match was updated with selected user of {} with reply {}", selected_value, res);
        interaction.create_response(context, poise::serenity_prelude::CreateInteractionResponse::UpdateMessage(rebuilt_message)).await?;
    }
    Ok(())
}

pub async fn select_player_race(
    interaction: &ComponentInteraction,
    context: &Context,
    api: &ApiConnectionService,
    message_id: u64,
    selected_value: &String
) -> Result<(), crate::Error> {
    if let Some(match_data) = api.get_match(GetMatch::default().with_message_id(message_id)).await?  {
        let selected_race = i64::from_str_radix(selected_value, 10)?;
        api.update_game(UpdateGame::new(match_data.id, match_data.current_game).with_first_player_race(selected_race)).await?;
        let rebuild_message = builders::report_message::build_game_message(context, api, message_id).await?;
        interaction.create_response(context, CreateInteractionResponse::UpdateMessage(rebuild_message)).await?;
    }
    Ok(())
}

pub async fn select_opponent_race(
    interaction: &ComponentInteraction,
    context: &Context,
    api: &ApiConnectionService,
    message_id: u64,
    selected_value: &String
) -> Result<(), crate::Error> {
    if let Some(match_data) = api.get_match(GetMatch::default().with_message_id(message_id)).await?  {
        let selected_race = i64::from_str_radix(selected_value, 10)?;
        api.update_game(UpdateGame::new(match_data.id, match_data.current_game).with_second_player_race(selected_race)).await?;
        let rebuild_message = builders::report_message::build_game_message(context, api, message_id).await?;
        interaction.create_response(context, CreateInteractionResponse::UpdateMessage(rebuild_message)).await?;
    }
    Ok(())
}

pub async fn select_player_hero(
    interaction: &ComponentInteraction,
    context: &Context,
    api: &ApiConnectionService,
    message_id: u64,
    selected_value: &String
) -> Result<(), crate::Error> {
    if let Some(match_data) = api.get_match(GetMatch::default().with_message_id(message_id)).await?  {
        let selected_hero = i64::from_str_radix(selected_value, 10)?;
        api.update_game(UpdateGame::new(match_data.id, match_data.current_game).with_first_player_hero(selected_hero)).await?;
        let rebuild_message = builders::report_message::build_game_message(context, api, message_id).await?;
        interaction.create_response(context, CreateInteractionResponse::UpdateMessage(rebuild_message)).await?;
    }
    Ok(())
}

pub async fn select_opponent_hero(
    interaction: &ComponentInteraction,
    context: &Context,
    api: &ApiConnectionService,
    message_id: u64,
    selected_value: &String
) -> Result<(), crate::Error> {
    if let Some(match_data) = api.get_match(GetMatch::default().with_message_id(message_id)).await?  {
        let selected_hero = i64::from_str_radix(selected_value, 10)?;
        api.update_game(UpdateGame::new(match_data.id, match_data.current_game).with_second_player_hero(selected_hero)).await?;
        let rebuild_message = builders::report_message::build_game_message(context, api, message_id).await?;
        interaction.create_response(context, CreateInteractionResponse::UpdateMessage(rebuild_message)).await?;
    }
    Ok(())
}

pub async fn select_game_result(
    interaction: &ComponentInteraction,
    context: &Context,
    api: &ApiConnectionService,
    message_id: u64,
    selected_value: &String
) -> Result<(), crate::Error> {
    if let Some(match_data) = api.get_match(GetMatch::default().with_message_id(message_id)).await?  {
        let selected_result = i32::from_str_radix(selected_value, 10)?;
        let result = int_to_game_result(selected_result);
        api.update_game(UpdateGame::new(match_data.id, match_data.current_game).with_result(result)).await?;
        let rebuild_message = builders::report_message::build_game_message(context, api, message_id).await?;
        interaction.create_response(context, CreateInteractionResponse::UpdateMessage(rebuild_message)).await?;
    }
    Ok(())
}

pub async fn save_report_user_message(
    context: &Context, 
    api: &ApiConnectionService,
    message_id: u64, 
    interaction_id: u64
) -> Result<(), crate::Error> {
    if let Some(match_data) = api.get_match(GetMatch::default().with_interaction_id(interaction_id.to_string())).await? {
        let res = api.update_match(UpdateMatch::new(match_data.id).with_message(message_id.to_string())).await?;
        tracing::info!("Match {} was updated with data_message with id {} in response of {}", match_data.id, message_id, res);
    }
    Ok(())
}

pub async fn process_bargains_modal(
    interaction: &ModalInteraction,
    context: &Context,
    api: &ApiConnectionService
) -> Result<(), crate::Error> {
    let message = &interaction.message.as_ref().unwrap().content;
    let mut bargains_value = 0;
    tracing::info!("Modal was created from message: {}", message);
    tracing::info!("Modal data: {:?}", &interaction.data);
    for row in &interaction.data.components {
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
    if let Some(match_data) = api.get_match(GetMatch::default().with_message_id(interaction.message.as_ref().unwrap().id.get())).await? {
        api.update_game(UpdateGame::new(match_data.id, match_data.current_game).with_bargains_amount(bargains_value as i64)).await?;
        let rebuilt_message=  builders::report_message::build_game_message(&context, api, interaction.message.as_ref().unwrap().id.get()).await?;
        interaction.create_response(context, CreateInteractionResponse::UpdateMessage(rebuilt_message)).await.unwrap();
    }
    Ok(())
}