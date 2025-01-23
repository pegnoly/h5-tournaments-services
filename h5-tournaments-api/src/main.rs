use std::sync::Arc;

use async_graphql::{http::GraphiQLSource, EmptySubscription, Schema};
use async_graphql_axum::GraphQL;
use axum::{response::{Html, IntoResponse}, routing::get, Router};
use h5_tournaments_api::{graphql::{mutation::Mutation, query::Query}, prelude::*};
use sea_orm::SqlxPostgresConnector;
use sqlx::PgPool;


async fn graphiql() -> impl IntoResponse {
    Html(
        GraphiQLSource::build().endpoint("/").subscription_endpoint("/ws").finish(),
    )
}

#[derive(Clone)]
pub struct Services {
    pub tournament_service: Arc<LegacyTournamentService>
}

#[shuttle_runtime::main]
async fn main(
    #[shuttle_shared_db::Postgres] pool: PgPool
) -> shuttle_axum::ShuttleAxum {

    let db = SqlxPostgresConnector::from_sqlx_postgres_pool(pool.clone());
    let schema = Schema::build(Query, Mutation, EmptySubscription)
        .data(db)
        .data(TournamentService {})
        .finish();

    let router = Router::new()
        .route("/", get(graphiql).post_service(GraphQL::new(schema.clone())))
        .merge(tournament_routes())
        //.merge(statistics_routes())
        .with_state(LegacyTournamentService {pool: pool.clone()});

    Ok(router.into())
}
