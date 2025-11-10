use create_user_mutation::CreateUserMutationCreateUser;
use get_tournament_query::GetTournamentQueryTournament;
use get_user_query::GetUserQueryUser;
use get_users_query::GetUsersQueryUsers;
use graphql_client::GraphQLQuery;
use h5_tournaments_api::prelude::ModType;

use crate::builders::{self, types::{GameType, TemplateType}};

type UUID = uuid::Uuid;

#[derive(Debug, PartialEq, Eq)]
#[repr(i16)]
pub enum GameEditState {
    NotSelected = 0,
    PlayerData = 1,
    OpponentData = 2,
    ResultData = 3,
}

#[derive(Debug, PartialEq, Eq)]
#[repr(i32)]
pub enum GameResult {
    NotSelected = 0,
    FirstPlayerWon = 1,
    SecondPlayerWon = 2,
}

#[derive(Debug, PartialEq, Eq)]
#[repr(i32)]
pub enum TournamentEditState {
    NotSelected = 0,
    ChannelsData = 1,
    ReportsData = 2,
}

#[derive(GraphQLQuery)]
#[graphql(
    schema_path = "src/graphql/schema.json",
    query_path = "src/graphql/queries/create_user.graphql",
    response_derives = "Debug"
)]
pub struct CreateUserMutation;

#[derive(GraphQLQuery)]
#[graphql(
    schema_path = "src/graphql/schema.json",
    query_path = "src/graphql/queries/create_tournament.graphql",
    response_derives = "Debug"
)]
pub struct CreateTournamentMutation;

#[derive(GraphQLQuery)]
#[graphql(
    schema_path = "src/graphql/schema.json",
    query_path = "src/graphql/queries/get_operator_section.graphql",
    response_derives = "Debug"
)]
pub struct GetOperatorSectionQuery;

#[derive(GraphQLQuery)]
#[graphql(
    schema_path = "src/graphql/schema.json",
    query_path = "src/graphql/queries/get_operator_data.graphql",
    response_derives = "Debug"
)]
pub struct GetOperatorDataQuery;

#[derive(GraphQLQuery)]
#[graphql(
    schema_path = "src/graphql/schema.json",
    query_path = "src/graphql/queries/get_tournament.graphql",
    response_derives = "Debug"
)]
pub struct GetTournamentQuery;
pub type GetTournamentResult = GetTournamentQueryTournament;

#[derive(GraphQLQuery)]
#[graphql(
    schema_path = "src/graphql/schema.json",
    query_path = "src/graphql/queries/get_user.graphql",
    response_derives = "Debug"
)]
pub struct GetUserQuery;

#[derive(GraphQLQuery)]
#[graphql(
    schema_path = "src/graphql/schema.json",
    query_path = "src/graphql/queries/create_match.graphql",
    response_derives = "Debug"
)]
pub struct CreateMatchMutation;

#[derive(GraphQLQuery)]
#[graphql(
    schema_path = "src/graphql/schema.json",
    query_path = "src/graphql/queries/update_match.graphql",
    response_derives = "Debug"
)]
pub struct UpdateMatch;

#[derive(GraphQLQuery)]
#[graphql(
    schema_path = "src/graphql/schema.json",
    query_path = "src/graphql/queries/get_match.graphql",
    response_derives = "Debug"
)]
pub struct GetMatchQuery;

#[derive(GraphQLQuery)]
#[graphql(
    schema_path = "src/graphql/schema.json",
    query_path = "src/graphql/queries/get_users.graphql",
    response_derives = "Debug"
)]
pub struct GetUsersQuery;

pub type GetUsersResult = GetUsersQueryUsers;

#[derive(GraphQLQuery)]
#[graphql(
    schema_path = "src/graphql/schema.json",
    query_path = "src/graphql/queries/update_user.graphql",
    response_derives = "Debug"
)]
pub struct UpdateUser;

// #[derive(GraphQLQuery)]
// #[graphql(
//     schema_path = "src/graphql/schema.json",
//     query_path = "src/graphql/queries/create_game.graphql",
//     response_derives = "Debug, PartialEq, Eq"
// )]
// pub struct CreateGameMutation;

// #[derive(GraphQLQuery)]
// #[graphql(
//     schema_path = "src/graphql/schema.json",
//     query_path = "src/graphql/queries/update_game.graphql",
//     response_derives = "Debug, PartialEq, Eq"
// )]
// pub struct UpdateGameMutation;

// #[derive(GraphQLQuery)]
// #[graphql(
//     schema_path = "src/graphql/schema.json",
//     query_path = "src/graphql/queries/get_game.graphql",
//     response_derives = "Debug, PartialEq, Eq"
// )]
// pub struct GetGameQuery;

#[derive(GraphQLQuery)]
#[graphql(
    schema_path = "src/graphql/schema.json",
    query_path = "src/graphql/queries/get_heroes.graphql",
    response_derives = "Debug"
)]
pub struct GetHeroesQuery;

#[derive(GraphQLQuery)]
#[graphql(
    schema_path = "src/graphql/schema.json",
    query_path = "src/graphql/queries/get_hero.graphql",
    response_derives = "Debug"
)]
pub struct GetHeroQuery;

// #[derive(GraphQLQuery)]
// #[graphql(
//     schema_path = "src/graphql/schema.json",
//     query_path = "src/graphql/queries/get_games.graphql",
//     response_derives = "Debug, PartialEq, Eq"
// )]
// pub struct GetGamesQuery;

#[derive(GraphQLQuery)]
#[graphql(
    schema_path = "src/graphql/schema.json",
    query_path = "src/graphql/queries/get_participant.graphql",
    response_derives = "Debug"
)]
pub struct GetParticipant;

// #[derive(GraphQLQuery)]
// #[graphql(
//     schema_path = "src/graphql/schema.json",
//     query_path = "src/graphql/queries/get_participants.graphql",
//     response_derives = "Debug"
// )]
// pub struct GetParticipants;

#[derive(GraphQLQuery)]
#[graphql(
    schema_path = "src/graphql/schema.json",
    query_path = "src/graphql/queries/create_participant.graphql",
    response_derives = "Debug"
)]
pub struct CreateParticipant;

#[derive(GraphQLQuery)]
#[graphql(
    schema_path = "src/graphql/schema.json",
    query_path = "src/graphql/queries/delete_participant.graphql",
    response_derives = "Debug"
)]
pub struct DeleteParticipant;

#[derive(GraphQLQuery)]
#[graphql(
    schema_path = "src/graphql/schema.json",
    query_path = "src/graphql/queries/create_organizer.graphql",
    response_derives = "Debug"
)]
pub struct CreateOrganizer;

#[derive(GraphQLQuery)]
#[graphql(
    schema_path = "src/graphql/schema.json",
    query_path = "src/graphql/queries/create_tournament_builder.graphql",
    response_derives = "Debug"
)]
pub struct CreateTournamentBuilder;

#[derive(GraphQLQuery)]
#[graphql(
    schema_path = "src/graphql/schema.json",
    query_path = "src/graphql/queries/get_organizer.graphql",
    response_derives = "Debug"
)]
pub struct GetOrganizer;

#[derive(GraphQLQuery)]
#[graphql(
    schema_path = "src/graphql/schema.json",
    query_path = "src/graphql/queries/get_tournament_builder.graphql",
    response_derives = "Debug, PartialEq, Eq"
)]
pub struct GetTournamentBuilder;

#[derive(GraphQLQuery)]
#[graphql(
    schema_path = "src/graphql/schema.json",
    query_path = "src/graphql/queries/update_tournament_builder.graphql",
    response_derives = "Debug, PartialEq, Eq"
)]
pub struct UpdateTournamentBuilder;

#[derive(GraphQLQuery)]
#[graphql(
    schema_path = "src/graphql/schema.json",
    query_path = "src/graphql/queries/get_tournaments.graphql",
    response_derives = "Debug"
)]
pub struct GetTournaments;

#[derive(GraphQLQuery)]
#[graphql(
    schema_path = "src/graphql/schema.json",
    query_path = "src/graphql/queries/update_tournament.graphql",
    response_derives = "Debug"
)]
pub struct UpdateTournament;

#[derive(GraphQLQuery)]
#[graphql(
    schema_path = "src/graphql/schema.json",
    query_path = "src/graphql/queries/get_tournament_users.graphql",
    response_derives = "Debug"
)]
pub struct GetTournamentUsers;

#[derive(GraphQLQuery)]
#[graphql(
    schema_path = "src/graphql/schema.json",
    query_path = "src/graphql/queries/update_participants_bulk.graphql",
    response_derives = "Debug"
)]
pub struct UpdateParticipantsBulk;

#[derive(GraphQLQuery)]
#[graphql(
    schema_path = "src/graphql/schema.json",
    query_path = "src/graphql/queries/create_games_bulk.graphql",
    response_derives = "Debug"
)]
pub struct CreateGamesBulk;

#[derive(GraphQLQuery)]
#[graphql(
    schema_path = "src/graphql/schema.json",
    query_path = "src/graphql/queries/update_users_bulk.graphql",
    response_derives = "Debug"
)]
pub struct UpdateUsersBulk;

#[derive(GraphQLQuery)]
#[graphql(
    schema_path = "src/graphql/schema.json",
    query_path = "src/graphql/queries/games_count.graphql",
    response_derives = "Debug"
)]
pub struct GamesCount;

// pub fn int_to_game_result(num: i32) -> update_game_mutation::GameResult {
//     match num {
//         1 => update_game_mutation::GameResult::FIRST_PLAYER_WON,
//         2 => update_game_mutation::GameResult::SECOND_PLAYER_WON,
//         _=> update_game_mutation::GameResult::NOT_SELECTED
//     }
// }

impl Into<update_tournament_builder::TournamentEditState>
    for get_tournament_builder::TournamentEditState
{
    fn into(self) -> update_tournament_builder::TournamentEditState {
        match self {
            get_tournament_builder::TournamentEditState::CHANNELS_DATA => {
                update_tournament_builder::TournamentEditState::CHANNELS_DATA
            }
            get_tournament_builder::TournamentEditState::NOT_SELECTED => {
                update_tournament_builder::TournamentEditState::NOT_SELECTED
            }
            get_tournament_builder::TournamentEditState::REPORTS_DATA => {
                update_tournament_builder::TournamentEditState::REPORTS_DATA
            }
            _ => update_tournament_builder::TournamentEditState::NOT_SELECTED,
        }
    }
}

impl Into<create_games_bulk::GameResult> for builders::types::GameResult {
    fn into(self) -> create_games_bulk::GameResult {
        match self {
            builders::types::GameResult::NotSelected => create_games_bulk::GameResult::NOT_SELECTED,
            builders::types::GameResult::FirstPlayerWon => {
                create_games_bulk::GameResult::FIRST_PLAYER_WON
            }
            builders::types::GameResult::SecondPlayerWon => {
                create_games_bulk::GameResult::SECOND_PLAYER_WON
            }
            _ => create_games_bulk::GameResult::NOT_SELECTED,
        }
    }
}

impl From<get_tournament_query::GameType> for GameType {
    fn from(value: get_tournament_query::GameType) -> Self {
        match value {
            get_tournament_query::GameType::ARENA => GameType::Arena,
            get_tournament_query::GameType::RMG => GameType::Rmg,
            _ => GameType::Arena,
        }
    }
}

impl Into<get_heroes_query::ModType> for h5_tournaments_api::prelude::ModType {
    fn into(self) -> get_heroes_query::ModType {
        match self {
            h5_tournaments_api::prelude::ModType::Hrta => get_heroes_query::ModType::HRTA,
            h5_tournaments_api::prelude::ModType::Universe => get_heroes_query::ModType::UNIVERSE,
            _ => get_heroes_query::ModType::UNIVERSE,
        }
    }
}

impl Into<create_games_bulk::GameOutcome> for crate::builders::types::GameOutcome {
    fn into(self) -> create_games_bulk::GameOutcome {
        match self {
            builders::types::GameOutcome::FinalBattleVictory => {
                create_games_bulk::GameOutcome::FINAL_BATTLE_VICTORY
            }
            builders::types::GameOutcome::NeutralsVictory => {
                create_games_bulk::GameOutcome::NEUTRALS_VICTORY
            }
            builders::types::GameOutcome::OpponentSurrender => {
                create_games_bulk::GameOutcome::OPPONENT_SURRENDER
            }
            _ => create_games_bulk::GameOutcome::FINAL_BATTLE_VICTORY,
        }
    }
}

impl Into<create_games_bulk::BargainsColor> for crate::builders::types::BargainsColor {
    fn into(self) -> create_games_bulk::BargainsColor {
        match self {
            builders::types::BargainsColor::NotSelected => {
                create_games_bulk::BargainsColor::NOT_SELECTED
            }
            builders::types::BargainsColor::BargainsColorBlue => {
                create_games_bulk::BargainsColor::BARGAINS_COLOR_BLUE
            }
            builders::types::BargainsColor::BargainsColorRed => {
                create_games_bulk::BargainsColor::BARGAINS_COLOR_RED
            }
        }
    }
}

impl From<CreateUserMutationCreateUser> for GetUserQueryUser {
    fn from(value: CreateUserMutationCreateUser) -> Self {
        GetUserQueryUser {
            nickname: value.nickname,
            id: value.id,
            registered: value.registered,
            discord_id: value.discord_id,
        }
    }
}

impl From<get_tournament_query::ModType> for ModType {
    fn from(value: get_tournament_query::ModType) -> Self {
        match value {
            get_tournament_query::ModType::HRTA => ModType::Hrta,
            get_tournament_query::ModType::UNIVERSE => ModType::Universe,
            _ => unreachable!(),
        }
    }
}

impl From<get_tournament_query::TemplateType> for TemplateType {
    fn from(value: get_tournament_query::TemplateType) -> Self {
        match value {
            get_tournament_query::TemplateType::ECHO_CASINO => TemplateType::EchoCasino,
            get_tournament_query::TemplateType::UNI_S_CASINO => TemplateType::UniSCasino,
            get_tournament_query::TemplateType::UNI_S_NEUTRAL_POOL => TemplateType::UniSNeutralPool,
            get_tournament_query::TemplateType::UNI_S_CENTRAL_RUSH => TemplateType::UniSCentralRush,
            get_tournament_query::TemplateType::UNI_S_NATIVE => TemplateType::UniSNative,
            get_tournament_query::TemplateType::JEBUS_CASINO => TemplateType::JebusCasino,
            get_tournament_query::TemplateType::JEBUS_NATIVE => TemplateType::JebusNative,
            get_tournament_query::TemplateType::JEBUS_MEGA_TREASURE => TemplateType::JebusMegaTreasure,
            get_tournament_query::TemplateType::MOON_CASINO => TemplateType::MoonCasino,
            get_tournament_query::TemplateType::MOON_NATIVE => TemplateType::MoonNative,
            get_tournament_query::TemplateType::MOON_CENTRAL_RUSH => TemplateType::MoonCentralRush,
            get_tournament_query::TemplateType::MOON_MEGA_TREASURE => TemplateType::MoonMegaTreasure,
            get_tournament_query::TemplateType::ECHO_NATIVE => TemplateType::EchoNative,
            get_tournament_query::TemplateType::ECHO_REDLINE => TemplateType::EchoRedline,
            get_tournament_query::TemplateType::ECHO_TIMELOCK => TemplateType::EchoTimelock,
            get_tournament_query::TemplateType::Other(_) => unreachable!(),
        }
    }
}

impl From<TemplateType> for create_games_bulk::TemplateType {
    fn from(val: TemplateType) -> Self {
        match val {
            TemplateType::UniSCasino => create_games_bulk::TemplateType::UNI_S_CASINO,
            TemplateType::UniSCentralRush => create_games_bulk::TemplateType::UNI_S_CENTRAL_RUSH,
            TemplateType::UniSNeutralPool => create_games_bulk::TemplateType::UNI_S_NEUTRAL_POOL,
            TemplateType::UniSNative => create_games_bulk::TemplateType::UNI_S_NATIVE,
            TemplateType::EchoCasino => create_games_bulk::TemplateType::ECHO_CASINO,
            TemplateType::EchoNative => create_games_bulk::TemplateType::ECHO_NATIVE,
            TemplateType::EchoRedline => create_games_bulk::TemplateType::ECHO_REDLINE,
            TemplateType::EchoTimelock => create_games_bulk::TemplateType::ECHO_TIMELOCK,
            TemplateType::MoonCasino => create_games_bulk::TemplateType::MOON_CASINO,
            TemplateType::MoonCentralRush => create_games_bulk::TemplateType::MOON_CENTRAL_RUSH,
            TemplateType::MoonMegaTreasure => create_games_bulk::TemplateType::MOON_MEGA_TREASURE,
            TemplateType::MoonNative => create_games_bulk::TemplateType::MOON_NATIVE,
            TemplateType::JebusCasino => create_games_bulk::TemplateType::JEBUS_CASINO,
            TemplateType::JebusMegaTreasure => create_games_bulk::TemplateType::JEBUS_MEGA_TREASURE,
            TemplateType::JebusNative => create_games_bulk::TemplateType::JEBUS_NATIVE,
        }
    }
}