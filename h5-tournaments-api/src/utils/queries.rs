use serde::{Deserialize, Serialize};
use uuid::Uuid;

use crate::tournament::utils::RaceType;

#[derive(Debug, Serialize, Deserialize)]
pub struct TournamentQueryModel {
    pub id: Option<Uuid>,
    pub channel_id: Option<i64>
}

#[derive(Debug, Serialize, Deserialize)]
pub struct MatchQueryModel {
    pub message: u64,
    pub tournament: Uuid,
    pub first_player: String,
    pub second_player: String
}

#[derive(Debug, Serialize, Deserialize)]
pub struct RacesPairQueryModel {
    pub race_one: RaceType,
    pub race_two: RaceType
}