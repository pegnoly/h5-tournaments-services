use poise::serenity_prelude::*;
use uuid::Uuid;
use crate::{
    services::h5_tournaments::service::H5TournamentsService, 
    graphql::queries::{
        get_game_query::{self, GetGameQueryGame}, 
        get_participants::GetParticipantsParticipants
    }, 
    types::payloads::{GetMatch, GetTournament, GetUser}
};

pub async fn initial_build(
    context: &Context,
    api: &H5TournamentsService,
    interaction: &ComponentInteraction,
    _id: &String,
    channel: u64,
    user: u64
) -> Result<(), crate::Error> {

    let tournament_data = api.get_tournament_data(GetTournament::default().with_reports_channel(channel.to_string())).await?.unwrap();
    let operator_data = api.get_operator_data(tournament_data.operator.unwrap()).await?;
    let user_data = api.get_user(GetUser::default().with_discord_id(user.to_string())).await?.unwrap();
    let participant = api.get_participant(tournament_data.id, user_data.id).await?.unwrap();
    let participants = api.get_participants(tournament_data.id, participant.group).await?;
    tracing::info!("Match build started by interaction {}", interaction.id.get());
    api.create_match(tournament_data.id, user_data.id, interaction.id.get()).await?;

    let message_builder = CreateInteractionResponseMessage::new()
        .add_embed(
            CreateEmbed::new()
                .title("Отчет о турнирной игре")
                .description(format!("Турнир **{}** by **{}**", tournament_data.name.to_uppercase(), operator_data.name))
                .fields(
                [
                    (
                        "Автор",
                        format!("{}", &user_data.nickname),
                        false
                    ),
                    (
                        "Стадия",
                        "Групповой этап".to_string(),
                        false
                    ),
                    (
                        "Группа",
                        participant.group.to_string(),
                        true
                    )
                ])
        )
        .select_menu(create_opponent_selector(participants, None, user_data.id))
        .select_menu(create_games_count_selector(5, None))
        .button(CreateButton::new("start_report").label("Начать заполнение отчета").disabled(true))
        .ephemeral(true);

    interaction.create_response(context, poise::serenity_prelude::CreateInteractionResponse::Message(message_builder)).await?;
    Ok(())
}

pub async fn rebuild_initial(match_id: Uuid, api: &H5TournamentsService) -> Result<CreateInteractionResponseMessage, crate::Error> {
    if let Some(match_data) = api.get_match(GetMatch::default().with_id(match_id)).await? {
        tracing::info!("Match data: {:?}", &match_data);
        let tournament_data = api.get_tournament_data(GetTournament::default().with_id(match_data.tournament)).await?.unwrap();
        tracing::info!("Tournament data: {:?}", &tournament_data);
        let operator_data = api.get_operator_data(tournament_data.operator.unwrap()).await?;
        tracing::info!("Operator data: {:?}", &operator_data);
        let user_data = api.get_user(GetUser::default().with_id(match_data.first_player)).await?.unwrap();
        tracing::info!("User data: {:?}", &user_data);
        let participant = api.get_participant(tournament_data.id, user_data.id).await?.unwrap();
        let participants = api.get_participants(tournament_data.id, participant.group).await?;
        let message_builder = CreateInteractionResponseMessage::new()
            .add_embed(
                CreateEmbed::new()
                    .title("Отчет о турнирной игре")
                    .description(format!("Турнир **{}** by **{}**", tournament_data.name.to_uppercase(), operator_data.name))
                    .fields(
                    [
                        (
                            "Автор",
                            format!("{}", &user_data.nickname),
                            false
                        ),
                        (
                            "Стадия",
                            "Групповой этап".to_string(),
                            false
                        ),
                        (
                            "Группа",
                            participant.group.to_string(),
                            true
                        )
                    ])
            )            
            .select_menu(create_opponent_selector(participants, match_data.second_player, user_data.id))
            .select_menu(create_games_count_selector(5, match_data.games_count))
            .button(CreateButton::new("start_report").label("Начать заполнение отчета").disabled(
                if match_data.games_count.is_some() && match_data.second_player.is_some() {
                    false
                } else {
                    true
                }
            ))
            .ephemeral(true);
        Ok(message_builder)
    }
    else {
        Err(crate::Error::from("Failed to rebuild message"))
    }
}

fn create_opponent_selector(users: Vec<GetParticipantsParticipants>, current_selected_opponent: Option<Uuid>, current_user: Uuid) -> CreateSelectMenu {
    let options = users.iter()
        .filter_map(|user| {
            if user.id != current_user {
                Some(
                    CreateSelectMenuOption::new(user.nickname.clone(), user.id.to_string())
                        .default_selection(
                            if current_selected_opponent.is_some() && current_selected_opponent.unwrap() == user.id {
                                true
                            }
                            else {
                                false
                            })
                )
            }
            else {
                None
            }
        })
        .collect::<Vec<CreateSelectMenuOption>>();

    CreateSelectMenu::new("opponent_selector", poise::serenity_prelude::CreateSelectMenuKind::String { options: options })
        .placeholder("Укажите оппонента")
}

fn create_games_count_selector(games_count: i32, selected_value: Option<i64>) -> CreateSelectMenu {
    let options = (1..games_count + 1)
        .map(|number| {
            CreateSelectMenuOption::new(number.to_string(), number.to_string()).default_selection(
                if selected_value.is_some() && selected_value.unwrap() as i32 == number {
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
    _context: &Context,
    api: &H5TournamentsService,
    initial_message: u64
) -> Result<CreateInteractionResponseMessage, crate::Error> {
    if let Some(match_data) = api.get_match(GetMatch::default().with_message_id(initial_message)).await? {
        let first_user = api.get_user(GetUser::default().with_id(match_data.first_player)).await?.unwrap().nickname;
        let second_user = api.get_user(GetUser::default().with_id(match_data.second_player.unwrap())).await?.unwrap().nickname;
        let current_game = match_data.current_game;
        // get current game data of this match
        let game_data = api.get_game(match_data.id, current_game).await?;
        let actual_game = if game_data.is_some() {
            game_data.unwrap()
        }
        else {
            let created_game = api.create_game(match_data.id, current_game).await?;
            GetGameQueryGame {
                first_player_race: created_game.first_player_race,
                first_player_hero: created_game.first_player_hero,
                second_player_race: created_game.second_player_race,
                second_player_hero: created_game.second_player_hero,
                edit_state: Some(get_game_query::GameEditState::PLAYER_DATA),
                bargains_amount: created_game.bargains_amount,
                result: get_game_query::GameResult::NOT_SELECTED
            }
        };
        tracing::info!("Game data: {:?}", &actual_game);
        let description = 
        format!(
            "
                **{},** _{}_ {} **{},** _{}_. **Торг: {}**
            ",
            if actual_game.first_player_race.is_some() { 
                api.races.iter().find(|r| r.id == actual_game.first_player_race.unwrap()).unwrap().name.clone()
            } 
            else { 
                "Неизвестная раса".to_string() 
            },
            if actual_game.first_player_hero.is_some() { 
                api.get_hero(actual_game.first_player_hero.unwrap()).await?.unwrap().name.clone()
            } else { 
                "Неизвестный герой".to_string() 
            },
            match actual_game.result {
                get_game_query::GameResult::NOT_SELECTED => "Неизвестный результат".to_string(),
                get_game_query::GameResult::FIRST_PLAYER_WON => ">".to_string(),
                get_game_query::GameResult::SECOND_PLAYER_WON => "<".to_string(),
                _=> "Неизвестный результат".to_string()
            },
            if actual_game.second_player_race.is_some() { 
                api.races.iter().find(|r| r.id == actual_game.second_player_race.unwrap()).unwrap().name.clone()
            } else { 
                "Неизвестная раса".to_string() 
            },
            if actual_game.second_player_hero.is_some() { 
                api.get_hero(actual_game.second_player_hero.unwrap()).await?.unwrap().name.clone()
            } else { 
                "Неизвестный герой".to_string() 
            },
            actual_game.bargains_amount.to_string()
        );
        let mut core_components = build_core_components(api, actual_game.edit_state.as_ref().unwrap());
        let mut second_row = generate_second_row(api, &actual_game).await;
        core_components.append(&mut second_row);
        core_components.push(CreateActionRow::Buttons(vec![
            CreateButton::new("previous_game_button").label("Предыдущая игра")
                .disabled(
                if match_data.current_game == 1 {
                    true
                } else {
                    false
                }),
            CreateButton::new("next_game_button").label("Следующая игра")
                .disabled(if match_data.current_game == match_data.games_count.unwrap() || !check_game_is_full_built(&actual_game) {
                    true
                } else {
                    false
                }),
            CreateButton::new("submit_report").label("Закончить отчет")
                .disabled(if match_data.current_game != match_data.games_count.unwrap() || !check_game_is_full_built(&actual_game) {
                    true
                } else {
                    false
                }),
        ]));
        Ok(
            CreateInteractionResponseMessage::new()
                .add_embed(CreateEmbed::new()
                    .title(format!("**{}** VS **{}**, **игра {}**\n", &first_user, &second_user, match_data.current_game))
                    .description(description))
                .components(core_components)
            )
    }
    else {
        Err(crate::Error::from("Failed to build game message"))   
    }
}

fn check_game_is_full_built(game: &GetGameQueryGame) -> bool {
    game.first_player_race.is_some() && 
    game.first_player_hero.is_some() && 
    game.second_player_race.is_some() && 
    game.second_player_hero.is_some() &&
    //game.bargains_amount.is_some() &&
    game.result != get_game_query::GameResult::NOT_SELECTED
}

async fn generate_second_row(api: &H5TournamentsService, game_data: &GetGameQueryGame) -> Vec<CreateActionRow> {
    match game_data.edit_state.as_ref().unwrap() {
        get_game_query::GameEditState::PLAYER_DATA => {
            build_player_selector(api, game_data.first_player_race, game_data.first_player_hero).await
        },
        get_game_query::GameEditState::OPPONENT_DATA => {
            build_opponent_selector(api, game_data.second_player_race, game_data.second_player_hero).await
        },
        get_game_query::GameEditState::RESULT_DATA => {
            build_result_selector(&game_data.result)
        },
        _=> {
            vec![]
        }
    }
}

// Main buttons, must be always rendered, style depends on current edit state
fn build_core_components(_api: &H5TournamentsService, edit_state: &get_game_query::GameEditState) -> Vec<CreateActionRow> {
    vec![
        CreateActionRow::Buttons(vec![
            CreateButton::new("player_data_button").label("Указать данные игрока")
                .style(if *edit_state == get_game_query::GameEditState::PLAYER_DATA {
                    ButtonStyle::Success
                } else {
                    ButtonStyle::Primary
                })
                .disabled(if *edit_state == get_game_query::GameEditState::PLAYER_DATA {
                    true
                } else {
                    false
                }),
            CreateButton::new("opponent_data_button").label("Указать данные оппонента")
                .style(if *edit_state == get_game_query::GameEditState::OPPONENT_DATA {
                    ButtonStyle::Success
                } else {
                    ButtonStyle::Primary
                })
                .disabled(if *edit_state == get_game_query::GameEditState::OPPONENT_DATA {
                    true
                } else {
                    false
                }),
            CreateButton::new("bargains_data_button").label("Указать данные о торгах"),
            CreateButton::new("result_data_button").label("Указать результат игры")
                .style(if *edit_state == get_game_query::GameEditState::RESULT_DATA {
                    ButtonStyle::Success
                } else {
                    ButtonStyle::Primary
                })
                .disabled(if *edit_state == get_game_query::GameEditState::RESULT_DATA {
                    true
                } else {
                    false
                })
        ])
    ]
}

async fn build_player_selector(api: &H5TournamentsService, race: Option<i64>, hero: Option<i64>) -> Vec<CreateActionRow> {
    vec![
        CreateActionRow::SelectMenu(
            CreateSelectMenu::new("player_race_selector", poise::serenity_prelude::CreateSelectMenuKind::String { 
                options: api.races.iter().map(|race_new| {
                    CreateSelectMenuOption::new(race_new.name.clone(), race_new.id.to_string())
                        .default_selection(if race.is_some() && race.unwrap() == race_new.id { true } else { false })
                }).collect::<Vec<CreateSelectMenuOption>>() }).placeholder("Выбрать расу игрока")
        ),
        CreateActionRow::SelectMenu(
            CreateSelectMenu::new("player_hero_selector", CreateSelectMenuKind::String { 
                options: build_heroes_list(api, race, hero).await 
            }).disabled(race.is_none()).placeholder("Выбрать героя игрока")
        )
    ]
}

async fn build_opponent_selector(api: &H5TournamentsService, race: Option<i64>, hero: Option<i64>) -> Vec<CreateActionRow> {
    vec![
        CreateActionRow::SelectMenu(
            CreateSelectMenu::new("opponent_race_selector", poise::serenity_prelude::CreateSelectMenuKind::String {                 
                options: api.races.iter().map(|race_new| {
                CreateSelectMenuOption::new(race_new.name.clone(), race_new.id.to_string())
                    .default_selection(if race.is_some() && race.unwrap() == race_new.id { true } else { false })
            }).collect::<Vec<CreateSelectMenuOption>>() }).placeholder("Выбрать расу оппонента")
        ),
        CreateActionRow::SelectMenu(
            CreateSelectMenu::new("opponent_hero_selector", CreateSelectMenuKind::String { 
                options: build_heroes_list(api, race, hero).await 
            }).disabled(race.is_none()).placeholder("Выбрать героя оппонента")
        )
    ]
}

fn build_result_selector(current_result: &get_game_query::GameResult) -> Vec<CreateActionRow> {
    vec![
        CreateActionRow::SelectMenu(
            CreateSelectMenu::new("game_result_selector", poise::serenity_prelude::CreateSelectMenuKind::String { options: vec![
                CreateSelectMenuOption::new("Победа игрока", "1").default_selection(
                    if *current_result == get_game_query::GameResult::FIRST_PLAYER_WON {
                        true
                    } else {
                        false
                    }
                ),
                CreateSelectMenuOption::new("Победа оппонента", "2").default_selection(
                    if *current_result == get_game_query::GameResult::SECOND_PLAYER_WON {
                        true
                    } else {
                        false
                    }
                )
            ]}).placeholder("Указать результат игры")
        )
    ]
}


async fn build_heroes_list(api: &H5TournamentsService, race: Option<i64>, current_hero: Option<i64>) -> Vec<CreateSelectMenuOption> {
    if race.is_none() {
        vec![
            CreateSelectMenuOption::new("Нет героя", "-1")
        ]
    }
    else {
        let heroes = api.get_heroes(race.unwrap()).await.unwrap();
        heroes.iter().map(|hero| {
            CreateSelectMenuOption::new(hero.name.to_string(), hero.id.to_string())
                .default_selection(if current_hero.is_some() && hero.id == current_hero.unwrap() { true } else { false })
        }).collect::<Vec<CreateSelectMenuOption>>()
    }
}