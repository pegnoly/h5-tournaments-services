use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize)]
pub struct ChallongeTournamentsSimple {
    pub data: Vec<ChallongeTournamentSimpleData>
}

#[derive(Debug, Serialize, Deserialize)]
pub struct ChallongeTournamentSimpleData {
    pub id: String,
    pub attributes: ChallongeTournamentSimpleAttributes
}

#[derive(Debug, Serialize, Deserialize)]
pub struct ChallongeTournamentSimpleAttributes {
    pub name: String
}

#[derive(Debug, Serialize, Deserialize)]
pub struct ChallongeParticipantsSimple {
    pub data: Vec<ChallongeParticipantSimpleData>
}

#[derive(Debug, Serialize, Deserialize)]
pub struct ChallongeParticipantSimpleData {
    pub id: String,
    pub attributes: ChallongeParticipantSimpleAttributes
}

#[derive(Debug, Serialize, Deserialize)]
pub struct ChallongeParticipantSimpleAttributes {
    pub name: String,
    pub misc: Option<String>
}