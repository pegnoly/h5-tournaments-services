use h5_tournaments_api::prelude::Game;
use poise::serenity_prelude::Message;

use super::{types::{MatchStructure, Parse}, utils::ParsingDataModel};

pub struct ParserService {

}

impl ParserService {
    pub fn parse_match_structure<P>(&self, message: &String, parser: &P) -> impl MatchStructure 
        where P: Parse
    {
        let match_data = parser.parse_match(message);
        match_data
    }

    pub fn parse_opponents<P, M>(&self, parser: &P, match_data: &M, data: &ParsingDataModel) -> Vec<String>
        where 
            P: Parse,
            M: MatchStructure
    {
        match_data.get_opponents_data();
        vec![]
    }

    pub fn parse_games<P, M>(&self, parser: &P, match_data: &M, data: &ParsingDataModel) -> Vec<Game> 
        where 
            P: Parse,
            M: MatchStructure
    {
        match_data.get_games_data();
        vec![]
    }
}