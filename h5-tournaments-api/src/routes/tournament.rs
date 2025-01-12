use axum::{extract::{Path, Query, State}, routing::{get, post}, Json, Router};
use uuid::Uuid;

use crate::services::tournament::prelude::*;

use super::models::{MatchRegistrationForm, TournamentCreationModel};

pub fn tournament_routes() -> Router<TournamentService> { 
    Router::new()
        .route("/tournament/create", post(create_tournament))
        .route("/tournament/get/:tournament_id", get(get_tournament))
        .route("/races", get(load_races))
        .route("/heroes/:mod_type", get(load_heroes))
        .route("/match/register", post(register_match))
        .route("/match/games", post(upload_games))
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
            tracing::info!("Failed to get races: {}", _error.to_string());
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
            tracing::info!("Heroes fetched correctly for mod {}: {:?}", mod_type, &heroes);
            Ok(Json(heroes))
        },
        Err(_error) => {
            tracing::error!("Failed to fetch heroes for mod {}: {}", mod_type, _error.to_string());
            Err(())
        }
    }
}

async fn register_match(
    State(tournament_service): State<TournamentService>,
    Query(registration_form): Query<MatchRegistrationForm>
) -> Result<Json<Uuid>, ()> {

    let registration_result = tournament_service.register_match(&registration_form).await;

    match registration_result {
        Ok(success) => {
            tracing::info!("Match registered with id {}", success);
            Ok(Json(success))
        },
        Err(_error) => {
            tracing::error!("Failed to register match: {}", _error.to_string());
            Err(())
        }
    }

}

async fn upload_games(
    State(tournament_service): State<TournamentService>,
    Json(games): Json<Vec<Game>>
) -> Result<(), ()> {

    tournament_service.upload_games(&games).await.unwrap();

    Ok(())
}