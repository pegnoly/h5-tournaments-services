use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize, Default)]
pub struct TournamentCreationModel {
    pub mod_type: i32,
    pub name: String,
    pub server_id: i64,
    pub channel_id: i64,
    pub first_message_id: i64,
    pub last_message_id: i64
}