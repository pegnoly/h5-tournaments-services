use std::{collections::HashMap, u32};

use h5_stats_types::{BargainsColorModel, Game, GameResult, GameResultModel, Hero, Match, Race, RaceType};
use itertools::Itertools;
use ordered_float::OrderedFloat;
use rust_xlsxwriter::{Color, DocProperties, Format, Workbook};

pub struct StatsGenerator {
    pub heroes_data: Vec<Hero>,
    pub races_data: Vec<Race>,
    pub games_data: Vec<Game>,
    pub matches_data: Vec<Match>,
    pub bargains_data: Vec<BargainsColorModel>,
    pub results_data: Vec<GameResultModel>,
    pub workbook: Workbook
}

impl StatsGenerator {

    pub fn new() -> Self {
        StatsGenerator {
            heroes_data: vec![],
            races_data: vec![],
            matches_data: vec![],
            games_data: vec![],
            bargains_data: vec![
                BargainsColorModel { id: h5_stats_types::BargainsColor::ColorRed, name: "Красный".to_string() }, 
                BargainsColorModel { id: h5_stats_types::BargainsColor::ColorBlue, name: "Синий".to_string() }
            ],
            results_data: vec![
                GameResultModel { id: h5_stats_types::GameResult::FirstPlayerWon, name: "Победа 1 игрока".to_string() },
                GameResultModel { id: h5_stats_types::GameResult::SecondPlayerWon, name: "Победа 2 игрока".to_string() }
            ],
            workbook: Workbook::new()
        }
    }

    pub fn process(&mut self) {
        let properties = DocProperties::new()
            .set_author("Gerter")
            //.set_title(format!("{} tournament statistics", &tournament_name))
            .set_company("Universe");
        self.workbook.set_properties(&properties);
        self.build_race_pairs_wl_stats();
    }

    pub fn save(&mut self) {
        let path = std::env::current_exe().unwrap().parent().unwrap().join("test.xlsx");
        self.workbook.save(path).unwrap();
    }

    fn build_race_pairs_wl_stats(&mut self) {
        let worksheet = self.workbook.add_worksheet().set_name("Общая статистика по расам").unwrap();

        let vs_format = Format::new()
            .set_align(rust_xlsxwriter::FormatAlign::VerticalCenter)
            .set_align(rust_xlsxwriter::FormatAlign::Center)
            .set_background_color(Color::Red);
        worksheet.merge_range(0, 0, 1, 0, "VS", &vs_format).unwrap();

        let thin_cell_format = Format::new().set_border(rust_xlsxwriter::FormatBorder::Thin);
        let thin_center_cell_format = Format::new()
            .set_border(rust_xlsxwriter::FormatBorder::Thin)
            .set_align(rust_xlsxwriter::FormatAlign::Center);

        let races_names = self.races_data.iter().map(|r| r.actual_name.clone()).collect::<Vec<String>>();
        let longest_string = races_names.iter().max_by_key(|x| x.len()).unwrap();
        let width = longest_string.chars().count();

        worksheet
            .set_column_width(0, (width + 1) as f64).unwrap();

        let mut total_wins_by_race = HashMap::from([
            (RaceType::Heaven, 0), (RaceType::Inferno, 0), (RaceType::Necropolis, 0), (RaceType::Preserve, 0),
            (RaceType::Dungeon, 0), (RaceType::Academy, 0), (RaceType::Fortress, 0), (RaceType::Stronghold, 0)
        ]);

        let mut total_losses_by_race = HashMap::from([
            (RaceType::Heaven, 0), (RaceType::Inferno, 0), (RaceType::Necropolis, 0), (RaceType::Preserve, 0),
            (RaceType::Dungeon, 0), (RaceType::Academy, 0), (RaceType::Fortress, 0), (RaceType::Stronghold, 0)
        ]);

        for race in &self.races_data {
            match race.id {
                RaceType::NotDetected => {},
                _=> {
                    worksheet.write_with_format(1 + (race.id as u32), 0, &race.actual_name, &thin_cell_format).unwrap();
                    let col_offset = (race.id as u16) * 2 - 1;
                    worksheet.merge_range(0, col_offset, 0, col_offset + 1, &race.actual_name, &thin_center_cell_format).unwrap();
                    worksheet.set_column_width(col_offset, (width as f64) / 1.5).unwrap();
                    worksheet.set_column_width(col_offset + 1, (width as f64) / 1.5).unwrap();
                    worksheet.write_with_format(1, col_offset, "Побед", &thin_center_cell_format).unwrap();
                    worksheet.write_with_format(1, col_offset + 1, "Поражений", &thin_center_cell_format).unwrap();

                    for opponent_race in &self.races_data {
                        match opponent_race.id {
                            RaceType::NotDetected => {},
                            _=> {
                                let row_offset = (opponent_race.id as u32) + 1;
                                if race.id != opponent_race.id {
                                    // check for games where either 1-2 pair is current race-opponent(and 1st won) or 1-2 is opponent-race(and 2nd won)
                                    let wins = self.games_data.iter().filter(|game| {
                                            (game.first_player_race == opponent_race.id && game.second_player_race == race.id && game.result == GameResult::FirstPlayerWon) ||
                                            (game.first_player_race == race.id && game.second_player_race == opponent_race.id && game.result == GameResult::SecondPlayerWon)
                                        })
                                        .unique_by(|g| g.id)
                                        .collect::<Vec<&Game>>()
                                        .len();

                                    *total_wins_by_race.get_mut(&opponent_race.id).unwrap() += wins;

                                    let losses = self.games_data.iter().filter(|game| {
                                            (game.first_player_race == opponent_race.id && game.second_player_race == race.id && game.result == GameResult::SecondPlayerWon) ||
                                            (game.first_player_race == race.id && game.second_player_race == opponent_race.id && game.result == GameResult::FirstPlayerWon)
                                        })
                                        .unique_by(|g| g.id)
                                        .collect::<Vec<&Game>>()
                                        .len();

                                    *total_losses_by_race.get_mut(&opponent_race.id).unwrap() += losses;

                                    worksheet.write_with_format(row_offset, col_offset, wins as u32, &thin_cell_format).unwrap();
                                    worksheet.write_with_format(row_offset, col_offset + 1, losses as u32, &thin_cell_format).unwrap();
                                }
                                else {
                                    worksheet.set_cell_format(row_offset, col_offset, &Format::new().set_background_color(Color::Black)).unwrap();
                                    worksheet.set_cell_format(row_offset, col_offset + 1, &Format::new().set_background_color(Color::Black)).unwrap();
                                }
                            }
                        }
                    }
                }
            }
        }

        // TOTAL GAMES & WINRATE BY RACE 
        worksheet
            .write_with_format(0, 17, "Всего игр", &thin_center_cell_format).unwrap()
            .set_cell_format(1, 17, &Format::new().set_background_color(Color::Silver)).unwrap();

        worksheet.merge_range(11, 0, 11, 1, "Общий винрейт", &thin_center_cell_format).unwrap();

        let races_total_games = self.races_data.iter()
            .map(|r| {
                (r.id, *total_losses_by_race.get(&r.id).unwrap() + *total_wins_by_race.get(&r.id).unwrap())
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

        let races_winrates = self.races_data.iter()
            .map(|r| {
                (r.id, (*total_wins_by_race.get(&r.id).unwrap() as f32) / (*races_total_games.get(&r.id).unwrap() as f32) * 100.0)
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
        
        for race in &self.races_data {
            match race.id {
                RaceType::NotDetected => {},
                _=> {
                    let row_offset = 1 + (race.id as u32);
                    worksheet.write_with_format(row_offset, 17, *races_total_games.get(&race.id).unwrap() as u32, &thin_cell_format).unwrap();
                    let row_offset = 11 + (race.id as u32);
                    worksheet.write_with_format(row_offset, 0, &race.actual_name, &thin_cell_format).unwrap();
                    worksheet.write_with_format(row_offset, 1, &format!("{}%", *races_winrates.get(&race.id).unwrap()), &thin_cell_format).unwrap();
                }
            }
        }

        worksheet
            .set_cell_format(1 + (*most_played_race as u32), 17, &Format::from(&thin_cell_format).set_background_color(Color::Green)).unwrap()
            .set_cell_format(1 + (*least_played_race as u32), 17, &Format::from(&thin_cell_format).set_background_color(Color::Red)).unwrap()
            .set_cell_format(11 + (*race_with_most_winrate as u32), 1, &Format::from(&thin_cell_format).set_background_color(Color::Green)).unwrap()
            .set_cell_format(11 + (*race_with_least_winrate as u32), 1, &Format::from(&thin_cell_format).set_background_color(Color::Red)).unwrap();

        // PAIRS GAMES COUNT & WINRATE
        
        let mut most_played_pair_row = 0;
        let mut most_played_pair_col = 0;
        
        let mut least_played_pair_row = 0;
        let mut least_played_pair_col = 0;

        let mut most_player_pair_games = u32::MIN;
        let mut least_player_pair_games = u32::MAX;

        for race in &self.races_data {
            match race.id {
                RaceType::NotDetected => {},
                _=> {
                    let col_offset = race.id as u16;
                    let row_offset = 22 + (race.id as u32);
                    worksheet.write_with_format(row_offset, 0, &race.actual_name, &thin_cell_format).unwrap();
                    worksheet.write_with_format(22, col_offset, &race.actual_name, &Format::from(&thin_center_cell_format).set_text_wrap()).unwrap();

                    // for opponent_race in &self.races_data {
                    //     match opponent_race.id {
                    //         RaceType::NotDetected => {},
                    //         _=> {
                    //             let pair_wins = self.games_data.iter()
                    //                 .filter(|game| {

                    //                 })
                    //         }
                    //     }
                    // }
                }
            }
        }
    }
}