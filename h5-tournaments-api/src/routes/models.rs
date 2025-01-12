use serde::{Deserialize, Serialize};
use uuid::Uuid;

#[derive(Debug, Serialize, Deserialize, Default)]
pub struct TournamentCreationModel {
    pub mod_type: i32,
    pub name: String,
    pub server_id: i64,
    pub channel_id: i64,
    pub first_message_id: i64,
    pub last_message_id: i64
}

#[derive(Debug, Serialize, Deserialize, Default)]
pub struct MatchRegistrationForm {
    pub tournament_id: Uuid,
    pub first_player: String,
    pub second_player: String,
    pub message_id: i64
}