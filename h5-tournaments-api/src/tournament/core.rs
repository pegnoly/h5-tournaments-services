use axum::{extract::{Path, Query, State}, http::StatusCode, response::IntoResponse, Json};
use serde::{Deserialize, Serialize};

use crate::ApiManager;

use super::utils::{Hero, Race};

#[derive(Debug, Serialize, Deserialize, sqlx::FromRow, Default)]
pub struct TournamentCreationModel {
    pub id: String,
    pub server_id: i64,
    pub channel_id: i64,
    pub name: String
}

#[derive(Debug, Serialize, Deserialize)]
pub struct SingleTournamentQueryModel {
    pub id: Option<String>,
    pub channel_id: Option<i64>
}

pub async fn create_tournament(
    State(api_manager): State<ApiManager>,
    Json(tournament_creation_model): Json<TournamentCreationModel>
) -> Result<String, String> {
    tracing::info!("Got json payload to create tournament: {:?}", &tournament_creation_model);
    let id = uuid::Uuid::new_v4().to_string().replace("-", "");
    let res = sqlx::query(r#"
            INSERT INTO tournaments 
            (id, server_id, channel_id, name)
            VALUES ($1, $2, $3, $4);
        "#)
        .bind(&id)
        .bind(tournament_creation_model.server_id as i64)
        .bind(tournament_creation_model.channel_id as i64)
        .bind(tournament_creation_model.name)
        .execute(&api_manager.pool)
        .await;
    match res {
        Ok(_success) => {
            tracing::info!("Tournament successfully inserted");
            Ok(format!("Tournament successfully created with id {}", id))
        },
        Err(failure) => {
            tracing::info!("Failed to insert tournament: {}", failure.to_string());
            Err(format!("Failed to create tournament: {}", failure.to_string()))
        }
    }
}

pub async fn get_tournament(
    State(api_manager): State<ApiManager>,
    Query(model): Query<SingleTournamentQueryModel>
) -> (StatusCode, Json<TournamentCreationModel>) {
    tracing::info!("We are here with params {:?}", &model);
    let res: Result<Option<TournamentCreationModel>, sqlx::Error> = sqlx::query_as(r#"
            SELECT * FROM tournaments WHERE id=$1 OR channel_id=$2;
        "#)
        .bind(model.id)
        .bind(model.channel_id)
        .fetch_optional(&api_manager.pool)
        .await;
    match res {
        Ok(optional_model) => {
            match optional_model {
                Some(tournament) => {
                    tracing::info!("Got tournament: {:?}", &tournament);
                    (StatusCode::OK, Json(tournament))
                },
                None => {
                    tracing::info!("No such tournament");
                    (StatusCode::NO_CONTENT, Json(TournamentCreationModel::default()))
                }
            }
        },
        Err(failure) => {
            tracing::info!("Failed to fetch tournament: {}", failure.to_string());
            (StatusCode::BAD_REQUEST, Json(TournamentCreationModel::default()))
        }
    }
}

pub async fn get_races(
    State(api_manager): State<ApiManager>
) -> Result<Json<Vec<Race>>, ()> {
    let res: Result<Vec<Race>, sqlx::Error> = sqlx::query_as("SELECT * FROM races;")
        .fetch_all(&api_manager.pool)
        .await;
    match res {
        Ok(races) => {
            Ok(Json(races))
        },
        Err(e) => {
            tracing::info!("Failed to fetch existing races: {}", e.to_string());
            Err(())
        }
    }
}

pub async fn get_heroes(
    State(api_manager): State<ApiManager>
) -> Result<Json<Vec<Hero>>, ()> {
    let res: Result<Vec<Hero>, sqlx::Error> = sqlx::query_as("SELECT * FROM heroes;")
    .fetch_all(&api_manager.pool)
    .await;
match res {
    Ok(heroes) => {
        Ok(Json(heroes))
    },
    Err(e) => {
        tracing::info!("Failed to fetch existing heroes: {}", e.to_string());
        Err(())
    }
} 
}