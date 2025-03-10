use axum::{extract::{Path, Query, State}, routing::{get, patch, post}, Json, Router};
use uuid::Uuid;

use crate::services::tournament::prelude::*;

use super::models::{MatchRegistrationForm, TournamentCreationModel};

pub fn tournament_routes() -> Router<LegacyTournamentService> { 
    Router::new()
        .route("/tournament/create", post(create_tournament))
        .route("/tournament/get/{tournament_id}", get(get_tournament))
        .route("/tournament/matches/{tournament_id}", get(load_matches_for_tournament))
        .route("/tournament/games/{tournament_id}", get(load_all_games_for_tournament))
        .route("/tournaments", get(load_tournaments))
        .route("/races", get(load_races))
        .route("/heroes/{mod_type}", get(load_heroes))
        .route("/match/register", post(register_match))
        .route("/match/games", post(upload_games))
        .route("/match/games/{match_id}", get(load_games_for_match))
        .route("/game/create", post(create_game))
        .route("/game/update", patch(update_game))
        .route("/match/update", patch(update_match))
}

async fn create_tournament(
    State(tournament_service): State<LegacyTournamentService>,
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
    State(tournament_service): State<LegacyTournamentService>,
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
    State(tournament_service): State<LegacyTournamentService>
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
    State(tournament_service): State<LegacyTournamentService>,
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
    State(tournament_service): State<LegacyTournamentService>,
    Query(registration_form): Query<MatchRegistrationForm>
) -> Result<Json<i32>, ()> {

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
    State(tournament_service): State<LegacyTournamentService>,
    Json(games): Json<Vec<Game>>
) -> Result<(), ()> {

    tournament_service.upload_games(&games).await.unwrap();

    Ok(())
}

async fn load_tournaments(
    State(tournament_service): State<LegacyTournamentService>
) -> Result<Json<Vec<Tournament>>, ()> {
    Ok(Json(tournament_service.load_existing_tournaments().await.unwrap()))
}

async fn load_matches_for_tournament(
    State(tournament_service): State<LegacyTournamentService>,
    Path(tournament_id): Path<Uuid>
) -> Result<Json<Vec<Match>>, ()> {
    Ok(Json(tournament_service.load_matches_for_tournament(tournament_id).await.unwrap()))
}

async fn load_games_for_match(
    State(tournament_service): State<LegacyTournamentService>,
    Path(match_id): Path<Uuid>
) -> Result<Json<Vec<Game>>, ()> {
    Ok(Json(tournament_service.load_games_for_match(match_id).await.unwrap()))
}

async fn create_game(
    State(tournament_service): State<LegacyTournamentService>,
    Json(game): Json<Game>
) -> Result<(), ()> {
    tournament_service.create_game(game).await.unwrap();
    Ok(())
}

async fn update_game(
    State(tournament_service): State<LegacyTournamentService>,
    Json(game): Json<Game>
) -> Result<(), ()> {
    tournament_service.update_game(game).await.unwrap();
    Ok(())
}

async fn update_match(
    State(tournament_service): State<LegacyTournamentService>,
    Json(match_to_update): Json<Match>
) -> Result<(), ()> {
    tournament_service.update_match(match_to_update).await.unwrap();
    Ok(())
}

async fn load_all_games_for_tournament(
    State(tournament_service): State<LegacyTournamentService>,
    Path(tournament_id): Path<Uuid>
) -> Result<Json<Vec<Game>>, ()> {
    Ok(Json(tournament_service.get_all_games_for_tournament(tournament_id).await.unwrap()))
}