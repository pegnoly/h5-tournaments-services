use axum::{routing::get, Router};
use sqlx::PgPool;
use tournament::{management::management_routes, statistics::statistics_routes};

pub mod tournament;
pub mod utils;

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
        .merge(management_routes())
        .merge(statistics_routes())
        .with_state(manager);

    Ok(router.into())
}
