use std::sync::Arc;

use axum::{routing::get, Router};
use h5_tournaments_api::prelude::*;
use sqlx::PgPool;

async fn hello_world() -> &'static str {
    "Hello, world!"
}

#[derive(Clone)]
pub struct Services {
    pub tournament_service: Arc<TournamentService>
}

#[shuttle_runtime::main]
async fn main(
    #[shuttle_shared_db::Postgres] pool: PgPool
) -> shuttle_axum::ShuttleAxum {

    let router = Router::new()
        .route("/", get(hello_world))
        .merge(tournament_routes())
        //.merge(statistics_routes())
        .with_state(TournamentService {pool: pool.clone()});

    Ok(router.into())
}
