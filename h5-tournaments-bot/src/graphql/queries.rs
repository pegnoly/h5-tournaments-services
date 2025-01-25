use graphql_client::GraphQLQuery;

type UUID = uuid::Uuid;

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