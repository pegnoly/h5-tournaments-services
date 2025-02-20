use std::collections::HashMap;

use h5_tournaments_api::prelude::Hero;
use poise::serenity_prelude::*;
use tokio::sync::RwLock;
use uuid::Uuid;
use crate::{
    builders::types::GameBuilderState, services::{challonge::{service::ChallongeService, types::ChallongeMatchData}, h5_tournaments::{payloads::GetOrganizerPayload, service::H5TournamentsService}}, types::payloads::{GetMatch, GetTournament, GetUser}
};

use super::types::{GameBuilder, GameBuilderContainer, GameResult, MatchBuilder, OpponentDataPayload, OpponentsData};

pub async fn collect_match_creation_data(
    context: &Context,
    interaction: &ComponentInteraction,
    tournaments_service: &H5TournamentsService,
    challonge_service: &ChallongeService,
    match_builders: &RwLock<HashMap<u64, RwLock<MatchBuilder>>>
) -> Result<(), crate::Error> {
    let tournament_data = tournaments_service
        .get_tournament_data(GetTournament::default().with_reports_channel(interaction.channel.as_ref().unwrap().id.to_string())).await?.unwrap();
    let organizer = tournaments_service
        .get_organizer(GetOrganizerPayload::default().with_id(tournament_data.organizer.unwrap())).await?.unwrap();
    let user_data = tournaments_service.get_user(
        GetUser::default().with_discord_id(interaction.user.id.get().to_string())
    ).await?.unwrap();
    let participant = tournaments_service.get_participant(Some(tournament_data.id), Some(user_data.id), None).await?.unwrap();
    let challonge_participants = challonge_service
        .get_participants(&organizer.challonge, tournament_data.challonge_id.as_ref().unwrap()).await?;
    let matches =  challonge_service.get_open_matches_for_participant(
        &organizer.challonge, 
        tournament_data.challonge_id.as_ref().unwrap(), 
        //participant.challonge.as_ref().unwrap().clone()
    ).await?;

    let matches_sorted = matches.iter()
        .filter(|m| {
            m.relationships.player1.data.id == *participant.challonge.as_ref().unwrap() || 
            m.relationships.player2.data.id == *participant.challonge.as_ref().unwrap()
        })
        .collect::<Vec<&ChallongeMatchData>>();

    let opponents_data = matches_sorted.iter()
        .map(|m| {
            if m.relationships.player1.data.id != *participant.challonge.as_ref().unwrap() {
                let opponent = challonge_participants.iter().find(|p| {
                    p.id == m.relationships.player1.data.id
                }).unwrap();
                OpponentsData {
                    nickname: opponent.attributes.name.clone(), 
                    challonge_data: serde_json::to_string(&OpponentDataPayload {
                        nickname: opponent.attributes.name.clone(),
                        opponent_id: opponent.id.clone(),
                        match_id: m.id.clone()
                    }).unwrap()
                }
            } else {
                let opponent = challonge_participants.iter().find(|p| {
                    p.id == m.relationships.player2.data.id
                }).unwrap();
                OpponentsData {
                    nickname: opponent.attributes.name.clone(), 
                    challonge_data: serde_json::to_string(&OpponentDataPayload {
                        nickname: opponent.attributes.name.clone(),
                        opponent_id: opponent.id.clone(),
                        match_id: m.id.clone()
                    }).unwrap()
                }
            }
        })
        .collect::<Vec<OpponentsData>>();

    let match_builder = MatchBuilder {
        opponents: opponents_data,
        selected_opponent: None,
        player: participant.challonge.as_ref().unwrap().clone(),
        games_count: None,
        user_nickname: user_data.nickname,
        tournament_name: tournament_data.name,
        tournament_id: tournament_data.id
    };

    let response_message = build_match_creation_interface(&match_builder).await?;
    interaction.create_response(context, CreateInteractionResponse::Message(response_message)).await?;
    let message_new = interaction.get_response(context).await?;
    let mut message_builders_locked = match_builders.write().await;
    message_builders_locked.insert(message_new.id.get(), tokio::sync::RwLock::new(match_builder));
    drop(message_builders_locked); // heh 3
    Ok(())
}

pub async fn build_match_creation_interface(
    builder: &MatchBuilder
) -> Result<CreateInteractionResponseMessage, crate::Error> {
    Ok(CreateInteractionResponseMessage::new()
        .ephemeral(true)
        .add_embed(
            CreateEmbed::new()
                .title("Отчет о турнирной игре")
                .description(format!("Турнир **{}**", builder.tournament_name.to_uppercase()))
                .fields([
                    (
                        "Автор",
                        format!("{}", builder.user_nickname),
                        false
                    ),
                    (
                        "Стадия",
                        "Групповой этап".to_string(),
                        false
                    )
                ]
            )
        )
        .select_menu(build_opponent_selector(&builder).await)
        .select_menu(build_games_count_selector(5, &builder).await)
        .button(CreateButton::new("start_report").label("Начать заполнение отчета").disabled(
            if builder.games_count.is_some() && builder.selected_opponent.is_some() {
                false
            } else {
                true
            }
        ))
    )
}

async fn build_opponent_selector(match_builder: &MatchBuilder) -> CreateSelectMenu {
    CreateSelectMenu::new("opponent_selector", CreateSelectMenuKind::String { options: Vec::from_iter(
        match_builder.opponents.iter()
            .map(|m| {
                CreateSelectMenuOption::new(m.nickname.clone(), m.challonge_data.clone())
                    .default_selection(match_builder.selected_opponent.is_some() && m.challonge_data == *match_builder.selected_opponent.as_ref().unwrap())
            })
        )
    })
    .placeholder("Укажите своего оппонента")
}

async fn build_games_count_selector(games_count: i32, match_builder: &MatchBuilder) -> CreateSelectMenu {
    let options = (1..games_count + 1)
        .map(|number| {
            CreateSelectMenuOption::new(number.to_string(), number.to_string()).default_selection(
                if match_builder.games_count.is_some() && match_builder.games_count.unwrap() == number {
                    true
                }
                else {
                    false
                }
            )
        })
        .collect::<Vec<CreateSelectMenuOption>>();

    CreateSelectMenu::new("games_count_selector", poise::serenity_prelude::CreateSelectMenuKind::String { options: options })
        .placeholder("Укажите число игр")
}

pub async fn build_game_message(
    tournaments_service: &H5TournamentsService,
    game_builder_container: &GameBuilderContainer
) -> Result<CreateInteractionResponseMessage, crate::Error> {
    let game_data = game_builder_container.builders.iter()
        .find(|g| g.number == game_builder_container.current_number)
        .unwrap();

    //tracing::info!("Heroes data: {:?}", game_builder_container.heroes);

    let description = format!(
        "
            **{},** _{}_ {} **{},** _{}_.
        ",
        if game_data.first_player_race.is_some() { 
            tournaments_service.races.iter().find(|r| r.id == game_data.first_player_race.unwrap()).unwrap().name.clone()
        } 
        else { 
            "Неизвестная раса".to_string() 
        },
        if game_data.first_player_hero.is_some() { 
            game_builder_container.heroes.iter().find(|h| h.id == game_data.first_player_hero.unwrap() as i32).unwrap().actual_name.clone()
        } else { 
            "Неизвестный герой".to_string() 
        },
        match game_data.result {
            GameResult::NotSelected => "Неизвестный результат".to_string(),
            GameResult::FirstPlayerWon => ">".to_string(),
            GameResult::SecondPlayerWon => "<".to_string(),
            _=> "Неизвестный результат".to_string()
        },
        if game_data.second_player_race.is_some() { 
            tournaments_service.races.iter().find(|r| r.id == game_data.second_player_race.unwrap()).unwrap().name.clone()
        } else { 
            "Неизвестная раса".to_string() 
        },
        if game_data.second_player_hero.is_some() { 
            game_builder_container.heroes.iter().find(|h| h.id == game_data.second_player_hero.unwrap() as i32).unwrap().actual_name.clone()
        } else { 
            "Неизвестный герой".to_string() 
        },
        //game_data.bargains_amount.to_string()
    );
    
    let mut core_components = build_core_components(game_data);
    let mut second_row = generate_second_row(tournaments_service, game_builder_container, game_data).await;
    core_components.append(&mut second_row);
    core_components.push(CreateActionRow::Buttons(vec![
        CreateButton::new("previous_game_button").label("Предыдущая игра")
            .disabled(
            if game_builder_container.current_number == 1 {
                true
            } else {
                false
            }),
        CreateButton::new("next_game_button").label("Следующая игра")
            .disabled(if game_builder_container.current_number == game_builder_container.builders.len() as i32 || !check_game_is_full_built(game_data) {
                true
            } else {
                false
            }),
        CreateButton::new("submit_report").label("Закончить отчет")
            .disabled(if game_builder_container.current_number != game_builder_container.builders.len() as i32 || !check_game_is_full_built(game_data) {
                true
            } else {
                false
            }),
    ]));
    Ok(
        CreateInteractionResponseMessage::new()
            .ephemeral(true)
            .embed(CreateEmbed::new()
                .title(format!("**{}** VS **{}**. Игра **{}**", &game_builder_container.player_nickname, &game_builder_container.opponent_nickname, game_builder_container.current_number))
                .description(description))
            .components(core_components)
    )
}

fn check_game_is_full_built(game: &GameBuilder) -> bool {
    game.first_player_race.is_some() && 
    game.first_player_hero.is_some() && 
    game.second_player_race.is_some() && 
    game.second_player_hero.is_some() &&
    //game.bargains_amount.is_some() &&
    game.result != GameResult::NotSelected
}

async fn generate_second_row(
    tournaments_service: &H5TournamentsService,
    game_builder_container: &GameBuilderContainer,
    game_data: &GameBuilder
) -> Vec<CreateActionRow> {
    match game_data.state {
        GameBuilderState::PlayerData => {
            build_player_data_selector(tournaments_service, &game_builder_container.heroes, game_data.first_player_race, game_data.first_player_hero).await
        },
        GameBuilderState::OpponentData => {
            build_opponent_data_selector(tournaments_service, &game_builder_container.heroes, game_data.second_player_race, game_data.second_player_hero).await
        },
        GameBuilderState::ResultData => {
            build_result_selector(&game_data.result)
        },
        _=> {
            vec![]
        }
    }
}

// Main buttons, must be always rendered, style depends on current edit state
fn build_core_components(
    game_builder: &GameBuilder    
) -> Vec<CreateActionRow> {
    vec![
        CreateActionRow::Buttons(vec![
            CreateButton::new("player_data_button").label("Указать данные игрока")
                .style(if game_builder.state == GameBuilderState::PlayerData {
                    ButtonStyle::Success
                } else {
                    ButtonStyle::Primary
                })
                .disabled(if game_builder.state == GameBuilderState::PlayerData {
                    true
                } else {
                    false
                }),
            CreateButton::new("opponent_data_button").label("Указать данные оппонента")
                .style(if game_builder.state == GameBuilderState::OpponentData {
                    ButtonStyle::Success
                } else {
                    ButtonStyle::Primary
                })
                .disabled(if game_builder.state == GameBuilderState::OpponentData {
                    true
                } else {
                    false
                }),
            //CreateButton::new("bargains_data_button").label("Указать данные о торгах"),
            CreateButton::new("result_data_button").label("Указать результат игры")
                .style(if game_builder.state == GameBuilderState::ResultData {
                    ButtonStyle::Success
                } else {
                    ButtonStyle::Primary
                })
                .disabled(if game_builder.state == GameBuilderState::ResultData {
                    true
                } else {
                    false
                })
        ])
    ]
}

async fn build_player_data_selector(
    tournaments_service: &H5TournamentsService,
    heroes: &Vec<Hero>,
    race: Option<i64>, 
    hero: Option<i64>
) -> Vec<CreateActionRow> {
    tracing::info!("Building player data, race: {:?}", race);
    vec![
        CreateActionRow::SelectMenu(
            CreateSelectMenu::new("player_race_selector", poise::serenity_prelude::CreateSelectMenuKind::String { 
                options: tournaments_service.races.iter().map(|race_new| {
                    CreateSelectMenuOption::new(race_new.name.clone(), race_new.id.to_string())
                        .default_selection(if race.is_some() && race.unwrap() == race_new.id { true } else { false })
                }).collect::<Vec<CreateSelectMenuOption>>() }).placeholder("Выбрать расу игрока")
        ),
        CreateActionRow::SelectMenu(
            CreateSelectMenu::new("player_hero_selector", CreateSelectMenuKind::String { 
                options: build_heroes_list(heroes, race, hero).await 
            }).disabled(race.is_none()).placeholder("Выбрать героя игрока")
        )
    ]
}

async fn build_opponent_data_selector(
    tournaments_service: &H5TournamentsService, 
    heroes: &Vec<Hero>,
    race: Option<i64>, 
    hero: Option<i64>
) -> Vec<CreateActionRow> {
    vec![
        CreateActionRow::SelectMenu(
            CreateSelectMenu::new("opponent_race_selector", poise::serenity_prelude::CreateSelectMenuKind::String {                 
                options: tournaments_service.races.iter().map(|race_new| {
                CreateSelectMenuOption::new(race_new.name.clone(), race_new.id.to_string())
                    .default_selection(if race.is_some() && race.unwrap() == race_new.id { true } else { false })
            }).collect::<Vec<CreateSelectMenuOption>>() }).placeholder("Выбрать расу оппонента")
        ),
        CreateActionRow::SelectMenu(
            CreateSelectMenu::new("opponent_hero_selector", CreateSelectMenuKind::String { 
                options: build_heroes_list(heroes, race, hero).await 
            }).disabled(race.is_none()).placeholder("Выбрать героя оппонента")
        )
    ]
}

fn build_result_selector(current_result: &GameResult) -> Vec<CreateActionRow> {
    vec![
        CreateActionRow::SelectMenu(
            CreateSelectMenu::new("game_result_selector", poise::serenity_prelude::CreateSelectMenuKind::String { options: vec![
                CreateSelectMenuOption::new("Победа игрока", "1").default_selection(
                    if *current_result == GameResult::FirstPlayerWon {
                        true
                    } else {
                        false
                    }
                ),
                CreateSelectMenuOption::new("Победа оппонента", "2").default_selection(
                    if *current_result == GameResult::SecondPlayerWon {
                        true
                    } else {
                        false
                    }
                )
            ]}).placeholder("Указать результат игры")
        )
    ]
}


async fn build_heroes_list(heroes: &Vec<Hero>, race: Option<i64>, current_hero: Option<i64>) -> Vec<CreateSelectMenuOption> {
    if race.is_none() {
        vec![
            CreateSelectMenuOption::new("Нет героя", "-1")
        ]
    }
    else {
        heroes.iter().filter_map(|hero| {
            if race.is_some() && hero.race == race.unwrap() as i32 {
                Some(CreateSelectMenuOption::new(hero.actual_name.to_string(), hero.id.to_string())
                    .default_selection(if current_hero.is_some() && hero.id == current_hero.unwrap() as i32 { true } else { false })) 
            } else {
                None
            }
        }).collect::<Vec<CreateSelectMenuOption>>()
    }
}