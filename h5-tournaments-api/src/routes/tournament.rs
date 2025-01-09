use axum::{extract::{Path, State}, routing::{get, post}, Json, Router};
use uuid::Uuid;

use crate::services::tournament::prelude::*;

use super::models::TournamentCreationModel;

pub fn tournament_routes() -> Router<TournamentService> { 
    Router::new()
        .route("/tournament/create", post(create_tournament))
        .route("/tournament/get/:tournament_id", get(get_tournament))
        .route("/races", get(load_races))
        .route("/heroes/:mod_type", get(load_heroes))
}

async fn create_tournament(
    State(tournament_service): State<TournamentService>,
    Json(creation_model): Json<TournamentCreationModel>
) -> Result<String, ()> {

    tracing::info!("Trying to access create_tournament route");
    tracing::info!("Got info: {:?}", &creation_model);
    // Ok("Ok?".to_string())

    let res = tournament_service.create_tournament(
        creation_model.mod_type,
        creation_model.server_id, 
        creation_model.channel_id,
        creation_model.first_message_id,
        creation_model.last_message_id,
        creation_model.name
    ).await;
    
    match res {
        Ok(res) => {
            tracing::info!("Tournament created with response: {}", &res);
            Ok(res)
        },
        Err(error) => {
            tracing::error!("Failed to create tournament: {:?}", error);
            Err(())
        }
    }
}

async fn get_tournament(
    State(tournament_service): State<TournamentService>,
    Path(tournament_id): Path<Uuid>
) -> Result<Json<Tournament>, ()> {

    tracing::info!("Got id of tournament: {}", tournament_id);
    let tournament = tournament_service.get_tournament_by_id(tournament_id).await;
    match tournament {
        Ok(tournament) => {
            Ok(Json(tournament))
        },
        Err(error) => {
            tracing::info!("Failed to get tournament with id {}: {}", tournament_id, error.to_string());
            Err(())
        }
    }
}

async fn load_races(
    State(tournament_service): State<TournamentService>
) -> Result<Json<Vec<Race>>, ()> {

    let races_data = tournament_service.load_races().await;
    
    match races_data {
        Ok(races) => {
            Ok(Json(races))
        },
        Err(_error) => {
            Err(())
        }
    }
}

async fn load_heroes(
    State(tournament_service): State<TournamentService>,
    Path(mod_type): Path<i16>
) -> Result<Json<Vec<Hero>>, ()> {
    
    let heroes_data = tournament_service.load_heroes_for_mod(ModType::from_repr(mod_type).unwrap()).await;
    
    match heroes_data {
        Ok(heroes) => {
            Ok(Json(heroes))
        },
        Err(_error) => {
            Err(())
        }
    }
}