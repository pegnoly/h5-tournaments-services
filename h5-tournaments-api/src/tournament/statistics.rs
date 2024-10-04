use axum::{extract::{Query, State}, http::StatusCode, Router};

use crate::{utils::queries::RacesPairQueryModel, ApiManager};

pub(crate) fn statistics_routes() -> Router<ApiManager> {
    Router::new()
}