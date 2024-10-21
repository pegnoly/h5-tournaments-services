use std::collections::HashMap;

use h5_stats_types::{Game, GameResult, Race, RaceType};
use itertools::Itertools;
use ordered_float::OrderedFloat;
use rust_xlsxwriter::{Format, Workbook, Worksheet};
use strum::IntoEnumIterator;

use crate::utils::StatsGeneratorDataModel;

use super::{styles, StatsBuilder};

pub struct PairStatsBuilder {
    pub wins_by_race: HashMap<RaceType, HashMap<RaceType, usize>>,
    pub losses_by_race: HashMap<RaceType, HashMap<RaceType, usize>>
}

impl PairStatsBuilder {
    pub fn new() -> Self {
        PairStatsBuilder { 
            wins_by_race: HashMap::from_iter(
                RaceType::iter().map(|r| {
                    (r, HashMap::from_iter(RaceType::iter().map(|r2| {
                            (r2, 0)
                    })))
                })), 

            losses_by_race: HashMap::from_iter(
                RaceType::iter().map(|r| {
                    (r, HashMap::from_iter(RaceType::iter().map(|r2| {
                            (r2, 0)
                    })))
                })) 
        }
    }
}

impl StatsBuilder for PairStatsBuilder {
    fn build(&mut self, data: &StatsGeneratorDataModel, workbook: &mut Workbook) {
        let worksheet = workbook.add_worksheet().set_name("Общая статистика по расам").unwrap();
        build_pairs_win_loss_stats(self, &data.races_data, &data.games_data, worksheet);
        build_total_games_and_winrates(self, worksheet, &data.races_data);
        build_match_ups_games_and_winrates(worksheet, &data.races_data, self);
    }
}

// utils

fn build_pairs_win_loss_stats(builder: &mut PairStatsBuilder, races_data: &Vec<Race>, games_data: &Vec<Game>, worksheet: &mut Worksheet) {
    let width = races_data.iter()
        .map(|r| r.actual_name.clone())
        .collect::<Vec<String>>().iter()
        .max_by_key(|x| x.len()).unwrap()
        .chars().count();

    worksheet.merge_range(0, 0, 1, 0, "VS", &styles::TEXT_CENTER_COLOR_RED).unwrap();

    worksheet
        .set_column_width(0, (width + 1) as f64).unwrap();


    for race in races_data {
        match race.id {
            RaceType::NotDetected => {},
            _=> {
                worksheet.write_with_format(1 + (race.id as u32), 0, &race.actual_name, &styles::THIN_BORDER_TEXP_WRAP).unwrap();
                let col_offset = (race.id as u16) * 2 - 1;
                worksheet.merge_range(
                    0, 
                    col_offset, 
                    0, 
                    col_offset + 1,
                     &race.actual_name, 
                     &styles::THIN_BORDER_TEXT_CENTER
                    ).unwrap();
                worksheet.set_column_width(col_offset, (width as f64) / 1.5).unwrap();
                worksheet.set_column_width(col_offset + 1, (width as f64) / 1.5).unwrap();
                worksheet.write_with_format(1, col_offset, "Побед", &styles::THIN_BORDER_TEXT_CENTER).unwrap();
                worksheet.write_with_format(1, col_offset + 1, "Поражений", &styles::THIN_BORDER_TEXT_CENTER).unwrap();

                for opponent_race in races_data {
                    match opponent_race.id {
                        RaceType::NotDetected => {},
                        _=> {
                            let row_offset = (opponent_race.id as u32) + 1;
                            if race.id != opponent_race.id {
                                // check for games where either 1-2 pair is current race-opponent(and 1st won) or 1-2 is opponent-race(and 2nd won)
                                let wins = games_data.iter().filter(|game| {
                                        (game.first_player_race == opponent_race.id && game.second_player_race == race.id && game.result == GameResult::FirstPlayerWon) ||
                                        (game.first_player_race == race.id && game.second_player_race == opponent_race.id && game.result == GameResult::SecondPlayerWon)
                                    })
                                    .unique_by(|g| g.id)
                                    .collect::<Vec<&Game>>()
                                    .len();

                                *builder.wins_by_race.get_mut(&opponent_race.id).unwrap().get_mut(&race.id).unwrap() = wins;

                                let losses = games_data.iter().filter(|game| {
                                        (game.first_player_race == opponent_race.id && game.second_player_race == race.id && game.result == GameResult::SecondPlayerWon) ||
                                        (game.first_player_race == race.id && game.second_player_race == opponent_race.id && game.result == GameResult::FirstPlayerWon)
                                    })
                                    .unique_by(|g| g.id)
                                    .collect::<Vec<&Game>>()
                                    .len();

                                *builder.losses_by_race.get_mut(&opponent_race.id).unwrap().get_mut(&race.id).unwrap() = losses;

                                worksheet.write_with_format(row_offset, col_offset, wins as u32, &styles::THIN_BORDER_TEXP_WRAP).unwrap();
                                worksheet.write_with_format(row_offset, col_offset + 1, losses as u32, &styles::THIN_BORDER_TEXP_WRAP).unwrap();
                            }
                            else {
                                worksheet.set_cell_format(row_offset, col_offset, &styles::BACKGROUND_BLACK).unwrap();
                                worksheet.set_cell_format(row_offset, col_offset + 1, &styles::BACKGROUND_BLACK).unwrap();
                            }
                        }
                    }
                }
            }
        }
    }
}

fn build_total_games_and_winrates(builder: &mut PairStatsBuilder, worksheet: &mut Worksheet,  races_data: &Vec<Race>) {
    worksheet
        .write_with_format(0, 17, "Всего игр", &styles::THIN_BORDER_TEXT_CENTER).unwrap()
        .set_cell_format(1, 17, &styles::BACKGROUND_SILVER).unwrap();

    worksheet.merge_range(11, 0, 11, 1, "Общий винрейт", &styles::THIN_BORDER_TEXT_CENTER).unwrap();

    let races_total_games = races_data.iter()
        .map(|r| {
            (r.id, calc_games(builder.losses_by_race.get(&r.id).unwrap()) + calc_games(builder.wins_by_race.get(&r.id).unwrap()))
        })
        .collect::<HashMap<RaceType, usize>>();

        let least_played_race = races_total_games.iter()
        .min_by_key(|r| r.1)
        .unwrap()
        .0;

    let most_played_race = races_total_games.iter()
        .max_by_key(|r| r.1)
        .unwrap()
        .0;

    let races_winrates = races_data.iter()
        .map(|r| {
            (r.id, (calc_games(builder.wins_by_race.get(&r.id).unwrap()) as f32) / (*races_total_games.get(&r.id).unwrap() as f32) * 100.0)
        })
        .collect::<HashMap<RaceType, f32>>();

    let race_with_least_winrate = races_winrates.iter()
        .min_by_key(|r| OrderedFloat(*r.1))
        .unwrap()
        .0;

    let race_with_most_winrate = races_winrates.iter()
        .max_by_key(|r| OrderedFloat(*r.1))
        .unwrap()
        .0;

    for race in races_data {
        match race.id {
            RaceType::NotDetected => {},
            _=> {
                let row_offset = 1 + (race.id as u32);
                worksheet.write_with_format(row_offset, 17, *races_total_games.get(&race.id).unwrap() as u32, &styles::THIN_BORDER_TEXP_WRAP).unwrap();
                let row_offset = 11 + (race.id as u32);
                worksheet.write_with_format(row_offset, 0, &race.actual_name, &styles::THIN_BORDER_TEXP_WRAP).unwrap();
                worksheet.write_with_format(row_offset, 1, &format!("{:.3}%", *races_winrates.get(&race.id).unwrap()), &styles::THIN_BORDER_TEXP_WRAP).unwrap();
            }
        }
    }

    worksheet
        .set_cell_format(1 + (*most_played_race as u32), 17, &styles::BACKGROUND_GREEN).unwrap()
        .set_cell_format(1 + (*least_played_race as u32), 17, &styles::BACKGROUND_RED).unwrap()
        .set_cell_format(11 + (*race_with_most_winrate as u32), 1, &styles::BACKGROUND_GREEN).unwrap()
        .set_cell_format(11 + (*race_with_least_winrate as u32), 1, &styles::BACKGROUND_RED).unwrap();
}


fn build_match_ups_games_and_winrates(worksheet: &mut Worksheet, races_data: &Vec<Race>, builder: &mut PairStatsBuilder) {
    let mut most_played_pair_first = RaceType::NotDetected;
    let mut most_played_pair_second = RaceType::NotDetected;
    
    let mut least_played_pair_first = RaceType::NotDetected;
    let mut least_played_pair_second = RaceType::NotDetected;

    let mut most_played_pair_games = u32::MIN;
    let mut least_played_pair_games = u32::MAX;

    worksheet.merge_range(21, 3, 21, 6, "Число игр по матчапам", 
        &Format::new().set_align(rust_xlsxwriter::FormatAlign::Center).set_align(rust_xlsxwriter::FormatAlign::CenterAcross).set_bold()).unwrap();

    worksheet.merge_range(33, 3, 33, 6, "Винрейты матчапов", 
    &Format::new().set_align(rust_xlsxwriter::FormatAlign::Center).set_align(rust_xlsxwriter::FormatAlign::CenterAcross).set_bold()).unwrap();

    for race in races_data {
        match race.id {
            RaceType::NotDetected => {},
            _=> {
                let col_offset = race.id as u16;
                let games_row_offset = 23 + (race.id as u32);
                let winrate_row_offset = 35 + (race.id as u32);

                worksheet.write_with_format(games_row_offset, 0, &race.actual_name, &styles::THIN_BORDER_TEXP_WRAP).unwrap();
                worksheet.write_with_format(23, col_offset, &race.actual_name, &styles::THIN_BORDER_TEXP_WRAP).unwrap();

                worksheet.write_with_format(winrate_row_offset, 0, &race.actual_name, &styles::THIN_BORDER_TEXP_WRAP).unwrap();
                worksheet.write_with_format(35, col_offset, &race.actual_name, &styles::THIN_BORDER_TEXP_WRAP).unwrap();

                for opponent_race in races_data {
                    match opponent_race.id {
                        RaceType::NotDetected => {},
                        _=> {
                            let col_offset = opponent_race.id as u16;
                            if race.id == opponent_race.id {
                                worksheet.set_cell_format(games_row_offset, col_offset, &styles::BACKGROUND_BLACK).unwrap();
                                worksheet.set_cell_format(winrate_row_offset, col_offset, &styles::BACKGROUND_BLACK).unwrap();
                            }
                            else {
                                let pair_wins = *builder.wins_by_race.get(&race.id).unwrap().get(&opponent_race.id).unwrap();
                                let pair_losses = *builder.losses_by_race.get(&race.id).unwrap().get(&opponent_race.id).unwrap();
                                let total_pair_games = (pair_wins + pair_losses) as u32;

                                let pair_winrate = (pair_wins as f32) / (total_pair_games as f32) * 100.0;
                                worksheet.write_with_format(games_row_offset, col_offset, total_pair_games as u32, &styles::THIN_BORDER_TEXP_WRAP).unwrap();   
                                worksheet.write_with_format(
                                    winrate_row_offset, 
                                    col_offset, 
                                    format!("{:.3}%", pair_winrate as f32), 
                                    &styles::THIN_BORDER_TEXP_WRAP)
                                    .unwrap();

                                if total_pair_games > most_played_pair_games {
                                    most_played_pair_games = total_pair_games;
                                    most_played_pair_first = opponent_race.id;
                                    most_played_pair_second = race.id;
                                }

                                if total_pair_games < least_played_pair_games {
                                    least_played_pair_games = total_pair_games;
                                    least_played_pair_first = opponent_race.id;
                                    least_played_pair_second = race.id;
                                }
                            }
                        }
                    }
                }
            }
        }
    }
    // MOST - LEAST PLAYED PAIRS
    worksheet.set_cell_format(23 + (most_played_pair_first as u32), most_played_pair_second as u16, &styles::BACKGROUND_GREEN).unwrap();
    worksheet.set_cell_format(23 + (most_played_pair_second as u32), most_played_pair_first as u16, &styles::BACKGROUND_GREEN).unwrap();
    worksheet.set_cell_format(23 + (least_played_pair_first as u32), least_played_pair_second as u16, &styles::BACKGROUND_RED).unwrap();
    worksheet.set_cell_format(23 + (least_played_pair_second as u32), least_played_pair_first as u16, &styles::BACKGROUND_RED).unwrap();
}

fn calc_games(data: &HashMap<RaceType, usize>) -> usize {
    data.into_iter().map(|d| *d.1).sum()
}