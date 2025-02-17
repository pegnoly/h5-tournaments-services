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