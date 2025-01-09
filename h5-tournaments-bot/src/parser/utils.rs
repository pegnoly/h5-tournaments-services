use serde::{Deserialize, Serialize};
use strum::{EnumIter, FromRepr};
use uuid::Uuid;

use h5_tournaments_api::prelude::*;

pub struct ParsingDataModel {
    pub races: Vec<Race>,
    pub heroes: Vec<Hero>
}


/// Possible game outcomes
#[derive(Debug, Serialize, Deserialize)]
#[repr(i16)]
pub enum GameResult {
    NotDetected = 0,
    FirstPlayerWon = 1,
    SecondPlayerWon = 2
}

/// Result of parsing single player's side info in a game
pub struct GameSideData {
    pub race: i32,
    pub hero: i32
}

/// Possible colors used in bargains
#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
#[repr(i16)]
pub enum BargainsColor {
    NotDetected,
    ColorRed,
    ColorBlue
}

/// Predefined type for bargains color detection
pub struct BargainsType {
    pub color: BargainsColor,
    pub actual_name: String,
    pub variants: Vec<String>
}

/// Result of parsing single game bargains info
pub struct BargainsData {
    pub color: BargainsColor,
    pub amount: i32
}