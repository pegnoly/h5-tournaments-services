use std::{collections::HashMap, str::FromStr};

use crate::{
    builders::types::GameBuilderState, graphql::queries::get_heroes_query::GetHeroesQueryHeroesNewHeroesEntities, services::{
        challonge::{service::ChallongeService, types::{ChallongeMatchData, ChallongeTournamentState}},
        h5_tournaments::{
            payloads::{GetOrganizerPayload, GetParticipantPayload},
            service::H5TournamentsService,
        },
    }, types::payloads::{GetTournament, GetUser}
};
use poise::serenity_prelude::*;
use tokio::sync::RwLock;
use super::types::{
    BargainsColor, GameBuilder, GameBuilderContainer, GameOutcome, GameResult, GameType, MatchBuilder, OpponentDataPayload, OpponentsData
};

/// Invoked when user starts to create report.
/// This function collects all data related to this uses in a context of this tournament.
/// Checks state of tournament and decides to allow user to start report.
pub async fn collect_match_creation_data(
    context: &Context,
    interaction: &ComponentInteraction,
    tournaments_service: &H5TournamentsService,
    challonge_service: &ChallongeService,
    match_builders: &RwLock<HashMap<u64, RwLock<MatchBuilder>>>,
) -> Result<(), crate::Error> {
    interaction.defer_ephemeral(context).await?;
    let tournament_data = tournaments_service
        .get_tournament_data(
            GetTournament::default()
                .with_reports_channel(interaction.channel.as_ref().unwrap().id.to_string()),
        )
        .await?
        .unwrap();
    let organizer = tournaments_service
        .get_organizer(GetOrganizerPayload::default().with_id(tournament_data.organizer))
        .await?
        .unwrap();
    let challonge_tournament = challonge_service.get_challonge_tournament(
        &organizer.challonge, tournament_data.challonge_id.as_ref().unwrap()).await?;
    let state = ChallongeTournamentState::from_str(&challonge_tournament.attributes.state)?;
    if state == ChallongeTournamentState::Pending {
        interaction.create_response(context, CreateInteractionResponse::Message(
            CreateInteractionResponseMessage::new()
                .ephemeral(true)
                .content("Нет возможности создавать отчеты - турнир еще не стартовал")   
        )).await?;
    } else {
        let user_data = tournaments_service
            .get_user(GetUser::default().with_discord_id(interaction.user.id.get().to_string()))
            .await?
            .unwrap();
        let participant = tournaments_service
            .get_participant(
                GetParticipantPayload::default()
                    .with_tournament(tournament_data.id)
                    .with_user(user_data.id),
            )
            .await?
            .ok_or(crate::Error::from(format!(
                "User {} isn't found in tournament {}",
                &user_data.nickname, &tournament_data.name
            )))?;
        let challonge_participants = challonge_service
            .get_participants(
                &organizer.challonge,
                tournament_data.challonge_id.as_ref().unwrap(),
            )
            .await?;
        let matches = challonge_service
            .get_open_matches_for_participant(
                &organizer.challonge,
                tournament_data.challonge_id.as_ref().unwrap(),
                //participant.challonge.as_ref().unwrap().clone()
            )
            .await?;
        let matches_sorted = matches
            .iter()
            .filter(|m| {
                m.attributes.points_by_participant[0].participant_id.to_string() == *participant.challonge.as_ref().unwrap()
                    || m.attributes.points_by_participant[1].participant_id.to_string() == *participant.challonge.as_ref().unwrap()
            })
            .collect::<Vec<&ChallongeMatchData>>();
        if matches_sorted.len() < 1 {
            interaction.create_followup(context, CreateInteractionResponseFollowup::new()
                .ephemeral(true)
                .content("Для вас нет открытых матчей на этом турнире")
            ).await?;
        } else {
            let opponents_data = matches_sorted
                .iter()
                .map(|m| {
                    if m.attributes.points_by_participant[0].participant_id.to_string() != *participant.challonge.as_ref().unwrap() {
                        let opponent = challonge_participants
                            .iter()
                            .find(|p| p.id == m.attributes.points_by_participant[0].participant_id.to_string())
                            .unwrap();
                        OpponentsData {
                            nickname: opponent.attributes.name.clone(),
                            challonge_data: serde_json::to_string(&OpponentDataPayload {
                                nickname: opponent.attributes.name.clone(),
                                opponent_id: opponent.id.clone(),
                                match_id: m.id.clone(),
                            })
                            .unwrap(),
                        }
                    } else {
                        let opponent = challonge_participants
                            .iter()
                            .find(|p| p.id == m.attributes.points_by_participant[1].participant_id.to_string())
                            .unwrap();
                        OpponentsData {
                            nickname: opponent.attributes.name.clone(),
                            challonge_data: serde_json::to_string(&OpponentDataPayload {
                                nickname: opponent.attributes.name.clone(),
                                opponent_id: opponent.id.clone(),
                                match_id: m.id.clone(),
                            })
                            .unwrap(),
                        }
                    }
                })
                .collect::<Vec<OpponentsData>>();
            
            tracing::info!("Opponents data: {:?}", &opponents_data);

            let match_builder = MatchBuilder {
                opponents: opponents_data,
                selected_opponent: None,
                player: participant.challonge.as_ref().unwrap().clone(),
                games_count: None,
                user_nickname: user_data.nickname,
                tournament_name: tournament_data.name,
                tournament_id: tournament_data.id,
                tournament_state: ChallongeTournamentState::from_str(&challonge_tournament.attributes.state)?
            };

            tracing::info!("Match builder: {:?}", &match_builder);

            let response_message = build_initial_match_creation_interface(&match_builder).await?;
            tracing::info!("Response message: {:?}", &response_message);
            // interaction
            //     .create_response(
            //         context,
            //         CreateInteractionResponse::Message(response_message),
            //     )
            //     .await?;
            let message_new = interaction.create_followup(context, response_message).await?;
            let mut message_builders_locked = match_builders.write().await;
            message_builders_locked.insert(
                message_new.id.get(),
                tokio::sync::RwLock::new(match_builder),
            );
            drop(message_builders_locked); // heh 3
        }
    }
    Ok(())
}

pub async fn build_initial_match_creation_interface(
    builder: &MatchBuilder,
) -> Result<CreateInteractionResponseFollowup, crate::Error> {
    Ok(CreateInteractionResponseFollowup::new()
        .ephemeral(true)
        .add_embed(
            CreateEmbed::new()
                .title("Отчет о турнирной игре")
                .description(format!(
                    "Турнир **{}**",
                    builder.tournament_name.to_uppercase()
                ))
                .fields([
                    ("Автор", format!("{}", builder.user_nickname), false),
                    ("Стадия", if builder.tournament_state == ChallongeTournamentState::GroupStagesUnderway { 
                        "Групповой этап".to_string() 
                    } else {
                        "Плей-офф".to_string()
                    } , false),
                ]),
        )
        .select_menu(build_opponent_selector(&builder).await)
        .select_menu(build_games_count_selector(2, 5, &builder).await)
        .button(
            CreateButton::new("start_report")
                .label("Начать заполнение отчета")
                .disabled(
                    if builder.games_count.is_some() && builder.selected_opponent.is_some() {
                        false
                    } else {
                        true
                    },
                ),
        ))
}

pub async fn build_match_creation_interface(
    builder: &MatchBuilder,
) -> Result<CreateInteractionResponseMessage, crate::Error> {
    Ok(CreateInteractionResponseMessage::new()
        .ephemeral(true)
        .add_embed(
            CreateEmbed::new()
                .title("Отчет о турнирной игре")
                .description(format!(
                    "Турнир **{}**",
                    builder.tournament_name.to_uppercase()
                ))
                .fields([
                    ("Автор", format!("{}", builder.user_nickname), false),
                    ("Стадия", if builder.tournament_state == ChallongeTournamentState::GroupStagesUnderway { 
                        "Групповой этап".to_string() 
                    } else {
                        "Плей-офф".to_string()
                    } , false),
                ]),
        )
        .select_menu(build_opponent_selector(&builder).await)
        .select_menu(build_games_count_selector(2, 5, &builder).await)
        .button(
            CreateButton::new("start_report")
                .label("Начать заполнение отчета")
                .disabled(
                    if builder.games_count.is_some() && builder.selected_opponent.is_some() {
                        false
                    } else {
                        true
                    },
                ),
        ))
}

async fn build_opponent_selector(match_builder: &MatchBuilder) -> CreateSelectMenu {
    CreateSelectMenu::new(
        "opponent_selector",
        CreateSelectMenuKind::String {
            options: Vec::from_iter(match_builder.opponents.iter().map(|m| {
                CreateSelectMenuOption::new(m.nickname.clone(), m.challonge_data.clone())
                    .default_selection(
                        match_builder.selected_opponent.is_some()
                            && m.challonge_data
                                == *match_builder.selected_opponent.as_ref().unwrap(),
                    )
            })),
        },
    )
    .placeholder("Укажите своего оппонента")
}

async fn build_games_count_selector(
    min_games_count: i32,
    max_games_count: i32,
    match_builder: &MatchBuilder,
) -> CreateSelectMenu {
    let options = (min_games_count..max_games_count + 1)
        .map(|number| {
            CreateSelectMenuOption::new(number.to_string(), number.to_string()).default_selection(
                if match_builder.games_count.is_some()
                    && match_builder.games_count.unwrap() == number
                {
                    true
                } else {
                    false
                },
            )
        })
        .collect::<Vec<CreateSelectMenuOption>>();

    CreateSelectMenu::new(
        "games_count_selector",
        poise::serenity_prelude::CreateSelectMenuKind::String { options: options },
    )
    .placeholder("Укажите число игр")
}

pub async fn build_game_message(
    tournaments_service: &H5TournamentsService,
    game_builder_container: &GameBuilderContainer,
) -> Result<CreateInteractionResponseMessage, crate::Error> {
    let game_data = game_builder_container
        .builders
        .iter()
        .find(|g| g.number == game_builder_container.current_number)
        .unwrap();

    //tracing::info!("Heroes data: {:?}", game_builder_container.heroes);

    let mut description = format!(
        "
            **{},** _{}_ {} **{},** _{}_.
        ",
        if game_data.first_player_race.is_some() {
            tournaments_service
                .races
                .iter()
                .find(|r| r.id == game_data.first_player_race.unwrap())
                .unwrap()
                .name
                .clone()
        } else {
            "Неизвестная фракция".to_string()
        },
        if game_data.first_player_hero.is_some() {
            game_builder_container
                .heroes
                .iter()
                .find(|h| h.id == game_data.first_player_hero.unwrap())
                .unwrap()
                .name
                .clone()
        } else {
            "Неизвестный герой".to_string()
        },
        match game_data.result {
            GameResult::NotSelected => "Неизвестный результат".to_string(),
            GameResult::FirstPlayerWon => ">".to_string(),
            GameResult::SecondPlayerWon => "<".to_string()
        },
        if game_data.second_player_race.is_some() {
            tournaments_service
                .races
                .iter()
                .find(|r| r.id == game_data.second_player_race.unwrap())
                .unwrap()
                .name
                .clone()
        } else {
            "Неизвестная фракция".to_string()
        },
        if game_data.second_player_hero.is_some() {
            game_builder_container
                .heroes
                .iter()
                .find(|h| h.id == game_data.second_player_hero.unwrap())
                .unwrap()
                .name
                .clone()
        } else {
            "Неизвестный герой".to_string()
        }
    );

    if game_builder_container.use_bargains {
        let mut bargains_string = String::from("Торг: ");
        if game_builder_container.use_bargains_color {
            if game_data.bargains_color.is_none() {
                bargains_string += &String::from("Неизвестный цвет, ");
            } else {
                match game_data.bargains_color.as_ref().unwrap() {
                    BargainsColor::NotSelected => bargains_string += &String::from("Неизвестный цвет, "),
                    BargainsColor::BargainsColorBlue => bargains_string += &String::from("Синий, "),
                    BargainsColor::BargainsColorRed => bargains_string += &String::from("Красный, ")
                }
            }
        }
        bargains_string += &game_data.bargains_amount.to_string();
        description += &bargains_string;
    }

    let mut content = String::new();
    let mut core_components = build_core_components(game_data, game_builder_container);
    let mut second_row =
        generate_second_row(tournaments_service, game_builder_container, game_data, &mut content).await;
    core_components.append(&mut second_row);
    core_components.push(CreateActionRow::Buttons(vec![
        CreateButton::new("previous_game_button")
            .label("Предыдущая игра")
            .disabled(if game_builder_container.current_number == 1 {
                true
            } else {
                false
            }),
        CreateButton::new("next_game_button")
            .label("Следующая игра")
            .disabled(
                if game_builder_container.current_number
                    == game_builder_container.builders.len() as i32
                    || !check_game_is_full_built(game_data, game_builder_container)
                {
                    true
                } else {
                    false
                },
            ),
        CreateButton::new("submit_report")
            .label("Закончить отчет")
            .disabled(
                if game_builder_container.current_number
                    != game_builder_container.builders.len() as i32
                    || !check_game_is_full_built(game_data, game_builder_container)
                {
                    true
                } else {
                    false
                },
            ),
    ]));
    Ok(CreateInteractionResponseMessage::new()
        .ephemeral(true)
        .embed(
            CreateEmbed::new()
                .title(format!(
                    "**{}** VS **{}**. Игра **{}**",
                    &game_builder_container.player_nickname,
                    &game_builder_container.opponent_nickname,
                    game_builder_container.current_number
                ))
                .description(description),
        )
        .content(content)
        .components(core_components))
}

fn check_game_is_full_built(game: &GameBuilder, container: &GameBuilderContainer) -> bool {
    let bargains_color_condition = if !container.use_bargains_color { 
        true 
    } else {
        if game.bargains_color.is_some() {
            true
        } else {
            false
        }
    };

    game.first_player_race.is_some() && 
    game.first_player_hero.is_some() && 
    game.second_player_race.is_some() && 
    game.second_player_hero.is_some() &&
    game.result != GameResult::NotSelected &&
    bargains_color_condition
}

/// Builds interface based on current state of report fill process
async fn generate_second_row(
    tournaments_service: &H5TournamentsService,
    game_builder_container: &GameBuilderContainer,
    game_data: &GameBuilder,
    content: &mut String
) -> Vec<CreateActionRow> {
    match game_data.state {
        GameBuilderState::PlayerData => {
            build_player_data_selector(
                tournaments_service,
                game_builder_container,
                game_data.first_player_race,
                game_data.first_player_hero_race,
                game_data.first_player_hero,
                content
            )
            .await
        }
        GameBuilderState::OpponentData => {
            build_opponent_data_selector(
                tournaments_service,
                &game_builder_container,
                game_data.second_player_race,
                game_data.second_player_hero_race,
                game_data.second_player_hero,
                content
            )
            .await
        }
        GameBuilderState::ResultData => build_result_selector(game_data, game_builder_container, content),
        GameBuilderState::BargainsData => build_bargains_data_interface(game_data, game_builder_container, content),
        _ => {
            vec![]
        }
    }
}

// Main buttons, must be always rendered, style depends on current edit state
fn build_core_components(game_builder: &GameBuilder, container: &GameBuilderContainer) -> Vec<CreateActionRow> {
    let mut buttons = vec![];
    buttons.push(
        CreateButton::new("player_data_button")
            .label("Указать данные игрока")
            .style(if game_builder.state == GameBuilderState::PlayerData {
                ButtonStyle::Success
            } else {
                ButtonStyle::Primary
            })
            .disabled(if game_builder.state == GameBuilderState::PlayerData {
                true
            } else {
                false
            })
    );
    buttons.push(        
        CreateButton::new("opponent_data_button")
            .label("Указать данные оппонента")
            .style(if game_builder.state == GameBuilderState::OpponentData {
                ButtonStyle::Success
            } else {
                ButtonStyle::Primary
            })
            .disabled(if game_builder.state == GameBuilderState::OpponentData {
                true
            } else {
                false
            })
    );
    if container.use_bargains {
        buttons.push(        
            CreateButton::new("bargains_data_button")
                .label("Указать данные о торгах")
                .style(if game_builder.state == GameBuilderState::BargainsData {
                    ButtonStyle::Success
                } else {
                    ButtonStyle::Primary
                })
                .disabled(if game_builder.state == GameBuilderState::BargainsData {
                    true
                } else {
                    false
                })
        );
    }
    buttons.push(
        CreateButton::new("result_data_button")
            .label("Указать результат игры")
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
    );
    vec![CreateActionRow::Buttons(buttons)]
}

/// Creates selectors of race and hero for user who fills the report.
/// If foreign heroes can be used in tournament, additional selector will be created.
async fn build_player_data_selector(
    tournaments_service: &H5TournamentsService,
    container: &GameBuilderContainer,
    race: Option<i64>,
    hero_race: Option<i64>,
    hero: Option<i64>,
    content: &mut String
) -> Vec<CreateActionRow> {
    let mut rows = vec![];
    rows.push(
        CreateActionRow::SelectMenu(
            CreateSelectMenu::new(
                "player_race_selector",
                poise::serenity_prelude::CreateSelectMenuKind::String {
                    options: tournaments_service
                        .races
                        .iter()
                        .map(|race_new| {
                            CreateSelectMenuOption::new(
                                race_new.name.clone(),
                                race_new.id.to_string(),
                            )
                            .default_selection(
                                if race.is_some() && race.unwrap() == race_new.id {
                                    true
                                } else {
                                    false
                                },
                            )
                        })
                        .collect::<Vec<CreateSelectMenuOption>>(),
                },
            )
            .placeholder("Выбрать расу игрока"),
        )
    );
    if container.use_foreign_heroes {
        let mut options = vec![
            CreateSelectMenuOption::new("Использовался родной герой", "-1")
                .default_selection(hero_race.is_none())
        ];
        options.append(&mut tournaments_service
            .races
            .iter()
            .map(|race_new| {
                CreateSelectMenuOption::new(
                    race_new.name.clone(),
                    race_new.id.to_string(),
                )
                .default_selection(
                    if hero_race.is_some() && hero_race.unwrap() == race_new.id {
                        true
                    } else {
                        false
                    },
                )
            })
            .collect::<Vec<CreateSelectMenuOption>>());
        rows.push(
            CreateActionRow::SelectMenu(
                CreateSelectMenu::new(
                    "player_hero_race_selector",
                    poise::serenity_prelude::CreateSelectMenuKind::String {
                        options: options
                    },
                )
                .disabled(race.is_none())
                .placeholder("Выбрать фракцию героя игрока"),
            )
        );
    }
    rows.push(
        CreateActionRow::SelectMenu(
            CreateSelectMenu::new(
                "player_hero_selector",
                CreateSelectMenuKind::String {
                    options: build_heroes_list(&container.heroes, race, hero_race, hero).await,
                },
            )
            .disabled(race.is_none())
            .placeholder("Выбрать героя игрока"),
        )
    );

    if container.use_foreign_heroes {
        *content += "Используйте второй селектор **ТОЛЬКО** в том случае, если играли неродным для фракции героем(например, эльфом за Академию). В противном случае оставляйте этот элемент пустым."
    }

    rows
}

async fn build_opponent_data_selector(
    tournaments_service: &H5TournamentsService,
    container: &GameBuilderContainer,
    race: Option<i64>,
    hero_race: Option<i64>,
    hero: Option<i64>,
    content: &mut String
) -> Vec<CreateActionRow> {
    let mut rows = vec![];
    rows.push(
        CreateActionRow::SelectMenu(
            CreateSelectMenu::new(
                "opponent_race_selector",
                poise::serenity_prelude::CreateSelectMenuKind::String {
                    options: tournaments_service
                        .races
                        .iter()
                        .map(|race_new| {
                            CreateSelectMenuOption::new(
                                race_new.name.clone(),
                                race_new.id.to_string(),
                            )
                            .default_selection(
                                if race.is_some() && race.unwrap() == race_new.id {
                                    true
                                } else {
                                    false
                                },
                            )
                        })
                        .collect::<Vec<CreateSelectMenuOption>>(),
                },
            )
            .placeholder("Выбрать фракцию оппонента"),
        )
    );
    if container.use_foreign_heroes {
        let mut options = vec![
            CreateSelectMenuOption::new("Использовался родной герой", "-1")
                .default_selection(hero_race.is_none())
        ];
        options.append(&mut tournaments_service
            .races
            .iter()
            .map(|race_new| {
                CreateSelectMenuOption::new(
                    race_new.name.clone(),
                    race_new.id.to_string(),
                )
                .default_selection(
                    if hero_race.is_some() && hero_race.unwrap() == race_new.id {
                        true
                    } else {
                        false
                    },
                )
            })
            .collect::<Vec<CreateSelectMenuOption>>());
        rows.push(
            CreateActionRow::SelectMenu(
                CreateSelectMenu::new(
                    "opponent_hero_race_selector",
                    poise::serenity_prelude::CreateSelectMenuKind::String {
                        options: options
                    },
                )
                .disabled(race.is_none())
                .placeholder("Выбрать фракцию героя оппонента"),
            )
        );
    }
    rows.push(
        CreateActionRow::SelectMenu(
            CreateSelectMenu::new(
                "opponent_hero_selector",
                CreateSelectMenuKind::String {
                    options: build_heroes_list(&container.heroes, race, hero_race, hero).await,
                },
            )
            .disabled(race.is_none())
            .placeholder("Выбрать героя оппонента"),
        )
    );

    if container.use_foreign_heroes {
        *content += "Используйте второй селектор **ТОЛЬКО** в том случае, если оппонент играл неродным для фракции героем(например, эльфом за Академию). В противном случае оставляйте этот элемент пустым."
    }

    rows
}

fn build_result_selector(game: &GameBuilder, container: &GameBuilderContainer, content: &mut String) -> Vec<CreateActionRow> {
    let mut rows = vec![];
    rows.push(CreateActionRow::SelectMenu(
        CreateSelectMenu::new(
            "game_result_selector",
            poise::serenity_prelude::CreateSelectMenuKind::String {
                options: vec![
                    CreateSelectMenuOption::new("Победа игрока", "1").default_selection(
                        if game.result == GameResult::FirstPlayerWon {
                            true
                        } else {
                            false
                        },
                    ),
                    CreateSelectMenuOption::new("Победа оппонента", "2").default_selection(
                        if game.result == GameResult::SecondPlayerWon {
                            true
                        } else {
                            false
                        },
                    ),
                ],
            },
        )
        .placeholder("Указать результат игры"),
    ));
    if container.game_type == GameType::Rmg {
        rows.push(CreateActionRow::SelectMenu(
            CreateSelectMenu::new(
                "game_outcome_selector", 
            CreateSelectMenuKind::String { options: vec![
                CreateSelectMenuOption::new("Победа нейтралов", GameOutcome::NeutralsVictory.to_string())
                    .default_selection(game.outcome == GameOutcome::NeutralsVictory),
                CreateSelectMenuOption::new("Оппонент сдался", GameOutcome::OpponentSurrender.to_string())
                    .default_selection(game.outcome == GameOutcome::OpponentSurrender)
            ] })
            .placeholder("Укажите точную причину победы")
        ));
    }

    if container.game_type == GameType::Rmg {
        *content += "Используйте второй селектор **ТОЛЬКО** при условии, что игра завершилась **НЕ** победой в финальной битве. Если вы или оппонент сдались в ходе финалки, это все равно считается победой в ней."
    }

    rows
}

async fn build_heroes_list(
    heroes: &Vec<GetHeroesQueryHeroesNewHeroesEntities>,
    race: Option<i64>,
    player_hero_race: Option<i64>,
    current_hero: Option<i64>
) -> Vec<CreateSelectMenuOption> {
    if race.is_none() && player_hero_race.is_none() {
        vec![CreateSelectMenuOption::new("Нет героя", "-1")]
    } else {
        heroes
            .iter()
            .filter_map(|hero| {
                if (race.is_some() && player_hero_race.is_none() && hero.race == race.unwrap()) || (player_hero_race.is_some() && hero.race == player_hero_race.unwrap()) {
                    Some(
                        CreateSelectMenuOption::new(
                            hero.name.to_string(),
                            hero.id.to_string(),
                        )
                        .default_selection(
                            if current_hero.is_some() && hero.id == current_hero.unwrap() {
                                true
                            } else {
                                false
                            },
                        ),
                    )
                } else {
                    None
                }
            })
            .collect::<Vec<CreateSelectMenuOption>>()
    }
}

fn build_bargains_data_interface(
    game: &GameBuilder,
    container: &GameBuilderContainer,
    content: &mut String
) -> Vec<CreateActionRow> {
    let mut rows = vec![];
    rows.push(
        CreateActionRow::Buttons(vec![
            CreateButton::new("bargains_amount_button").style(ButtonStyle::Success).label("Укажите размер торга")
        ])
    );
    if container.use_bargains_color {
        rows.push(
            CreateActionRow::SelectMenu(
                CreateSelectMenu::new("bargains_color_selector", CreateSelectMenuKind::String { options: vec![
                    CreateSelectMenuOption::new("Красный", BargainsColor::BargainsColorRed.to_string())
                        .default_selection(game.bargains_color.is_some() && *game.bargains_color.as_ref().unwrap() == BargainsColor::BargainsColorRed),
                    CreateSelectMenuOption::new("Синий", BargainsColor::BargainsColorBlue.to_string())
                        .default_selection(game.bargains_color.is_some() && *game.bargains_color.as_ref().unwrap() == BargainsColor::BargainsColorBlue),    
                ] })
                .placeholder("Укажите цвет, на котором ВЫ играли")
            )
        );
    }

    *content += "Указывайте размер торга именно с вашей стороны, **НЕЗАВИСИМО** от результата игры. То есть, если вы играли с -5000 по золоту, всегда указывайте это число, неважно, выиграли или проиграли.\n";
    if container.use_bargains_color {
        *content += "Указывайте именно тот цвет, на котором **ВЫ** играли, независимо от того, за какой цвет шел торг."
    }
    rows
}