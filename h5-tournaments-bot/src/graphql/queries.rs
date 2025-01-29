use get_match_query::GetMatchQueryTournamentMatch;
use get_tournament_query::GetTournamentQueryTournament;
use get_users_query::GetUsersQueryUsers;
use graphql_client::GraphQLQuery;

type UUID = uuid::Uuid;

#[derive(Debug, PartialEq, Eq)]
#[repr(i16)]
pub enum GameEditState {
    NotSelected = 0,
    PlayerData = 1,
    OpponentData = 2,
    ResultData = 3
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
pub struct UpdateMatchMutation;

#[derive(GraphQLQuery)]
#[graphql(
    schema_path = "src/graphql/schema.json",
    query_path = "src/graphql/queries/get_match.graphql",
    response_derives = "Debug"
)]
pub struct GetMatchQuery;

pub type GetMatchResult = GetMatchQueryTournamentMatch;


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
    query_path = "src/graphql/queries/create_game.graphql",
    response_derives = "Debug, PartialEq, Eq"
)]
pub struct CreateGameMutation;

#[derive(GraphQLQuery)]
#[graphql(
    schema_path = "src/graphql/schema.json",
    query_path = "src/graphql/queries/update_game.graphql",
    response_derives = "Debug, PartialEq, Eq"
)]
pub struct UpdateGameMutation;

#[derive(GraphQLQuery)]
#[graphql(
    schema_path = "src/graphql/schema.json",
    query_path = "src/graphql/queries/get_game.graphql",
    response_derives = "Debug, PartialEq, Eq"
)]
pub struct GetGameQuery;

#[derive(GraphQLQuery)]
#[graphql(
    schema_path = "src/graphql/schema.json",
    query_path = "src/graphql/queries/get_heroes.graphql",
    response_derives = "Debug"
)]
pub struct GetHeroesQuery;