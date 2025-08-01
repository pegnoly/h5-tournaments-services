use serde::{Deserialize, Serialize};
use strum::{Display, EnumString};

#[derive(Debug, Serialize, Deserialize, EnumString, Display, PartialEq, Eq, Clone)]
pub enum ChallongeTournamentState {
    #[strum(serialize = "pending")]
    Pending,
    #[strum(serialize = "group_stages_underway")]
    GroupStagesUnderway,
    #[strum(serialize = "group_stages_finalized")]
    GroupStagesFinalized,
    #[strum(serialize = "underway")]
    Underway,
    #[strum(serialize = "awaiting_review")]
    AwaitingReview,
    #[strum(serialize = "complete")]
    Complete
}

#[derive(Debug, Serialize, Deserialize)]
pub struct ChallongeTournamentsSimple {
    pub data: Vec<ChallongeTournamentSimpleData>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct ChallongeTournamentSimple {
    pub data: ChallongeTournamentSimpleData
}

#[derive(Debug, Serialize, Deserialize)]
pub struct ChallongeTournamentSimpleData {
    pub id: String,
    pub attributes: ChallongeTournamentSimpleAttributes,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct ChallongeTournamentSimpleAttributes {
    pub name: String,
    pub state: String,
    pub starts_at: Option<String>
}

#[derive(Debug, Serialize, Deserialize)]
pub struct ChallongeParticipantsSimple {
    pub data: Vec<ChallongeParticipantSimpleData>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct ChallongeParticipantSimple {
    pub data: ChallongeParticipantSimpleData,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct ChallongeParticipantSimpleData {
    pub id: String,
    pub attributes: ChallongeParticipantSimpleAttributes,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct ChallongeParticipantSimpleAttributes {
    pub name: String,
    pub misc: Option<String>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct ChallongeMatches {
    pub data: Vec<ChallongeMatchData>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct ChallongeSingleMatch {
    pub data: ChallongeMatchData,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct ChallongeMatchData {
    pub id: String,
    pub attributes: ChallongeMatchAttributes,
    //pub relationships: ChallongeMatchRelationships,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct ChallongeMatchAttributes {
    pub state: String,
    pub round: i32,
    pub identifier: String,
    pub points_by_participant: Vec<ChallongeMatchParticipantInfo>
}

#[derive(Debug, Serialize, Deserialize)]
pub struct ChallongeMatchParticipantInfo {
    pub participant_id: i32, // so cool to store ids in different data types in different models :)
    pub scores: Vec<i32>
}

#[derive(Debug, Serialize, Deserialize)]
pub struct ChallongeMatchRelationships {
    pub player1: ChallongeMatchRelationshipsPlayer,
    pub player2: ChallongeMatchRelationshipsPlayer,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct ChallongeMatchRelationshipsPlayer {
    pub data: ChallongeMatchRelationshipsPlayerData,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct ChallongeMatchRelationshipsPlayerData {
    pub id: String,
}
