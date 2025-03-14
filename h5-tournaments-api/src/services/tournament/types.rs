use sea_orm::DeriveActiveEnum;
use serde::{Deserialize, Serialize};
use sqlx::types::Json;
use strum::{Display, EnumIter, EnumString, FromRepr};
use uuid::Uuid;

#[derive(Debug, Serialize, Deserialize, EnumIter, FromRepr, EnumString, Display, Clone, Copy, PartialEq, Eq, Hash, Default, sqlx::Type, async_graphql::Enum, DeriveActiveEnum)]
#[sea_orm(rs_type = "i32", db_type = "Integer")]
#[repr(i16)]
pub enum ModType {
    #[default]
    Universe = 0,
    Hrta = 1
}

#[derive(Debug, Serialize, Deserialize)]
pub struct NameVariants {
    pub variants: Vec<String>
}


#[derive(Debug, Serialize, Deserialize, sqlx::FromRow)]
pub struct Race {
    pub id: i32,
    pub actual_name: String,
    pub name_variants: Json<NameVariants>
}


#[derive(Debug, Serialize, Deserialize, sqlx::FromRow)]
pub struct Hero {
    pub id: i32,
    pub race: i32,
    pub actual_name: String,
    pub name_variants: Json<NameVariants>,
    pub mod_type: i16
}

#[derive(Debug, Serialize, Deserialize, sqlx::FromRow)]
pub struct TournamentProvider {
    pub id: Uuid,
}


#[derive(Debug, Serialize, Deserialize, sqlx::FromRow)]
pub struct Tournament {
    pub id: Uuid,
    pub mod_type: i16,
    pub server_id: i64,
    pub channel_id: i64,
    pub first_message_id: i64,
    pub last_message_id: i64,
    pub name: String
}


/// A match between two players in a concrete tournament. Contains Games.
#[derive(Debug, Serialize, Deserialize, Default, Clone, sqlx::FromRow)]
pub struct Match {
    pub id: i32,
    pub tournament_id: Uuid,
    pub first_player: String,
    pub second_player: String,
    pub message_id: i64
}

/// Possible game outcomes
#[derive(Debug, Serialize, Deserialize, FromRepr, Clone, PartialEq, Eq, sqlx::Type)]
#[repr(i16)]
pub enum GameResult {
    NotDetected = 0,
    FirstPlayerWon = 1,
    SecondPlayerWon = 2
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct GameResultModel {
    pub id: GameResult,
    pub name: String
}

/// Possible colors used in bargains
#[derive(Debug, Clone, Copy, Serialize, Deserialize, FromRepr, PartialEq, Eq, sqlx::Type, EnumString, Display)]
#[repr(i16)]
pub enum BargainsColor {
    NotDetected,
    ColorRed,
    ColorBlue
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct BargainsColorModel {
    pub id: BargainsColor,
    pub name: String
}

/// A single game between two players.
#[derive(Debug, Serialize, Deserialize, sqlx::FromRow)]
pub struct Game {
    pub id: i32,
    pub match_id: i32,
    pub first_player_race: i64,
    pub first_player_hero: i64,
    pub second_player_race: i64,
    pub second_player_hero: i64,
    pub bargains_amount: i32,
    pub result: GameResult
}

impl Default for Game {
    fn default() -> Self {
        Game {
            first_player_race: 0,
            first_player_hero: 0,
            second_player_race: 0,
            second_player_hero: 0,
            result: GameResult::NotDetected,
            id: 0,
            match_id: 0,
            bargains_amount: 0
        }
    }
}

#[derive(Debug, Serialize, Deserialize, sqlx::FromRow)]
pub struct TempMessageModel {
    pub message_id: i64,
    pub message_text: String,
    pub tournament_id: Uuid
}