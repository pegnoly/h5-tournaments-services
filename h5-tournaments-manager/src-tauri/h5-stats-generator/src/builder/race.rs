use std::collections::HashMap;

use h5_stats_types::{Game, GameResult, Hero, HeroType, Race, RaceType};
use itertools::Itertools;
use rust_xlsxwriter::{Color, Format, Worksheet};
use strum::IntoEnumIterator;

use super::{styles, StatsBuilder};

const BARGAINS_CELLS_NAMES: [&'static str; 12] = [
    "Игр с плюсом по золоту", 
    "Игр с минусом по золоту", 
    "Побед с плюсом", 
    "Поражений с плюсом",
    "Винрейт с плюсом",
    "Побед с минусом",
    "Поражений с минусом",
    "Винрейт с минусом",
    "Максимальный плюс по золоту",
    "Максимальный минус по золоту",
    "Средний плюсовый торг",
    "Средний минусовый торг"
];

const BARGAINS_TOTAL_STATS_NAMES: [&'static str; 6] = [
    "Общий средний торг",
    "Суммарно игр с плюсовым торгом",
    "Суммарно игр с минусовым торгом",
    "Общий винрейт с плюсовым торгом",
    "Общий винрейт с минусовым торгом",
    "Общий винрейт фракции"
];

// const HEROES_CELLS_NAMES: [&'static str; 10] = [

// ]

#[derive(Debug, Default)]
pub struct RaceBargainsStats {
    pub average_bargains: Vec<f64>,
    pub total_plus_bargain_games: u32,
    pub total_minus_bargain_games: u32,
    pub total_plus_bargain_wins: u32,
    pub total_minus_bargain_wins: u32,
}

pub struct RaceStatsBuilder {
    pub bargains_data: HashMap<RaceType, RaceBargainsStats>
}

impl RaceStatsBuilder {
    pub fn new() -> Self {
        RaceStatsBuilder {
            bargains_data: HashMap::from_iter(RaceType::iter().filter(|r| *r != RaceType::NotDetected).map(|r| {
                (r, RaceBargainsStats::default())
            }))
        }
    }
}

impl StatsBuilder for RaceStatsBuilder {
    fn build(&mut self, data: &crate::utils::StatsGeneratorDataModel, workbook: &mut rust_xlsxwriter::Workbook) {
        for race in &data.races_data {
            match race.id {
                RaceType::NotDetected => {},
                _=> {
                    let worksheet = workbook.add_worksheet().set_name(&race.actual_name).unwrap();
                    build_bargains_stats(self, race, &data.races_data, &data.games_data, worksheet);
                    build_heroes_stats(race, &data.races_data, &data.heroes_data, &data.games_data, worksheet); 
                }
            }
        }
    }
}

// region: BARGAINS DATA

fn build_bargains_stats(builder: &mut RaceStatsBuilder, race: &Race, races_data: &Vec<Race>, games_data: &Vec<Game>, worksheet: &mut Worksheet) {
    let games_played_with_minus_gold = games_data.iter()
        .filter(|game| {
            (game.first_player_race == race.id && game.bargains_amount < 0) ||
            (game.second_player_race == race.id && game.bargains_amount > 0)  
        })
        //.unique_by(|game| game.id)
        .collect::<Vec<&Game>>();

    let games_played_with_plus_gold = games_data.iter()
        .filter(|game| {
            (game.first_player_race == race.id && game.bargains_amount >= 0) ||
            (game.second_player_race == race.id && game.bargains_amount <= 0) 
        })
        //.unique_by(|game| game.id) 
        .collect::<Vec<&Game>>();

    // setup table shape

    worksheet.merge_range(1, 3, 1, 8, "Данные о торгах за фракцию", &styles::TEXT_BOLD_CENTERED).unwrap();

    let mut data_column = 0;
    let mut data_row = 2;
    for cell_name in BARGAINS_CELLS_NAMES {
        data_column += 1;
        worksheet.set_column_width(data_column, 12).unwrap();
        worksheet.write_with_format(data_row, data_column as u16, cell_name, &styles::THIN_BORDER_TEXP_WRAP).unwrap();
    }

    // this one builds bargains info for all opponents races
    worksheet.set_column_width(0, 20).unwrap();
    for opp_race in races_data.iter().filter(|r| r.id != race.id && r.id != RaceType::NotDetected) { 
        data_row += 1;
        worksheet.write_with_format(data_row, 0, &opp_race.actual_name, &styles::THIN_BORDER_TEXP_WRAP).unwrap();
        build_race_bargains_stats(builder, race, opp_race, &games_played_with_plus_gold, &games_played_with_minus_gold, worksheet, data_row);
    }

    data_row += 2;
    data_column = 0;

    for cell_name in BARGAINS_TOTAL_STATS_NAMES {
        worksheet.write_with_format(data_row, data_column as u16, cell_name, &styles::THIN_BORDER_TEXP_WRAP).unwrap();
        data_column += 1;
    }

    data_row += 1;
    
    // this one shows complete race bargains data
    let race_bargains_total_data = builder.bargains_data.get(&race.id).unwrap();
    let total_average_bargain = race_bargains_total_data.average_bargains.iter().sum::<f64>() / 7.0;
    let total_plus_bargain_games = race_bargains_total_data.total_plus_bargain_games;
    let total_minus_bargain_games = race_bargains_total_data.total_minus_bargain_games;
    let total_plus_bargain_wins = race_bargains_total_data.total_plus_bargain_wins;
    let total_minus_bargain_wins = race_bargains_total_data.total_minus_bargain_wins;
    let total_plus_bargain_winrate = total_plus_bargain_wins as f64 / total_plus_bargain_games as f64 * 100.0;
    let total_minus_bargain_winrate = total_minus_bargain_wins as f64 / total_minus_bargain_games as f64 * 100.0;

    let total_winrate = (total_plus_bargain_wins + total_minus_bargain_wins) as f64 / (total_minus_bargain_games + total_plus_bargain_games) as f64 * 100.0;

    worksheet.write_with_format(data_row, 0, format!("{:.2}", total_average_bargain), &styles::THIN_BORDER_TEXP_WRAP).unwrap();
    worksheet.write_with_format(data_row, 1, total_plus_bargain_games, &styles::THIN_BORDER_TEXP_WRAP).unwrap();
    worksheet.write_with_format(data_row, 2, total_minus_bargain_games, &styles::THIN_BORDER_TEXP_WRAP).unwrap();
    worksheet.write_with_format(data_row, 3, format!("{:.3}%", total_plus_bargain_winrate), &styles::THIN_BORDER_TEXP_WRAP).unwrap();
    worksheet.write_with_format(data_row, 4, format!("{:.3}%", total_minus_bargain_winrate), &styles::THIN_BORDER_TEXP_WRAP).unwrap();
    worksheet.write_with_format(data_row, 5, format!("{:.3}%", total_winrate), &styles::THIN_BORDER_TEXP_WRAP).unwrap();

}

pub struct BargainsGameData {
    pub wins: u32,
    pub losses: u32,
    pub win_bargains: Vec<i16>,
    pub loss_bargains: Vec<i16>
}

// writes count / w-l / amount of bargains for concrete opponent race.
fn build_race_bargains_stats(builder: &mut RaceStatsBuilder, race: &Race, opp_race: &Race, games_with_plus: &Vec<&Game>, games_with_minus: &Vec<&Game>, worksheet: &mut Worksheet, data_row: u32) {
    let mut plus_bargains_data = BargainsGameData { wins: 0, losses: 0, win_bargains: vec![], loss_bargains: vec![] };
    games_with_plus.iter()
        .filter(|game| game.first_player_race == opp_race.id || game.second_player_race == opp_race.id)
        //.unique_by(|game| game.id)
        .for_each(|game| {
            if game.first_player_race == race.id && game.result == GameResult::FirstPlayerWon || game.second_player_race == race.id && game.result == GameResult::SecondPlayerWon {
                plus_bargains_data.wins += 1;
                plus_bargains_data.win_bargains.push(game.bargains_amount.abs());
            }
            else if game.first_player_race == race.id && game.result == GameResult::SecondPlayerWon || game.second_player_race == race.id && game.result == GameResult::FirstPlayerWon {
                plus_bargains_data.losses += 1;
                plus_bargains_data.loss_bargains.push(game.bargains_amount.abs());
            }
        });
    
    let plus_games_count = plus_bargains_data.wins + plus_bargains_data.losses;

    builder.bargains_data.get_mut(&race.id).unwrap().total_plus_bargain_games += plus_games_count;
    builder.bargains_data.get_mut(&race.id).unwrap().total_plus_bargain_wins += plus_bargains_data.wins;

    worksheet.write_with_format(data_row, 1, plus_games_count as u32, &styles::THIN_BORDER_TEXP_WRAP).unwrap();
    worksheet.write_with_format(data_row, 3, plus_bargains_data.wins, &styles::THIN_BORDER_TEXP_WRAP).unwrap();
    worksheet.write_with_format(data_row, 4, plus_bargains_data.losses, &styles::THIN_BORDER_TEXP_WRAP).unwrap();
    worksheet.write_with_format(data_row, 5, format!("{:.3}%", calc_winrate(plus_bargains_data.wins, plus_games_count)), &styles::THIN_BORDER_TEXP_WRAP).unwrap();
    worksheet.write_with_format(
        data_row, 
        9, 
        if let Some(max_plus) = get_max_bargain(&plus_bargains_data.win_bargains, &plus_bargains_data.loss_bargains) {format!("{}", &max_plus)} else {"Не игралось в плюс".to_string()}, 
        &styles::THIN_BORDER_TEXP_WRAP
    ).unwrap();


    let mut minus_bargains_data = BargainsGameData { wins: 0, losses: 0, win_bargains: vec![], loss_bargains: vec![] };
    games_with_minus.iter()
        .filter(|game| game.first_player_race == opp_race.id || game.second_player_race == opp_race.id)
        //.unique_by(|game| game.id)
        .for_each(|game| {
            if game.first_player_race == race.id && game.result == GameResult::FirstPlayerWon || game.second_player_race == race.id && game.result == GameResult::SecondPlayerWon {
                minus_bargains_data.wins += 1;
                minus_bargains_data.win_bargains.push(if game.bargains_amount >= 0 { game.bargains_amount * -1 } else { game.bargains_amount });
            }
            else if game.first_player_race == race.id && game.result == GameResult::SecondPlayerWon || game.second_player_race == race.id && game.result == GameResult::FirstPlayerWon {
                minus_bargains_data.losses += 1;
                minus_bargains_data.loss_bargains.push(if game.bargains_amount >= 0 { game.bargains_amount * -1 } else { game.bargains_amount });
            }
        });
    
    let minus_games_count = minus_bargains_data.wins + minus_bargains_data.losses;

    builder.bargains_data.get_mut(&race.id).unwrap().total_minus_bargain_games += minus_games_count;
    builder.bargains_data.get_mut(&race.id).unwrap().total_minus_bargain_wins += minus_bargains_data.wins;

    worksheet.write_with_format(data_row, 2, minus_games_count as u32, &styles::THIN_BORDER_TEXP_WRAP).unwrap();
    worksheet.write_with_format(data_row, 6, minus_bargains_data.wins, &styles::THIN_BORDER_TEXP_WRAP).unwrap();
    worksheet.write_with_format(data_row, 7, minus_bargains_data.losses, &styles::THIN_BORDER_TEXP_WRAP).unwrap();
    worksheet.write_with_format(data_row, 8, format!("{:.3}%", calc_winrate(minus_bargains_data.wins, minus_games_count)), &styles::THIN_BORDER_TEXP_WRAP).unwrap();
    worksheet.write_with_format(
        data_row, 
        10, 
        if let Some(max_minus) = get_min_bargain(&minus_bargains_data.win_bargains, &minus_bargains_data.loss_bargains) {format!("{}", &max_minus)} else {"Не игралось в минус".to_string()}, 
        &styles::THIN_BORDER_TEXP_WRAP
    ).unwrap();

    // average bargains 
    let plus_bargains_sum = plus_bargains_data.win_bargains.iter().map(|b| *b as i64).sum::<i64>() + plus_bargains_data.loss_bargains.iter().map(|b| *b as i64).sum::<i64>();
    let minus_bargains_sum = minus_bargains_data.win_bargains.iter().map(|b| *b as i64).sum::<i64>() + minus_bargains_data.loss_bargains.iter().map(|b| *b as i64).sum::<i64>();
    let bargains_sum = plus_bargains_sum + minus_bargains_sum;

    // average in this pair
    builder.bargains_data.get_mut(&race.id).unwrap().average_bargains.push(bargains_sum as f64 / (plus_games_count + minus_games_count) as f64);

    worksheet.write_with_format(
        data_row, 
        11, 
        if plus_games_count == 0 { "Не игралось в плюс".to_string() } else { format!("{:.3}", plus_bargains_sum as f64 / plus_games_count as f64)}, 
        &styles::THIN_BORDER_TEXP_WRAP).unwrap();
    worksheet.write_with_format(
        data_row, 
        12, 
        if minus_games_count == 0 { "Не игралось в минус".to_string() } else { format!("{:.3}", minus_bargains_sum as f64 / minus_games_count as f64)},
        &styles::THIN_BORDER_TEXP_WRAP).unwrap();
}

// self expl
fn get_max_bargain(win_bargains: &Vec<i16>, loss_bargains: &Vec<i16>) -> Option<i16> {
    let max_win_bargain = if let Some(max) = win_bargains.iter().max() { *max } else { i16::MIN };
    let max_loss_bargain = if let Some(max) = loss_bargains.iter().max() { *max } else { i16::MIN };

    if max_win_bargain == i16::MIN && max_loss_bargain == i16::MIN {
        return None;
    }    
    Some(max_win_bargain.max(max_loss_bargain))
}

// self expl
fn get_min_bargain(win_bargains: &Vec<i16>, loss_bargains: &Vec<i16>) -> Option<i16> {
    let min_win_bargain = if let Some(min) = win_bargains.iter().min() { *min } else { i16::MAX };
    let min_loss_bargain = if let Some(min) = loss_bargains.iter().min() { *min } else { i16::MAX };

    if min_win_bargain == i16::MAX && min_loss_bargain == i16::MAX {
        return None;
    }    
    Some(min_win_bargain.min(min_loss_bargain))
}

fn calc_winrate(wins: u32, total_games: u32) -> f32 {
    if total_games == 0 {
        return 0.0;
    }
    (wins as f32 / total_games as f32) * 100.0
}

// endregion


// region: HEROES DATA

fn build_heroes_stats(race: &Race, races_data: &Vec<Race>, heroes_data: &Vec<Hero>, games_data: &Vec<Game>, worksheet: &mut Worksheet) {
    worksheet.merge_range(14, 4, 14, 9, "Общая статистика использования героев", &styles::TEXT_BOLD_CENTERED).unwrap();
    let mut row_offset = 16;
    // first of all, collect all games for hero.
    let mut heroes_count = 0;

    let mut heroes_used_by_race  = vec![];

    let total_race_picks = games_data.iter()
        .filter(|game| {
            game.first_player_race == race.id || game.second_player_race == race.id 
        })
        .collect::<Vec<&Game>>();

    total_race_picks.iter()
        .for_each(|game| {
            if game.first_player_race == race.id {
                heroes_used_by_race.push(heroes_data.iter().find(|hero| hero.id == game.first_player_hero).unwrap());
            }
            else if game.second_player_race == race.id {
                heroes_used_by_race.push(heroes_data.iter().find(|hero| hero.id == game.second_player_hero).unwrap());
            }
        });
    
    let unique_picked_heroes = heroes_used_by_race.into_iter().unique_by(|hero| hero.id).collect::<Vec<&Hero>>();

    worksheet.write_with_format(row_offset - 1, 1, "Всего побед", &styles::THIN_BORDER_TEXP_WRAP).unwrap();
    worksheet.write_with_format(row_offset - 1, 2, "Всего поражений", &styles::THIN_BORDER_TEXP_WRAP).unwrap();
    worksheet.write_with_format(row_offset - 1, 3, "Всего игр", &styles::THIN_BORDER_TEXP_WRAP).unwrap();
    worksheet.write_with_format(row_offset - 1, 4, "Процент выбора", &styles::THIN_BORDER_TEXP_WRAP).unwrap();

    let mut col_offset = 5;
    for opp_race in races_data.iter().filter(|r| r.id != RaceType::NotDetected && r.id != race.id) {
        worksheet.write_with_format(row_offset - 1, col_offset, format!("Игр vs {}", &opp_race.actual_name), &styles::THIN_BORDER_TEXP_WRAP).unwrap();
        worksheet.write_with_format(row_offset - 1, col_offset + 1, format!("Винрейт vs {}", &opp_race.actual_name), &styles::THIN_BORDER_TEXP_WRAP).unwrap();
        col_offset += 2;
    }

    for hero in &unique_picked_heroes {
        let hero_wins = games_data.iter().filter(|game| {
            (game.first_player_hero == hero.id && game.first_player_race == race.id && game.result == GameResult::FirstPlayerWon) ||
            (game.second_player_hero == hero.id && game.second_player_race == race.id && game.result == GameResult::SecondPlayerWon)
        })
        .collect::<Vec<&Game>>();
        
        let hero_losses = games_data.iter().filter(|game| {
            (game.first_player_hero == hero.id && game.first_player_race == race.id && game.result == GameResult::SecondPlayerWon) ||
            (game.second_player_hero == hero.id && game.second_player_race == race.id && game.result == GameResult::FirstPlayerWon)
        })
        .collect::<Vec<&Game>>();

        let total_hero_games = hero_losses.len() + hero_wins.len();

        worksheet.write_with_format(row_offset + heroes_count, 0, &hero.actual_name, &styles::THIN_BORDER_TEXP_WRAP).unwrap();
        worksheet.write_with_format(row_offset + heroes_count, 1, hero_wins.len() as u32, &styles::THIN_BORDER_TEXP_WRAP).unwrap();
        worksheet.write_with_format(row_offset + heroes_count, 2, hero_losses.len() as u32, &styles::THIN_BORDER_TEXP_WRAP).unwrap();
        worksheet.write_with_format(row_offset + heroes_count, 3, total_hero_games as u32, &styles::THIN_BORDER_TEXP_WRAP).unwrap();
        // pickrate
        worksheet.write_with_format(row_offset + heroes_count, 4, format!("{:.3}%", total_hero_games as f64 / total_race_picks.len() as f64 * 100.0), &styles::THIN_BORDER_TEXP_WRAP).unwrap();

        let mut col_offset = 5;
        for opp_race in races_data.iter().filter(|r| r.id != RaceType::NotDetected && r.id != race.id) {
            let opp_race_wins = total_race_picks.iter()
                .filter(|game| {
                    (game.first_player_hero == hero.id && game.first_player_race == race.id && game.second_player_race == opp_race.id && game.result == GameResult::FirstPlayerWon) || 
                    (game.second_player_hero == hero.id && game.second_player_race == race.id && game.first_player_race == opp_race.id && game.result == GameResult::SecondPlayerWon) 
                })
                .map(|game| *game )
                .collect::<Vec<&Game>>()
                .len();

            let opp_race_losses = total_race_picks.iter()
                .filter(|game| {
                    (game.first_player_hero == hero.id && game.first_player_race == race.id && game.second_player_race == opp_race.id && game.result == GameResult::SecondPlayerWon) || 
                    (game.second_player_hero == hero.id && game.second_player_race == race.id && game.first_player_race == opp_race.id && game.result == GameResult::FirstPlayerWon) 
                })
                .map(|game| *game )
                .collect::<Vec<&Game>>()
                .len();
                
            let total_opp_race_games = opp_race_wins + opp_race_losses;
            worksheet.write_with_format(
                row_offset + heroes_count, 
                col_offset, 
                if total_opp_race_games == 0 { "Нет игр".to_string() } else { total_opp_race_games.to_string() }, 
                &styles::THIN_BORDER_TEXP_WRAP).unwrap();
            worksheet.write_with_format(
                row_offset + heroes_count, 
                col_offset + 1, 
                if total_opp_race_games == 0 { "Нет игр".to_string() } else { format!("{:.3}%", opp_race_wins as f64 / total_opp_race_games as f64 * 100.0) },
                &styles::THIN_BORDER_TEXP_WRAP).unwrap();
            col_offset += 2;
        }

        heroes_count += 1;
    }

    row_offset += heroes_count + 1;

    for opp_race in races_data.iter().filter(|r| r.id != RaceType::NotDetected && r.id != race.id) {
        build_hero_stats_vs_race(race, &unique_picked_heroes, heroes_data, opp_race, games_data, worksheet, row_offset);
        row_offset += heroes_count + 4;
    }
}

fn build_hero_stats_vs_race(race: &Race, race_heroes: &Vec<&Hero>, heroes_data: &Vec<Hero>, opp_race: &Race, games_data: &Vec<Game>, worksheet: &mut Worksheet, row_offset: u32) {
    worksheet.merge_range(row_offset, 4, row_offset, 9, &format!("{} vs {}", race.actual_name, opp_race.actual_name), &styles::TEXT_BOLD_CENTERED).unwrap();
    worksheet.merge_range(row_offset + 1, 0, row_offset + 2, 0, "VS", &styles::TEXT_CENTER_COLOR_RED).unwrap();
    
    let opp_race_heroes = heroes_data.iter().filter(|h| h.race == opp_race.id).collect::<Vec<&Hero>>();
    let mut col_offset = 1;

    let mut heroes_count = 0;
    for hero in race_heroes {
        worksheet.write_with_format(row_offset + 3 + heroes_count, 0, &hero.actual_name, &styles::THIN_BORDER).unwrap();
        heroes_count += 1;
    }

    for hero in &opp_race_heroes {
        worksheet.merge_range(row_offset + 1, col_offset, row_offset + 1, col_offset + 1, &hero.actual_name, &styles::THIN_BORDER_TEXT_CENTER).unwrap();
        worksheet.set_column_width(col_offset, 12).unwrap().set_column_width(col_offset + 1, 12).unwrap();
        worksheet.write_with_format(row_offset + 2, col_offset, "Побед", &styles::THIN_BORDER_TEXP_WRAP).unwrap();
        worksheet.write_with_format(row_offset + 2, col_offset + 1, "Поражений", &styles::THIN_BORDER_TEXP_WRAP).unwrap();
        col_offset += 2;
    }

    worksheet.set_column_width(col_offset + 1, 12).unwrap().set_column_width(col_offset + 2, 12).unwrap();
    worksheet.write_with_format(row_offset + 1, col_offset + 1, "Всего игр", &styles::THIN_BORDER_TEXP_WRAP).unwrap();
    worksheet.write_with_format(row_offset + 1, col_offset + 2, "Винрейт", &styles::THIN_BORDER_TEXP_WRAP).unwrap();
    worksheet.set_cell_format(row_offset + 2, col_offset + 1, &styles::BACKGROUND_SILVER).unwrap();
    worksheet.set_cell_format(row_offset + 2, col_offset + 2, &styles::BACKGROUND_SILVER).unwrap();

    heroes_count = 0;
    for hero in race_heroes {
        let mut opp_hero_count = 1;
        let mut total_games = 0;
        let mut total_wins = 0;
        for opp_hero in &opp_race_heroes {
            let (wins, losses) = get_heroes_pair_stats(hero, race, opp_hero, games_data);
            if wins == 0 {
                worksheet.write_with_format(row_offset + heroes_count + 3, opp_hero_count, wins, &styles::THIN_BORDER_TEXP_WRAP).unwrap();
            }
            else {
                worksheet.write_with_format(row_offset + heroes_count + 3, opp_hero_count, wins, &styles::BACKGROUND_GREEN).unwrap();
            }
            if losses == 0 {
                worksheet.write_with_format(row_offset + heroes_count + 3, opp_hero_count + 1, losses, &styles::THIN_BORDER_TEXP_WRAP).unwrap();
            }
            else {
                worksheet.write_with_format(row_offset + heroes_count + 3, opp_hero_count + 1, losses, &styles::BACKGROUND_RED).unwrap();
            }
            total_games += wins + losses;
            total_wins += wins;
            opp_hero_count += 2;
        }

        worksheet.write_with_format(
            row_offset + heroes_count + 3, 
            opp_hero_count + 1, 
            if total_games == 0 { "Нет игр".to_string() } else { total_games.to_string() }, &styles::THIN_BORDER_TEXP_WRAP).unwrap();
        worksheet.write_with_format(
            row_offset + heroes_count + 3, 
            opp_hero_count + 2,
            if total_games == 0 { "Нет игр".to_string() } else { format!("{:.3}%", total_wins as f64 / total_games as f64 * 100.0) },
            &styles::THIN_BORDER_TEXP_WRAP).unwrap();

        heroes_count += 1;
    }
}

fn get_heroes_pair_stats(hero: &Hero, race: &Race, opp_hero: &Hero, games_data: &Vec<Game>) -> (u32, u32) {
    let wins = games_data.iter().filter(|game| {
            (game.first_player_hero == hero.id && game.first_player_race == race.id && game.second_player_hero == opp_hero.id && game.result == GameResult::FirstPlayerWon) || 
            (game.second_player_hero == hero.id && game.second_player_race == race.id &&  game.first_player_hero == opp_hero.id && game.result == GameResult::SecondPlayerWon)
        })
        .collect::<Vec<&Game>>()
        .len();
    let losses = games_data.iter().filter(|game| {
            (game.first_player_hero == hero.id && game.first_player_race == race.id && game.second_player_hero == opp_hero.id && game.result == GameResult::SecondPlayerWon) || 
            (game.second_player_hero == hero.id && game.second_player_race == race.id && game.first_player_hero == opp_hero.id && game.result == GameResult::FirstPlayerWon)
        })
        .collect::<Vec<&Game>>()
        .len();
    (wins as u32, losses as u32)
}

// endregion