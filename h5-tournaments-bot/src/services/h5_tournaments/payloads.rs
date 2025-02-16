use uuid::Uuid;

use crate::graphql::queries::{create_organizer, get_organizer, CreateOrganizer};

#[derive(Debug)]
pub struct CreateOrganizerPayload {
    pub discord_id: u64,
    pub challonge_key: String
}

impl CreateOrganizerPayload {
    pub fn new(discord: u64, key: String) -> Self {
        CreateOrganizerPayload {
            discord_id: discord,
            challonge_key: key
        }
    }
}

impl From<CreateOrganizerPayload> for create_organizer::Variables {
    fn from(value: CreateOrganizerPayload) -> Self {
        create_organizer::Variables { 
            discord_id: value.discord_id.to_string(), 
            challonge_key: value.challonge_key 
        }
    }
}

#[derive(Debug, Default)]
pub struct GetOrganizerPayload {
    pub id: Option<Uuid>,
    pub discord_id: Option<i64>,
    pub challonge_key: Option<String>
}

impl GetOrganizerPayload {
    pub fn with_id(mut self, id: Uuid) -> Self {
        self.id = Some(id);
        self
    }

    pub fn with_discord_id(mut self, discord: i64) -> Self {
        self.discord_id = Some(discord);
        self
    }

    pub fn with_key(mut self, key: String) -> Self {
        self.challonge_key = Some(key);
        self
    }
}

impl From<GetOrganizerPayload> for get_organizer::Variables {
    fn from(value: GetOrganizerPayload) -> Self {
        get_organizer::Variables { 
            id: value.id, 
            discord_id: value.discord_id, 
            challonge_key: value.challonge_key 
        }
    }
}