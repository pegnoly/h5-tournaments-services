use serde::{Serialize, Deserialize};
use strum::{Display, EnumString};

#[derive(Debug, EnumString, Display, Serialize, Deserialize)]
pub enum ChallongePayloadType {
    Participants
}

#[derive(Debug, Serialize, Deserialize)]
pub struct ChallongeData<T>
    where T: Serialize
{
    pub data: T
}

#[derive(Debug, Serialize, Deserialize)]
pub struct ChallongeParticipantPayload {
    #[serde(rename = "type")]
    pub _type: ChallongePayloadType,
    pub attributes: Option<ChallongeParticipantAttributes>
}

#[derive(Debug, Serialize, Deserialize)]
pub struct ChallongeParticipantAttributes {
    pub name: String,
    pub seed: Option<i32>,
    pub misc: Option<String>,
    pub email: Option<String>,
    pub username: Option<String>
}

#[derive(Debug, Serialize, Deserialize)]
pub struct ChallongeParticipantsBulkAttributes {
    pub participants: Vec<ChallongeParticipantAttributes>
}

#[derive(Debug, Serialize, Deserialize)]
pub struct ChallongeParticipantsBulkAddPayload {
    #[serde(rename = "type")]
    pub _type: ChallongePayloadType,
    pub attributes: Option<ChallongeParticipantsBulkAttributes>
}