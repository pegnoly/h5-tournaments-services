use std::{collections::HashMap, fmt::format};

use h5_tournaments_api::prelude::*;
use itertools::Itertools;
use rust_xlsxwriter::Worksheet;

use super::{styles, StatsBuilder};

const GAMES_HISTORY_CELLS: [&'static str; 6] = [
    "Раса игрока",
    "Раса оппонента",
    "Торг игрока",
    "Герой игрока",
    "Герой оппонента",
    "Результат"
];

pub struct PlayersStatsBuilder {

}

impl StatsBuilder for PlayersStatsBuilder {
    fn build(&mut self, data: &crate::utils::StatsGeneratorDataModel, workbook: &mut rust_xlsxwriter::Workbook) {

        let mut players = vec![];

        data.matches_data.iter()
            .for_each(|m| {
                players.push(m.first_player.clone());
                players.push(m.second_player.clone());
            });
        
        let unique_players = players.into_iter().unique().collect::<Vec<String>>();
        println!("Unique players: {:?}", unique_players);
        for player in &unique_players {
            let player_matches = data.matches_data.iter()
                .filter(|m| m.first_player == *player || m.second_player == *player)
                .unique_by(|m| m.id)
                .sorted_by_key(|m| m.message_id)
                .collect::<Vec<&Match>>();

            let worksheet = workbook.add_worksheet().set_name(player).unwrap();
            build_player_stats(player, player_matches, &data.games_data, &data.races_data, &data.heroes_data, &data.bargains_data, worksheet);
        }
    }
}

pub(self) enum GameRes {
    Win,
    Loss
}

fn build_player_stats(
    player: &String, 
    matches: Vec<&Match>, 
    games_data: &Vec<Game>, 
    races_data: &Vec<Race>, 
    heroes_data: &Vec<Hero>,
    bargains_data: &Vec<BargainsColorModel>, 
    worksheet: &mut Worksheet
) {

    worksheet.merge_range(0, 1, 0, 6, "История игр", &styles::TEXT_BOLD_CENTERED).unwrap();

    worksheet.set_column_width(0, 14).unwrap();
    let mut col_offset = 1;
    for cell_name in GAMES_HISTORY_CELLS {
        worksheet.set_column_width(col_offset, 14).unwrap();
        worksheet.write_with_format(1, col_offset, cell_name, &styles::THIN_BORDER_TEXP_WRAP).unwrap();
        col_offset += 1;
    }

    worksheet.write_with_format(1, 0, "VS", &styles::TEXT_CENTER_COLOR_RED).unwrap();

    let mut games_count = 0;
    let mut race_games_count = HashMap::<i32, u16>::new();
    let mut race_games_wins = HashMap::<i32, u16>::new();
    let mut hero_games_count = HashMap::<i32, u16>::new();
    let mut hero_wins_count = HashMap::<i32, u16>::new();

    let mut total_wins_bargains = 0;
    let mut total_loss_bargains = 0;

    for actual_match in matches {
        let games = games_data.iter().filter(|game| game.match_id == actual_match.id).collect::<Vec<&Game>>();
        let opponent = if actual_match.first_player == *player { &actual_match.second_player } else { &actual_match.first_player };
        for game in games {
            games_count += 1;
            worksheet.write_with_format(1 + games_count, 0, opponent, &styles::TEXT_BOLD_CENTERED).unwrap();

            let (player_race, opponent_race) = get_players_races(player, actual_match, game);

            if let Some(count ) = race_games_count.get_mut(&player_race) {
                *count += 1;
            }
            else {
                race_games_count.insert(player_race, 1);
            }

            worksheet.write_with_format(
                1 + games_count, 
                1, 
                get_race_actual_name(races_data, player_race), 
                &styles::THIN_BORDER_TEXP_WRAP)
                .unwrap();
            worksheet.write_with_format(
                1 + games_count, 
                2, 
                get_race_actual_name(races_data, opponent_race), 
                &styles::THIN_BORDER_TEXP_WRAP)
                .unwrap();

            let bargains_amount = get_player_bargains_amount(player, actual_match, game);
            worksheet.write_with_format(1 + games_count, 3, bargains_amount, &styles::THIN_BORDER_TEXP_WRAP).unwrap();

            let (player_hero, opponent_hero) = get_players_heroes(player, actual_match, game);

            if let Some(count ) = hero_games_count.get_mut(&player_hero) {
                *count += 1;
            }
            else {
                hero_games_count.insert(player_hero, 1);
            }

            worksheet.write_with_format(
                1 + games_count, 
                4, 
                get_hero_actual_name(heroes_data, player_hero), 
                &styles::THIN_BORDER_TEXP_WRAP)
                .unwrap();
            worksheet.write_with_format(
                1 + games_count, 
                5, 
                get_hero_actual_name(heroes_data, opponent_hero),
                &styles::THIN_BORDER_TEXP_WRAP)
                .unwrap();

            match get_game_result(player, actual_match, game) {
                GameRes::Win => {
                    worksheet.write_with_format(1 + games_count, 6, "Победа", &styles::BACKGROUND_GREEN).unwrap();
                    total_wins_bargains += bargains_amount as i64;

                    if let Some(count ) = race_games_wins.get_mut(&player_race) {
                        *count += 1;
                    }
                    else {
                        race_games_wins.insert(player_race, 1);
                    }

                    if let Some(count ) = hero_wins_count.get_mut(&player_hero) {
                        *count += 1;
                    }
                    else {
                        hero_wins_count.insert(player_hero, 1);
                    }
                },
                GameRes::Loss => {
                    worksheet.write_with_format(1 + games_count, 6, "Поражение", &styles::BACKGROUND_RED).unwrap();
                    total_loss_bargains += bargains_amount as i64;
                }
            }
        }
    }

    let mut row_offset = 3 + games_count;

    let total_games = race_games_count.iter()
        .map(|g| {
            *g.1
        })
        .sum::<u16>();

    let total_wins = race_games_wins.iter()
        .map(|g| {
            *g.1
        })
        .sum::<u16>();

    let total_losses = total_games - total_wins;

    worksheet.write_with_format(row_offset, 0, "Всего игр", &styles::THIN_BORDER_TEXP_WRAP).unwrap();
    worksheet.write_with_format(row_offset, 1, total_games, &styles::THIN_BORDER_TEXP_WRAP).unwrap();
    worksheet.write_with_format(row_offset + 1, 0, "Общий винрейт", &styles::THIN_BORDER_TEXP_WRAP).unwrap();
    worksheet.write_with_format(row_offset + 1, 1, format!("{:.3}%", total_wins as f64 / total_games as f64 * 100.0), &styles::THIN_BORDER_TEXP_WRAP).unwrap();

    let average_win_bargains = if total_wins == 0 { "Нет побед".to_string() } else { format!("{:.3}", total_wins_bargains as f64 / total_wins as f64) };
    worksheet.write_with_format(row_offset + 2, 0, "Средний торг в победных играх", &styles::THIN_BORDER_TEXP_WRAP).unwrap();
    worksheet.write_with_format(row_offset + 2, 1, average_win_bargains, &styles::THIN_BORDER_TEXP_WRAP).unwrap();

    let average_loss_bargains = if total_losses == 0 { "Нет поражений".to_string() } else { format!("{:.3}", total_loss_bargains as f64 / total_losses as f64) };
    worksheet.write_with_format(row_offset + 3, 0, "Средний торг в проигранных играх", &styles::THIN_BORDER_TEXP_WRAP).unwrap();
    worksheet.write_with_format(row_offset + 3, 1, average_loss_bargains, &styles::THIN_BORDER_TEXP_WRAP).unwrap();
    
    //

    row_offset += 6;

    worksheet.merge_range(row_offset - 1, 0, row_offset - 1, 2, "Выбор рас", &styles::TEXT_BOLD_CENTERED).unwrap();
    worksheet.write_with_format(row_offset, 1, "Всего игр", &styles::THIN_BORDER_TEXP_WRAP).unwrap();
    worksheet.write_with_format(row_offset, 2, "Винрейт", &styles::THIN_BORDER_TEXP_WRAP).unwrap();

    let mut races_count = 0;
    for race_info in race_games_count {
        races_count += 1;
        let winrate = *race_games_wins.get(&race_info.0).unwrap_or(&0) as f64 / race_info.1 as f64 * 100.0;
        worksheet.write_with_format(
            row_offset + races_count, 
            0, 
            get_race_actual_name(races_data, race_info.0), 
            &styles::THIN_BORDER_TEXP_WRAP
        ).unwrap();
        worksheet.write_with_format(row_offset + races_count, 1, race_info.1, &styles::THIN_BORDER_TEXP_WRAP).unwrap();
        worksheet.write_with_format(row_offset + races_count, 2, format!("{:.3}%", winrate), &styles::THIN_BORDER_TEXP_WRAP).unwrap();
    }

    row_offset += 3 + races_count;

    worksheet.merge_range(row_offset - 1, 0, row_offset - 1, 2, "Выбор героев", &styles::TEXT_BOLD_CENTERED).unwrap();
    worksheet.write_with_format(row_offset, 1, "Всего игр", &styles::THIN_BORDER_TEXP_WRAP).unwrap();
    worksheet.write_with_format(row_offset, 2, "Винрейт", &styles::THIN_BORDER_TEXP_WRAP).unwrap();

    let mut heroes_count = 0;
    for hero_info in hero_games_count {
        heroes_count += 1;
        let winrate = *hero_wins_count.get(&hero_info.0).unwrap_or(&0) as f64 / hero_info.1 as f64 * 100.0;
        worksheet.write_with_format(
            row_offset + heroes_count, 
            0, 
            get_hero_actual_name(heroes_data, hero_info.0), 
            &styles::THIN_BORDER_TEXP_WRAP
        ).unwrap();
        worksheet.write_with_format(row_offset + heroes_count, 1, hero_info.1, &styles::THIN_BORDER_TEXP_WRAP).unwrap();
        worksheet.write_with_format(row_offset + heroes_count, 2, format!("{:.3}%", winrate), &styles::THIN_BORDER_TEXP_WRAP).unwrap();
    }
}

fn get_players_races<'a>(player: &'a String, actual_match: &Match, actual_game: &Game) -> (i32, i32) {
    let (player_race_id, opponent_race_id) = if actual_match.first_player == *player { 
        (actual_game.first_player_race, actual_game.second_player_race) 
    }
    else {
        (actual_game.second_player_race, actual_game.first_player_race)
    };

    (player_race_id, opponent_race_id)
}

fn get_player_bargains_amount<'a>(player: &'a String, actual_match: &Match, actual_game: &Game) -> i16 {
    let bargains_amount = if actual_match.first_player == *player { 
        actual_game.bargains_amount
    } else { 
        -1 * actual_game.bargains_amount
    };
    bargains_amount
}

fn get_players_heroes<'a>(player: &'a String, actual_match: &Match, actual_game: &Game) -> (i32, i32) {
    let (player_hero_id, opponent_hero_id) = if actual_match.first_player == *player { 
        (actual_game.first_player_hero, actual_game.second_player_hero) 
    }
    else {
        (actual_game.second_player_hero, actual_game.first_player_hero)
    };

    (player_hero_id, opponent_hero_id)
}

fn get_game_result(player: &String, actual_match: &Match, actual_game: &Game) -> GameRes {
    let res = if actual_match.first_player == *player { if actual_game.result == GameResult::FirstPlayerWon { GameRes::Win } else { GameRes::Loss } } else {
        if actual_game.result == GameResult::SecondPlayerWon { GameRes::Win } else { GameRes::Loss } 
    };

    res
}
 
fn get_hero_actual_name<'a>(heroes_data: &'a Vec<Hero>, hero: i32) -> &'a str {
    if let Some(actual_hero) = heroes_data.iter().find(|h| h.id == hero) {
        &actual_hero.actual_name
    }
    else {
        "Не определено"
    }
}

fn get_race_actual_name<'a>(races_data: &'a Vec<Race>, race: i32) -> &'a str {
    if let Some(actual_race) = races_data.iter().find(|r| r.id == race) {
        &actual_race.actual_name
    }
    else {
        "Не определено"
    }
}