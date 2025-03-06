use itertools::Itertools;
use poise::serenity_prelude::*;
use std::{collections::HashMap, str::FromStr};
use tokio::sync::{RwLock, RwLockReadGuard};
use uuid::Uuid;

use crate::{
    builders::{
        self,
        report_message::build_game_message,
        types::{
            BargainsColor, GameBuilder, GameBuilderContainer, GameBuilderState, GameOutcome, GameResult, GameType, MatchBuilder, OpponentDataPayload, OpponentsData
        },
    },
    graphql::queries::{create_games_bulk, GetMatchQuery},
    services::{
        challonge::{
            payloads::{
                ChallongeMatchParticipantsData, ChallongeUpdateMatchAttributes,
                ChallongeUpdateMatchPayload,
            },
            service::ChallongeService, types::ChallongeTournamentState,
        },
        h5_tournaments::{
            payloads::{GetOperatorPayload, GetOrganizerPayload, GetParticipantPayload},
            service::H5TournamentsService,
        },
    },
    types::payloads::{GetMatch, GetTournament, GetUser, UpdateMatch},
};

pub async fn select_opponent(
    context: &Context,
    interaction: &ComponentInteraction,
    match_builders: &RwLock<HashMap<u64, RwLock<MatchBuilder>>>,
    selected_value: &String,
) -> Result<(), crate::Error> {
    tracing::info!("Selected opponent: {}", selected_value);
    let match_builders_locked = match_builders.read().await;
    if let Some(builder) = match_builders_locked.get(&interaction.message.id.get()) {
        let mut builder_locked = builder.write().await;
        tracing::info!("Builder locked: {:?}", &builder_locked);
        builder_locked.selected_opponent = Some(selected_value.clone());
        drop(builder_locked);
        let response_message =
            builders::report_message::build_match_creation_interface(&*builder.read().await)
                .await?;
        interaction
            .create_response(
                context,
                CreateInteractionResponse::UpdateMessage(response_message),
            )
            .await?;
    }
    Ok(())
}

pub async fn select_games_count(
    context: &Context,
    interaction: &ComponentInteraction,
    match_builders: &RwLock<HashMap<u64, RwLock<MatchBuilder>>>,
    selected_value: &String,
) -> Result<(), crate::Error> {
    let match_builders_locked = match_builders.read().await;
    if let Some(builder) = match_builders_locked.get(&interaction.message.id.get()) {
        let mut builder_locked = builder.write().await;
        builder_locked.games_count = Some(i32::from_str_radix(selected_value, 10)?);
        drop(builder_locked);
        let response_message =
            builders::report_message::build_match_creation_interface(&*builder.read().await)
                .await?;
        interaction
            .create_response(
                context,
                CreateInteractionResponse::UpdateMessage(response_message),
            )
            .await?;
    }
    Ok(())
}

pub async fn finish_match_creation(
    context: &Context,
    interaction: &ComponentInteraction,
    tournaments_service: &H5TournamentsService,
    match_builders: &RwLock<HashMap<u64, RwLock<MatchBuilder>>>,
    game_builders: &RwLock<HashMap<u64, RwLock<GameBuilderContainer>>>,
) -> Result<(), crate::Error> {
    let match_builders_locked = match_builders.read().await;
    if let Some(existing_builder) = match_builders_locked.get(&interaction.message.id.get()) {
        let builder_locked = existing_builder.read().await;
        let first_player = tournaments_service
            .get_participant(
                GetParticipantPayload::default().with_challonge(builder_locked.player.clone()),
            )
            .await?
            .ok_or(crate::Error::from(format!(
                "No participant found with challonge id {}",
                &builder_locked.player
            )))?;
        let opponent_data = serde_json::from_str::<OpponentDataPayload>(
            &builder_locked.selected_opponent.as_ref().unwrap(),
        )?;
        let second_player = tournaments_service
            .get_participant(
                GetParticipantPayload::default().with_challonge(opponent_data.opponent_id.clone()),
            )
            .await?
            .ok_or(crate::Error::from(format!(
                "No participant found with challonge id {}",
                &opponent_data.opponent_id
            )))?;
        let created_match_id = tournaments_service
            .create_match(
                builder_locked.tournament_id,
                interaction.message.id.get(),
                first_player.user,
                second_player.user,
                opponent_data.match_id,
            )
            .await?;

        let heroes = tournaments_service
            .get_heroes(h5_tournaments_api::prelude::ModType::Universe)
            .await?;

        let tournament_data = tournaments_service.get_tournament_data(
            GetTournament::default().with_id(builder_locked.tournament_id)
        ).await?.unwrap();

        let container = GameBuilderContainer {
            match_id: created_match_id,
            tournament_id: builder_locked.tournament_id,
            heroes: heroes,
            current_number: 1,
            use_bargains: tournament_data.with_bargains,
            use_bargains_color: tournament_data.with_bargains_color,
            use_foreign_heroes: tournament_data.with_foreign_heroes,
            game_type: GameType::from(tournament_data.game_type),
            player_nickname: builder_locked.user_nickname.clone(),
            opponent_nickname: opponent_data.nickname,
            builders: Vec::from_iter((1..builder_locked.games_count.unwrap() + 1).map(|n| {
                GameBuilder {
                    number: n,
                    ..Default::default()
                }
            })),
            tournament_state: builder_locked.tournament_state.clone()
        };
        drop(builder_locked);
        drop(match_builders_locked);
        let mut match_builders_to_remove = match_builders.write().await;
        match_builders_to_remove.remove(&interaction.message.id.get());
        drop(match_builders_to_remove);

        let response_message = build_game_message(tournaments_service, &container).await?;
        let mut game_builders_locked = game_builders.write().await;
        game_builders_locked.insert(interaction.message.id.get(), RwLock::new(container));
        drop(game_builders_locked);
        interaction
            .create_response(
                context,
                CreateInteractionResponse::UpdateMessage(response_message),
            )
            .await?;
    }
    Ok(())
}

pub async fn show_bargains_modal(
    interaction: &ComponentInteraction,
    context: &Context,
    game_builders: &RwLock<HashMap<u64, RwLock<GameBuilderContainer>>>
) -> Result<(), crate::Error> {
    let message = interaction.message.id.get();
    let builders_locked = game_builders.read().await;
    if let Some(container) = builders_locked.get(&message) {
        let container_locked = container.read().await;
        let current_game_number = container_locked.current_number;
        let current_game = container_locked
            .builders
            .iter()
            .find(|g| g.number == current_game_number)
            .unwrap();
        interaction.create_response(context, CreateInteractionResponse::Modal(
            CreateModal::new("bargains_input_modal", "Указать размер торга")
                .components(vec![
                    CreateActionRow::InputText(
                        CreateInputText::new(InputTextStyle::Short, "Торг", "bargains_amount_input")
                            .value(current_game.bargains_amount.to_string())
                    )
                ])
        )).await?;
    }
    Ok(())
}

pub async fn switch_to_edition_state(
    interaction: &ComponentInteraction,
    context: &Context,
    tournaments_service: &H5TournamentsService,
    game_builders: &RwLock<HashMap<u64, RwLock<GameBuilderContainer>>>,
    new_state: GameBuilderState,
) -> Result<(), crate::Error> {
    let message = interaction.message.id.get();
    let game_builders_locked = game_builders.read().await;
    if let Some(container) = game_builders_locked.get(&message) {
        let mut container_locked = container.write().await;
        let current_game_number = container_locked.current_number;
        let current_game = container_locked
            .builders
            .iter_mut()
            .find(|g| g.number == current_game_number)
            .unwrap();
        current_game.state = new_state;
        drop(container_locked);
        let response_message =
            build_game_message(tournaments_service, &*container.read().await).await?;
        interaction
            .create_response(
                context,
                CreateInteractionResponse::UpdateMessage(response_message),
            )
            .await?;
    }
    Ok(())
}

pub async fn switch_games(
    interaction: &ComponentInteraction,
    context: &Context,
    tournaments_service: &H5TournamentsService,
    game_builders: &RwLock<HashMap<u64, RwLock<GameBuilderContainer>>>,
    game_change: i32,
) -> Result<(), crate::Error> {
    let message = interaction.message.id.get();
    let game_builders_locked = game_builders.read().await;
    if let Some(container) = game_builders_locked.get(&message) {
        let mut container_locked = container.write().await;
        container_locked.current_number += game_change;
        drop(container_locked);
        let response_message =
            build_game_message(tournaments_service, &*container.read().await).await?;
        interaction
            .create_response(
                context,
                CreateInteractionResponse::UpdateMessage(response_message),
            )
            .await?;
    }
    Ok(())
}

pub async fn generate_final_report_message(
    context: &Context,
    interaction: &ComponentInteraction,
    tournaments_service: &H5TournamentsService,
    challonge_service: &ChallongeService,
    game_builders: &RwLock<HashMap<u64, RwLock<GameBuilderContainer>>>,
) -> Result<(), crate::Error> {
    let message = interaction.message.id.get();
    let game_builders_locked = game_builders.read().await;
    if let Some(container) = game_builders_locked.get(&message) {
        let container_locked = container.read().await;
        let games_count = tournaments_service
            .get_games_of_match_count(container_locked.match_id)
            .await?;
        if games_count == 0 {
            generate_report_final_message(
                context,
                tournaments_service,
                challonge_service,
                &container_locked
            )
            .await?;
            interaction
            .create_response(
                context,
                CreateInteractionResponse::UpdateMessage(
                    CreateInteractionResponseMessage::new()
                        .add_embed(
                            CreateEmbed::new()
                                .title("Отчет успешно создан, можете закрыть это сообщение."),
                        )
                        .components(vec![]),
                ),
            )
            .await?;
            drop(container_locked);
            drop(game_builders_locked);
            let mut builders_to_remove = game_builders.write().await;
            builders_to_remove.remove(&message);
            drop(builders_to_remove);
        } else {
            let match_data = tournaments_service
                .get_match(container_locked.match_id)
                .await?
                .ok_or(crate::Error::from(format!(
                    "No match found with id {}",
                    container_locked.match_id
                )))?;
            if let Some(link) = match_data.report_link {
                let response_message = CreateInteractionResponseMessage::new()
                    .content(format!("Отчет об этом матче уже был создан: {}. Не имеет значения, кто из игроков создал отчет, однако, убедитесь, что он был заполнен корректно вашим оппонентом. Если это не так, обратитесь к организаторам турнира.", &link))
                    .components(vec![]);
                interaction
                    .create_response(
                        context,
                        CreateInteractionResponse::UpdateMessage(response_message),
                    )
                    .await?;
            } else {
                let response_message = CreateInteractionResponseMessage::new()
                    .content("Создание отчета не было завершено корректно. Обратитесь к организаторам турнира.")
                    .components(vec![]);
                interaction
                    .create_response(
                        context,
                        CreateInteractionResponse::UpdateMessage(response_message),
                    )
                    .await?;
            }
        }
    }
    Ok(())
}

async fn generate_report_final_message(
    context: &Context,
    tournaments_service: &H5TournamentsService,
    challonge_service: &ChallongeService,
    container: &RwLockReadGuard<'_, GameBuilderContainer>
) -> Result<(), crate::Error> {
    let tournament_data = tournaments_service
        .get_tournament_data(GetTournament::default().with_id(container.tournament_id))
        .await?
        .unwrap();
    let operator_data = tournaments_service
        .get_operator_data(GetOperatorPayload::default().with_id(tournament_data.operator))
        .await?;
    let output_channel = ChannelId::from(operator_data.generated as u64);
    let match_data = tournaments_service
        .get_match(container.match_id)
        .await?
        .unwrap();
    let first_user = tournaments_service
        .get_user(GetUser::default().with_id(match_data.first_player))
        .await?
        .unwrap();
    let second_user = tournaments_service
        .get_user(GetUser::default().with_id(match_data.second_player))
        .await?
        .unwrap();
    let games = container
        .builders
        .iter()
        .sorted_by_key(|g| g.number)
        .collect::<Vec<&GameBuilder>>();
    let first_player_wins = games
        .iter()
        .filter(|g| g.result == GameResult::FirstPlayerWon)
        .count();
    let second_player_wins = games
        .iter()
        .filter(|g| g.result == GameResult::SecondPlayerWon)
        .count();

    let games_to_insert = games
        .iter()
        .map(|g| create_games_bulk::CreateGameModel {
            match_id: container.match_id,
            first_player_race: g.first_player_race,
            first_player_hero: g.first_player_hero,
            second_player_race: g.second_player_race,
            second_player_hero: g.second_player_hero,
            bargains_color: if g.bargains_color.is_none() { None } else { Some(g.bargains_color.clone().unwrap().into()) },
            bargains_amount: Some(g.bargains_amount),
            result: g.result.clone().into(),
            outcome: Some(g.outcome.clone().into())
        })
        .collect::<Vec<create_games_bulk::CreateGameModel>>();

    tournaments_service
        .create_games_bulk(games_to_insert)
        .await?;

    let mut fields = vec![];
    for game in &games {
        let mut game_string = format!(
            "**{},** _{}_ **{}** **{},** _{}_. ",
            &tournaments_service
                .races
                .iter()
                .find(|r| r.id == game.first_player_race.unwrap())
                .unwrap()
                .name,
            &container
                .heroes
                .iter()
                .find(|h| h.id == game.first_player_hero.unwrap())
                .unwrap()
                .name,
            match game.result {
                builders::types::GameResult::FirstPlayerWon => ">".to_string(),
                builders::types::GameResult::SecondPlayerWon => "<".to_string(),
                _ => "?".to_string(),
            },
            &tournaments_service
                .races
                .iter()
                .find(|r| r.id == game.second_player_race.unwrap())
                .unwrap()
                .name,
            &container
                .heroes
                .iter()
                .find(|h| h.id == game.second_player_hero.unwrap())
                .unwrap()
                .name
        );
        if container.use_bargains {
            let mut bargains_string = String::from("**Торг**: ");
            if container.use_bargains_color {
                match game.bargains_color.as_ref().unwrap() {
                    BargainsColor::NotSelected => bargains_string += &String::from("Неизвестный цвет, "),
                    BargainsColor::BargainsColorBlue => bargains_string += &String::from("Синий, "),
                    BargainsColor::BargainsColorRed => bargains_string += &String::from("Красный, ")
                }
            }
            bargains_string += &game.bargains_amount.to_string();
            game_string += &bargains_string;
        }
        if container.game_type == GameType::Rmg {
            match game.outcome {
                GameOutcome::FinalBattleVictory => game_string += &String::from("\n**Победа в финалке.**"),
                GameOutcome::NeutralsVictory => game_string += &String::from("\n**Победа нейтралов.**"),
                GameOutcome::OpponentSurrender => game_string += &String::from("\n**Признание поражения.**")
            }
        }
        fields.push((
            format!("_Игра {}_", game.number),
            game_string,
            false,
        ))
    }

    fields.push((
        "_Счёт_".to_string(),
        format!("**{} - {}**", first_player_wins, second_player_wins),
        false,
    ));
    let message_builder = CreateMessage::new().add_embed(
        CreateEmbed::new()
            .title(format!(
                "**Турнир {}**, _{}_",
                &tournament_data.name.to_uppercase(),
                if container.tournament_state == ChallongeTournamentState::GroupStagesUnderway {
                    "групповой этап"
                } else {
                    "плей-офф"
                }
            ))
            .description(format!(
                "**{}** _VS_ **{}**",
                &first_user.nickname, &second_user.nickname
            ))
            .fields(fields),
    );

    let created_message = output_channel
        .send_message(context, message_builder)
        .await?;
    let link = created_message.link();
    tournaments_service
        .update_match(container.match_id, link)
        .await?;

    let first_participant = tournaments_service
        .get_participant(
            GetParticipantPayload::default()
                .with_tournament(tournament_data.id)
                .with_user(first_user.id),
        )
        .await?
        .ok_or(crate::Error::from(format!(
            "User {} isn't found in tournament {}",
            &first_user.nickname, &tournament_data.name
        )))?;
    let second_participant = tournaments_service
        .get_participant(
            GetParticipantPayload::default()
                .with_tournament(tournament_data.id)
                .with_user(second_user.id),
        )
        .await?
        .ok_or(crate::Error::from(format!(
            "User {} isn't found in tournament {}",
            &second_user.nickname, &tournament_data.name
        )))?;
    let challonge_match = match_data.challonge;
    let challonge_tournament = tournament_data.challonge_id.unwrap();
    let organizer = tournaments_service
        .get_organizer(GetOrganizerPayload::default().with_id(tournament_data.organizer))
        .await?
        .unwrap();
    let challonge_payload = ChallongeUpdateMatchPayload {
        _type: crate::services::challonge::payloads::ChallongePayloadType::Match,
        attributes: ChallongeUpdateMatchAttributes {
            match_data: vec![
                ChallongeMatchParticipantsData {
                    participant_id: first_participant.challonge.unwrap(),
                    score_set: first_player_wins.to_string(),
                    rank: 1.to_string(),
                    advancing: first_player_wins > second_player_wins,
                },
                ChallongeMatchParticipantsData {
                    participant_id: second_participant.challonge.unwrap(),
                    score_set: second_player_wins.to_string(),
                    rank: 1.to_string(),
                    advancing: first_player_wins < second_player_wins,
                },
            ],
            tie: false,
        },
    };
    challonge_service
        .update_challonge_match(
            &organizer.challonge,
            &challonge_tournament,
            &challonge_match,
            challonge_payload,
        )
        .await?;
    Ok(())
}

pub async fn select_player_race(
    interaction: &ComponentInteraction,
    context: &Context,
    tournaments_service: &H5TournamentsService,
    game_builders: &RwLock<HashMap<u64, RwLock<GameBuilderContainer>>>,
    selected_value: &String,
) -> Result<(), crate::Error> {
    let message = interaction.message.id.get();
    let game_builders_locked = game_builders.read().await;
    if let Some(container) = game_builders_locked.get(&message) {
        let mut container_locked = container.write().await;
        let current_game_number = container_locked.current_number;
        let current_game = container_locked
            .builders
            .iter_mut()
            .find(|g| g.number == current_game_number)
            .unwrap();
        current_game.first_player_race = Some(i64::from_str_radix(selected_value, 10)?);
        drop(container_locked);
        let response_message =
            build_game_message(tournaments_service, &*container.read().await).await?;
        interaction
            .create_response(
                context,
                CreateInteractionResponse::UpdateMessage(response_message),
            )
            .await?;
    }
    Ok(())
}

pub async fn select_player_hero_race(
    interaction: &ComponentInteraction,
    context: &Context,
    tournaments_service: &H5TournamentsService,
    game_builders: &RwLock<HashMap<u64, RwLock<GameBuilderContainer>>>,
    selected_value: &String,
) -> Result<(), crate::Error> {
    let message = interaction.message.id.get();
    let game_builders_locked = game_builders.read().await;
    if let Some(container) = game_builders_locked.get(&message) {
        let mut container_locked = container.write().await;
        let current_game_number = container_locked.current_number;
        let current_game = container_locked
            .builders
            .iter_mut()
            .find(|g| g.number == current_game_number)
            .unwrap();
        let selected_race = i64::from_str_radix(selected_value, 10)?;
        current_game.first_player_hero_race = if selected_race == -1 { None } else { Some(selected_race) };
        drop(container_locked);
        let response_message =
            build_game_message(tournaments_service, &*container.read().await).await?;
        interaction
            .create_response(
                context,
                CreateInteractionResponse::UpdateMessage(response_message),
            )
            .await?;
    }
    Ok(())
}

pub async fn select_opponent_race(
    interaction: &ComponentInteraction,
    context: &Context,
    tournaments_service: &H5TournamentsService,
    game_builders: &RwLock<HashMap<u64, RwLock<GameBuilderContainer>>>,
    selected_value: &String,
) -> Result<(), crate::Error> {
    let message = interaction.message.id.get();
    let game_builders_locked = game_builders.read().await;
    if let Some(container) = game_builders_locked.get(&message) {
        let mut container_locked = container.write().await;
        let current_game_number = container_locked.current_number;
        let current_game = container_locked
            .builders
            .iter_mut()
            .find(|g| g.number == current_game_number)
            .unwrap();
        current_game.second_player_race = Some(i64::from_str_radix(selected_value, 10)?);
        drop(container_locked);
        let response_message =
            build_game_message(tournaments_service, &*container.read().await).await?;
        interaction
            .create_response(
                context,
                CreateInteractionResponse::UpdateMessage(response_message),
            )
            .await?;
    }
    Ok(())
}

pub async fn select_opponent_hero_race(
    interaction: &ComponentInteraction,
    context: &Context,
    tournaments_service: &H5TournamentsService,
    game_builders: &RwLock<HashMap<u64, RwLock<GameBuilderContainer>>>,
    selected_value: &String,
) -> Result<(), crate::Error> {
    let message = interaction.message.id.get();
    let game_builders_locked = game_builders.read().await;
    if let Some(container) = game_builders_locked.get(&message) {
        let mut container_locked = container.write().await;
        let current_game_number = container_locked.current_number;
        let current_game = container_locked
            .builders
            .iter_mut()
            .find(|g| g.number == current_game_number)
            .unwrap();
        let selected_race = i64::from_str_radix(selected_value, 10)?;
        current_game.second_player_hero_race = if selected_race == -1 { None } else { Some(selected_race) };
        drop(container_locked);
        let response_message =
            build_game_message(tournaments_service, &*container.read().await).await?;
        interaction
            .create_response(
                context,
                CreateInteractionResponse::UpdateMessage(response_message),
            )
            .await?;
    }
    Ok(())
}

pub async fn select_player_hero(
    interaction: &ComponentInteraction,
    context: &Context,
    tournaments_service: &H5TournamentsService,
    game_builders: &RwLock<HashMap<u64, RwLock<GameBuilderContainer>>>,
    selected_value: &String,
) -> Result<(), crate::Error> {
    let message = interaction.message.id.get();
    let game_builders_locked = game_builders.read().await;
    if let Some(container) = game_builders_locked.get(&message) {
        let mut container_locked = container.write().await;
        let current_game_number = container_locked.current_number;
        let current_game = container_locked
            .builders
            .iter_mut()
            .find(|g| g.number == current_game_number)
            .unwrap();
        current_game.first_player_hero = Some(i64::from_str_radix(selected_value, 10)?);
        drop(container_locked);
        let response_message =
            build_game_message(tournaments_service, &*container.read().await).await?;
        interaction
            .create_response(
                context,
                CreateInteractionResponse::UpdateMessage(response_message),
            )
            .await?;
    }
    Ok(())
}

pub async fn select_opponent_hero(
    interaction: &ComponentInteraction,
    context: &Context,
    tournaments_service: &H5TournamentsService,
    game_builders: &RwLock<HashMap<u64, RwLock<GameBuilderContainer>>>,
    selected_value: &String,
) -> Result<(), crate::Error> {
    let message = interaction.message.id.get();
    let game_builders_locked = game_builders.read().await;
    if let Some(container) = game_builders_locked.get(&message) {
        let mut container_locked = container.write().await;
        let current_game_number = container_locked.current_number;
        let current_game = container_locked
            .builders
            .iter_mut()
            .find(|g| g.number == current_game_number)
            .unwrap();
        current_game.second_player_hero = Some(i64::from_str_radix(selected_value, 10)?);
        drop(container_locked);
        let response_message =
            build_game_message(tournaments_service, &*container.read().await).await?;
        interaction
            .create_response(
                context,
                CreateInteractionResponse::UpdateMessage(response_message),
            )
            .await?;
    }
    Ok(())
}

pub async fn select_game_result(
    interaction: &ComponentInteraction,
    context: &Context,
    tournaments_service: &H5TournamentsService,
    game_builders: &RwLock<HashMap<u64, RwLock<GameBuilderContainer>>>,
    selected_value: &String,
) -> Result<(), crate::Error> {
    let message = interaction.message.id.get();
    let game_builders_locked = game_builders.read().await;
    if let Some(container) = game_builders_locked.get(&message) {
        let mut container_locked = container.write().await;
        let current_game_number = container_locked.current_number;
        let current_game = container_locked
            .builders
            .iter_mut()
            .find(|g| g.number == current_game_number)
            .unwrap();
        current_game.result =
            GameResult::from_repr(i32::from_str_radix(selected_value, 10)?).unwrap();
        drop(container_locked);
        let response_message =
            build_game_message(tournaments_service, &*container.read().await).await?;
        interaction
            .create_response(
                context,
                CreateInteractionResponse::UpdateMessage(response_message),
            )
            .await?;
    }
    Ok(())
}

pub async fn select_game_outcome(
    interaction: &ComponentInteraction,
    context: &Context,
    tournaments_service: &H5TournamentsService,
    game_builders: &RwLock<HashMap<u64, RwLock<GameBuilderContainer>>>,
    selected_value: &String,
) -> Result<(), crate::Error> {
    let message = interaction.message.id.get();
    let game_builders_locked = game_builders.read().await;
    if let Some(container) = game_builders_locked.get(&message) {
        let mut container_locked = container.write().await;
        let current_game_number = container_locked.current_number;
        let current_game = container_locked
            .builders
            .iter_mut()
            .find(|g| g.number == current_game_number)
            .unwrap();
        current_game.outcome = GameOutcome::from_str(&selected_value)?;
        drop(container_locked);
        let response_message =
            build_game_message(tournaments_service, &*container.read().await).await?;
        interaction
            .create_response(
                context,
                CreateInteractionResponse::UpdateMessage(response_message),
            )
            .await?;
    }
    Ok(())
}

pub async fn process_bargains_modal(
    interaction: &ModalInteraction,
    context: &Context,
    tournaments_service: &H5TournamentsService,
    game_builders: &RwLock<HashMap<u64, RwLock<GameBuilderContainer>>>,
) -> Result<(), crate::Error> {
    let message = &interaction.message.as_ref().unwrap().id.get();
    let game_builders_locked = game_builders.read().await;
    if let Some(container) = game_builders_locked.get(&message) {
        for row in &interaction.data.components {
            for component in &row.components {
                match component {
                    ActionRowComponent::InputText(text) => {
                        if text.custom_id.as_str() == "bargains_amount_input" {
                            let value = i64::from_str_radix(&text.value.as_ref().unwrap(), 10).unwrap();
                            let mut container_locked = container.write().await;
                            let current_game_number = container_locked.current_number;
                            let current_game = container_locked
                                .builders
                                .iter_mut()
                                .find(|g| g.number == current_game_number)
                                .unwrap();
                            current_game.bargains_amount = value;
                            drop(container_locked);
                            let response_message =
                                build_game_message(tournaments_service, &*container.read().await).await?;
                            interaction.create_response(context, CreateInteractionResponse::UpdateMessage(response_message)).await?;
                        }
                    },
                    _=> {}
                }
            }
        }
    }
    Ok(())
}

pub async fn select_bargains_color(
    interaction: &ComponentInteraction,
    context: &Context,
    tournaments_service: &H5TournamentsService,
    game_builders: &RwLock<HashMap<u64, RwLock<GameBuilderContainer>>>,
    selected_value: &String,
) -> Result<(), crate::Error> {
    let message = interaction.message.id.get();
    let game_builders_locked = game_builders.read().await;
    if let Some(container) = game_builders_locked.get(&message) {
        let mut container_locked = container.write().await;
        let current_game_number = container_locked.current_number;
        let current_game = container_locked
            .builders
            .iter_mut()
            .find(|g| g.number == current_game_number)
            .unwrap();
        current_game.bargains_color = Some(BargainsColor::from_str(&selected_value)?);
        drop(container_locked);
        let response_message =
            build_game_message(tournaments_service, &*container.read().await).await?;
        interaction
            .create_response(
                context,
                CreateInteractionResponse::UpdateMessage(response_message),
            )
            .await?;
    }
    Ok(())
}