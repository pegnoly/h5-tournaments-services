use h5_tournaments_api::prelude::Game;

use super::utils::ParsingDataModel;

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

pub trait Parse {
    type Output: MatchStructure;

    fn parse_match(&self, message_text: &String) -> Self::Output;
    fn parse_opponents(&self, match_data: &Self::Output) -> Vec<String>;
    fn parse_games(&self, match_data: &Self::Output) -> Vec<Game>;
}

pub struct UniverseParser;

impl Parse for UniverseParser {
    type Output = UniverseMatchStructure;

    fn parse_match(&self, message_text: &String) -> UniverseMatchStructure {
        todo!()
    }

    fn parse_opponents(&self, match_data: &UniverseMatchStructure) -> Vec<String> {
        todo!()
    }

    fn parse_games(&self, match_data: &UniverseMatchStructure) -> Vec<Game> {
        todo!()
    }
}

pub struct HrtaParser;

impl Parse for HrtaParser {
    type Output = HrtaMatchStructure;

    fn parse_match(&self, message_text: &String) -> Self::Output {
        todo!()
    }

    fn parse_opponents(&self, match_data: &Self::Output) -> Vec<String> {
        todo!()
    }

    fn parse_games(&self, match_data: &Self::Output) -> Vec<Game> {
        todo!()
    }
}