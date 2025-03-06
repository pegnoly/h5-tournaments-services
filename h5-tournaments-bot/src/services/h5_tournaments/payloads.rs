use h5_tournaments_api::prelude::ModType;
use uuid::Uuid;

use crate::{builders::{self, types::GameType}, graphql::queries::{
    create_organizer, create_participant, create_tournament_mutation, create_user_mutation, delete_participant, get_operator_data_query, get_organizer, get_participant, get_tournament_builder, update_tournament, update_tournament_builder, CreateOrganizer
}};

#[derive(Debug)]
pub struct CreateOrganizerPayload {
    pub discord_id: u64,
    pub challonge_key: String,
}

impl CreateOrganizerPayload {
    pub fn new(discord: u64, key: String) -> Self {
        CreateOrganizerPayload {
            discord_id: discord,
            challonge_key: key,
        }
    }
}

impl From<CreateOrganizerPayload> for create_organizer::Variables {
    fn from(value: CreateOrganizerPayload) -> Self {
        create_organizer::Variables {
            discord_id: value.discord_id.to_string(),
            challonge_key: value.challonge_key,
        }
    }
}

#[derive(Debug, Default)]
pub struct GetOrganizerPayload {
    pub id: Option<Uuid>,
    pub discord_id: Option<i64>,
    pub challonge_key: Option<String>,
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
            challonge_key: value.challonge_key,
        }
    }
}

#[derive(Debug, Default)]
pub struct GetTournamentBuilderPayload {
    pub id: Option<Uuid>,
    pub message: Option<i64>,
}

impl GetTournamentBuilderPayload {
    pub fn with_id(mut self, id: Uuid) -> Self {
        self.id = Some(id);
        self
    }

    pub fn with_message(mut self, message: u64) -> Self {
        self.message = Some(message as i64);
        self
    }
}

impl From<GetTournamentBuilderPayload> for get_tournament_builder::Variables {
    fn from(value: GetTournamentBuilderPayload) -> Self {
        get_tournament_builder::Variables {
            id: value.id,
            message: value.message,
        }
    }
}

#[derive(Debug)]
pub struct UpdateTournamentBuilderPayload {
    pub id: Uuid,
    pub name: Option<String>,
    pub edit_state: Option<update_tournament_builder::TournamentEditState>,
    pub register_channel: Option<String>,
    pub reports_channel: Option<String>,
    pub role: Option<String>,
    pub use_bargains: Option<bool>,
    pub use_bargains_color: Option<bool>,
    pub use_foreign_heroes: Option<bool>,
}

impl UpdateTournamentBuilderPayload {
    pub fn new(id: Uuid) -> Self {
        UpdateTournamentBuilderPayload {
            id: id,
            name: None,
            edit_state: Some(update_tournament_builder::TournamentEditState::NOT_SELECTED),
            register_channel: None,
            reports_channel: None,
            role: None,
            use_bargains: None,
            use_bargains_color: None,
            use_foreign_heroes: None,
        }
    }

    pub fn with_name(mut self, name: String) -> Self {
        self.name = Some(name);
        self
    }

    pub fn with_edit_state(
        mut self,
        state: update_tournament_builder::TournamentEditState,
    ) -> Self {
        self.edit_state = Some(state);
        self
    }

    pub fn with_register_channel(mut self, channel: u64) -> Self {
        self.register_channel = Some(channel.to_string());
        self
    }

    pub fn with_reports_channel(mut self, channel: u64) -> Self {
        self.reports_channel = Some(channel.to_string());
        self
    }

    pub fn with_role(mut self, role: u64) -> Self {
        self.role = Some(role.to_string());
        self
    }

    pub fn with_bargains(mut self, bargains: bool) -> Self {
        self.use_bargains = Some(bargains);
        self
    }

    pub fn with_bargains_color(mut self, color: bool) -> Self {
        self.use_bargains_color = Some(color);
        self
    }

    pub fn with_foreign_heroes(mut self, heroes: bool) -> Self {
        self.use_foreign_heroes = Some(heroes);
        self
    }
}

impl From<UpdateTournamentBuilderPayload> for update_tournament_builder::Variables {
    fn from(value: UpdateTournamentBuilderPayload) -> Self {
        update_tournament_builder::Variables {
            id: value.id,
            name: value.name,
            state: value.edit_state,
            register_channel: value.register_channel,
            reports_channel: value.reports_channel,
            role: value.role,
            use_bargains: value.use_bargains,
            use_bargains_color: value.use_bargains_color,
            use_foreign_heroes: value.use_foreign_heroes,
        }
    }
}

#[derive(Debug)]
pub struct CreateTournamentPayload {
    pub name: String,
    pub operator_id: Uuid,
    pub channel_id: String,
    pub register_channel: String,
    pub use_bargains: bool,
    pub use_bargains_color: bool,
    pub use_foreign_heroes: bool,
    pub role: String,
    pub organizer: Uuid,
    pub game_type: GameType,
    pub mod_type: ModType
}

impl Into<create_tournament_mutation::GameType> for builders::types::GameType {
    fn into(self) -> create_tournament_mutation::GameType {
        match self {
            GameType::Rmg => create_tournament_mutation::GameType::RMG,
            GameType::Arena => create_tournament_mutation::GameType::ARENA
        }
    }
}

impl Into<create_tournament_mutation::ModType> for ModType {
    fn into(self) -> create_tournament_mutation::ModType {
        match self {
            ModType::Universe => create_tournament_mutation::ModType::UNIVERSE,
            ModType::Hrta => create_tournament_mutation::ModType::HRTA
        }
    }
}

impl From<CreateTournamentPayload> for create_tournament_mutation::Variables {
    fn from(value: CreateTournamentPayload) -> Self {
        create_tournament_mutation::Variables {
            name: value.name,
            operator_id: value.operator_id,
            channel_id: value.channel_id,
            register_channel: value.register_channel,
            use_bargains: value.use_bargains,
            use_bargains_color: value.use_bargains_color,
            use_foreign_heroes: value.use_foreign_heroes,
            role: value.role,
            organizer: value.organizer,
            game_type: value.game_type.into(),
            mod_type: value.mod_type.into()
        }
    }
}

#[derive(Debug)]
pub struct UpdateTournamentPayload {
    pub id: Uuid,
    pub stage: Option<update_tournament::TournamentStage>,
    pub challonge_id: Option<String>,
}

impl UpdateTournamentPayload {
    pub fn new(id: Uuid) -> Self {
        UpdateTournamentPayload {
            id: id,
            stage: None,
            challonge_id: None,
        }
    }

    pub fn with_stage(mut self, stage: update_tournament::TournamentStage) -> Self {
        self.stage = Some(stage);
        self
    }

    pub fn with_challonge_id(mut self, challonge: String) -> Self {
        self.challonge_id = Some(challonge);
        self
    }
}

impl From<UpdateTournamentPayload> for update_tournament::Variables {
    fn from(value: UpdateTournamentPayload) -> Self {
        update_tournament::Variables {
            id: value.id,
            stage: value.stage,
            challonge_id: value.challonge_id,
        }
    }
}

#[derive(Debug)]
pub struct CreateParticipantPayload {
    pub tournament_id: Uuid,
    pub user_id: Uuid,
    pub challonge_id: String,
}

impl CreateParticipantPayload {
    pub fn new(tournament: Uuid, user: Uuid, challonge: String) -> Self {
        CreateParticipantPayload {
            tournament_id: tournament,
            user_id: user,
            challonge_id: challonge,
        }
    }
}

impl From<CreateParticipantPayload> for create_participant::Variables {
    fn from(value: CreateParticipantPayload) -> Self {
        create_participant::Variables {
            tournament_id: value.tournament_id,
            user_id: value.user_id,
            challonge_id: value.challonge_id,
        }
    }
}

#[derive(Debug)]
pub struct DeleteParticipantPayload {
    pub tournament_id: Uuid,
    pub id: Option<Uuid>,
    pub user_id: Option<Uuid>,
    pub challonge_id: Option<String>,
}

impl DeleteParticipantPayload {
    pub fn new(tournament: Uuid) -> Self {
        DeleteParticipantPayload {
            tournament_id: tournament,
            id: None,
            user_id: None,
            challonge_id: None,
        }
    }

    pub fn with_id(mut self, id: Uuid) -> Self {
        self.id = Some(id);
        self
    }

    pub fn with_user_id(mut self, user: Uuid) -> Self {
        self.user_id = Some(user);
        self
    }

    pub fn with_challonge_id(mut self, challonge: String) -> Self {
        self.challonge_id = Some(challonge);
        self
    }
}

impl From<DeleteParticipantPayload> for delete_participant::Variables {
    fn from(value: DeleteParticipantPayload) -> Self {
        delete_participant::Variables {
            tournament_id: value.tournament_id,
            id: value.id,
            user_id: value.user_id,
            challonge_id: value.challonge_id,
        }
    }
}

#[derive(Debug)]
pub struct CreateUserPayload {
    pub lobby_nick: String,
    pub discord_id: u64,
    pub discord_nick: String,
}

impl CreateUserPayload {
    pub fn new(lobby: String, id: u64, nick: String) -> Self {
        CreateUserPayload {
            lobby_nick: lobby,
            discord_id: id,
            discord_nick: nick,
        }
    }
}

impl From<CreateUserPayload> for create_user_mutation::Variables {
    fn from(value: CreateUserPayload) -> Self {
        create_user_mutation::Variables {
            name: value.lobby_nick,
            discord_id: value.discord_id as i64,
            discord_nick: value.discord_nick,
        }
    }
}

#[derive(Debug, Default)]
pub struct GetParticipantPayload {
    pub tournament_id: Option<Uuid>,
    pub user_id: Option<Uuid>,
    pub challonge_id: Option<String>,
}

impl GetParticipantPayload {
    pub fn with_tournament(mut self, tournament: Uuid) -> Self {
        self.tournament_id = Some(tournament);
        self
    }

    pub fn with_user(mut self, user: Uuid) -> Self {
        self.user_id = Some(user);
        self
    }

    pub fn with_challonge(mut self, challonge: String) -> Self {
        self.challonge_id = Some(challonge);
        self
    }
}

impl From<GetParticipantPayload> for get_participant::Variables {
    fn from(value: GetParticipantPayload) -> Self {
        get_participant::Variables {
            tournament_id: value.tournament_id,
            user_id: value.user_id,
            challonge: value.challonge_id,
        }
    }
}

#[derive(Debug, Default)]
pub struct GetOperatorPayload {
    pub id: Option<Uuid>,
    pub server_id: Option<i64>
}

impl GetOperatorPayload {
    pub fn with_id(mut self, id: Uuid) -> Self {
        self.id = Some(id);
        self
    }

    pub fn with_server_id(mut self, server: u64) -> Self {
        self.server_id = Some(server as i64);
        self
    }
}

impl From<GetOperatorPayload> for get_operator_data_query::Variables {
    fn from(value: GetOperatorPayload) -> Self {
        get_operator_data_query::Variables { 
            id: value.id, 
            server_id: value.server_id 
        }
    }
}