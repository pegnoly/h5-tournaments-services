use std::any;

use h5_tournaments_api::prelude::Game;

use super::{types::Parse, utils::ParsingDataModel};

pub struct ParserService;

#[derive(Debug, Default)]
pub struct ParsedData<'a> {
    pub first_player: &'a str,
    pub second_player: &'a str,
    pub games: Vec<Game>,
}

impl ParserService {
    pub fn parse_match_structure<'a, P>(
        &self,
        message: &'a String,
        parser: &'a P,
        data_model: &ParsingDataModel,
    ) -> ParsedData<'a>
    where
        P: Parse<'a>,
    {
        let message_parts = message
            .split("\n")
            .filter(|s| s.len() > 0)
            .collect::<Vec<&str>>();

        let mut opponents_found = false;
        let mut any_game_found = false;
        let mut parsed_data = ParsedData::default();

        for message_part in message_parts {
            if !opponents_found {
                if let Some(opponents) = parser.try_parse_opponents(message_part) {
                    tracing::info!("Got the match between {} and {}", opponents.0, opponents.1);
                    parsed_data.first_player = opponents.0;
                    parsed_data.second_player = opponents.1;
                    opponents_found = true;
                } else {
                    continue;
                }
            } else {
                if let Some(game) = parser.parse_game(message_part, data_model) {
                    if !any_game_found {
                        any_game_found = true;
                    }
                    tracing::info!("Got the game: {:?}", &game);
                    parsed_data.games.push(game);
                } else {
                    if any_game_found {
                        break;
                    }
                }
            }
        }

        parsed_data
    }
}
