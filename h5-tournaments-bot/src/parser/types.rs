use super::utils::ParsingDataModel;
use h5_tournaments_api::prelude::{Game, GameResult};
use human_regex::{any, digit, exactly, one_or_more, or, whitespace, zero_or_more};

pub trait MatchStructure {
    fn get_opponents_data(&self);
    fn get_games_data(&self);
}

pub struct UniverseMatchStructure;

impl MatchStructure for UniverseMatchStructure {
    fn get_games_data(&self) {
        todo!()
    }

    fn get_opponents_data(&self) {
        todo!()
    }
}

pub struct HrtaMatchStructure;

impl MatchStructure for HrtaMatchStructure {
    fn get_games_data(&self) {
        todo!()
    }

    fn get_opponents_data(&self) {
        todo!()
    }
}

pub trait Parse<'a> {
    fn try_parse_opponents(&self, opponents_string: &'a str) -> Option<(&'a str, &'a str)> {
        let opponents = opponents_string
            .split("vs")
            .map(|s| s.trim())
            .collect::<Vec<&str>>();

        if opponents.len() == 1 {
            None
        } else {
            Some((opponents[0], opponents[1]))
        }
    }

    fn parse_game(&self, game_string: &str, model: &ParsingDataModel) -> Option<Game>;
}

pub struct UniverseParser;

impl<'a> Parse<'a> for UniverseParser {
    fn parse_game(&self, game_string: &str, model: &ParsingDataModel) -> Option<Game> {
        todo!()
    }
}

pub struct HrtaParser;

impl<'a> Parse<'a> for HrtaParser {
    fn parse_game(&self, game_string: &str, model: &ParsingDataModel) -> Option<Game> {
        let sides_data: Vec<&str> = game_string
            .split_inclusive(|c| c == '>' || c == '<')
            .map(|s| s.trim())
            .collect();

        if sides_data.len() != 2 {
            None
        } else {
            let mut result = GameResult::NotDetected;
            if let Some(_) = sides_data.iter().find(|d| d.contains(">")) {
                result = GameResult::FirstPlayerWon;
            } else {
                result = GameResult::SecondPlayerWon;
            }

            let mut game = Game::default();
            let first_player_data = process_side_data(sides_data[0], model);
            let second_player_data = process_side_data(sides_data[1], model);
            game.first_player_hero = first_player_data.hero_id;
            game.first_player_race = first_player_data.race_id;
            game.second_player_hero = second_player_data.hero_id;
            game.second_player_race = second_player_data.race_id;
            game.bargains_amount = first_player_data.bargains_amount as i16;
            game.result = result;
            Some(game)
        }
    }
}

pub(self) struct SideData {
    pub race_id: i32,
    pub hero_id: i32,
    pub bargains_amount: i32,
}

fn process_side_data(side_string: &str, data_model: &ParsingDataModel) -> SideData {
    let side_string = side_string.to_lowercase();

    let mut side_data = SideData {
        race_id: 0,
        hero_id: 0,
        bargains_amount: 0,
    };

    if let Some(race) = data_model.races.iter().find(|r| {
        r.name_variants
            .variants
            .iter()
            .any(|v| side_string.contains(v))
    }) {
        side_data.race_id = race.id as i32;
    }

    if let Some(hero) = data_model.heroes.iter().find(|h| {
        h.name_variants
            .variants
            .iter()
            .any(|v| side_string.contains(v))
    }) {
        side_data.hero_id = hero.id;
    }

    let bargains_parts = side_string.split("(").collect::<Vec<&str>>();
    if bargains_parts.len() != 2 {
        return side_data;
    }

    let readable_regex =
        exactly(1, or(&["+", "-"])) + zero_or_more(whitespace()) + one_or_more(digit());
    let amount_regex = readable_regex.to_regex();
    if let Some(capture) = amount_regex.find(&bargains_parts[1]) {
        if let Ok(amount) = capture.as_str().replace(" ", "").parse::<i32>() {
            side_data.bargains_amount = amount;
        }
    }

    side_data
}
