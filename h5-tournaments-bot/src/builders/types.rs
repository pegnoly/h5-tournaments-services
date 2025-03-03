use std::default;

use h5_tournaments_api::prelude::{Hero, ModType};
use serde::{Deserialize, Serialize};
use strum::{Display, EnumString, FromRepr};
use uuid::Uuid;

#[derive(Serialize, Deserialize)]
pub struct OpponentDataPayload {
    pub nickname: String,
    pub opponent_id: String,
    pub match_id: String,
}

#[derive(Debug)]
pub struct OpponentsData {
    pub nickname: String,
    pub challonge_data: String,
}

#[derive(Debug)]
pub struct MatchBuilder {
    pub opponents: Vec<OpponentsData>,
    pub selected_opponent: Option<String>,
    pub player: String,
    pub games_count: Option<i32>,
    pub user_nickname: String,
    pub tournament_name: String,
    pub tournament_id: Uuid,
}

#[derive(Debug, Default, PartialEq, Eq)]
pub enum GameBuilderState {
    NotSelected,
    #[default]
    PlayerData,
    OpponentData,
    ResultData,
    BargainsData,
}

#[derive(Debug, EnumString, Display, Default, PartialEq, Eq, FromRepr, Clone)]
#[repr(i32)]
pub enum GameResult {
    #[default]
    NotSelected = 0,
    FirstPlayerWon = 1,
    SecondPlayerWon = 2,
}

#[derive(Debug, Default)]
pub struct GameBuilder {
    pub number: i32,
    pub state: GameBuilderState,
    pub first_player_race: Option<i64>,
    pub first_player_hero: Option<i64>,
    pub second_player_race: Option<i64>,
    pub second_player_hero: Option<i64>,
    pub bargains_amount: i64,
    pub result: GameResult,
}

#[derive(Debug)]
pub struct GameBuilderContainer {
    pub tournament_id: Uuid,
    pub match_id: Uuid,
    pub heroes: Vec<Hero>,
    pub current_number: i32,
    pub player_nickname: String,
    pub opponent_nickname: String,
    pub builders: Vec<GameBuilder>,
}

#[derive(Debug, PartialEq, Eq, EnumString, Display, Clone, Copy)]
pub enum GameType {
    Rmg,
    Arena,
}

#[derive(Debug, Default, PartialEq, Eq)]
pub enum TournamentBuildState {
    #[default]
    BaseData,
    ChannelsData,
    ReportsData
}

#[derive(Debug, Default)]
pub struct TournamentBuilder {
    pub name: Option<String>,
    pub organizer: Option<Uuid>,
    pub edit_state: TournamentBuildState, 
    pub game_type: Option<GameType>,
    pub mod_type: Option<ModType>,
    pub register_channel: Option<u64>,
    pub reports_channel: Option<u64>,
    pub role: Option<u64>,
    pub use_bargains: Option<bool>,
    pub use_bargains_color: Option<bool>,
    pub use_foreign_heroes: Option<bool>,
}
