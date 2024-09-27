use axum::{routing::{get, post}, Router};
use sqlx::PgPool;
use tournament::core::{create_tournament, get_heroes, get_races, get_tournament};

pub mod tournament;

async fn hello_world() -> &'static str {
    "Hello, world!"
}

#[derive(Clone)]
pub struct ApiManager {
    pub pool: PgPool
}

#[shuttle_runtime::main]
async fn main(
    #[shuttle_shared_db::Postgres] pool: PgPool
) -> shuttle_axum::ShuttleAxum {
    let manager = ApiManager {
        pool: pool
    };
    let router = Router::new()
        .route("/", get(hello_world))
        .route("/tournament/create", post(create_tournament))
        .route("/tournament", get(get_tournament))
        .route("/races", get(get_races))
        .route("/heroes", get(get_heroes))
        .with_state(manager);

    Ok(router.into())
}
