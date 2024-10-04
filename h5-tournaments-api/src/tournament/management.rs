use axum::{extract::{Path, Query, State}, http::StatusCode, routing::{get, patch, post}, Json, Router};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

use crate::{utils::queries::{MatchQueryModel, TournamentQueryModel}, ApiManager};

use super::utils::{Game, Hero, Match, Race, Tournament, TournamentCreationModel};

pub(crate) fn management_routes() -> Router<ApiManager> {
    Router::new()
        .route("/tournament", post(create_tournament))
        .route("/tournament", get(get_tournament))
        .route("/races", get(get_races))
        .route("/heroes", get(get_heroes))
        .route("/match", post(create_match))
        .route("/game", post(create_game))
        .route("/tournaments", get(get_tournaments))
        .route("/matches/:tournament_id", get(get_matches))
        .route("/games/:match_id", get(get_games))
        .route("/match", patch(update_match))
        .route("/game", patch(update_game))
}

async fn create_tournament(
    State(api_manager): State<ApiManager>,
    Query(tournament_creation_model): Query<TournamentCreationModel>
) -> Result<String, String> {
    tracing::info!("Got json payload to create tournament: {:?}", &tournament_creation_model);
    let id = uuid::Uuid::new_v4();
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

async fn get_tournament(
    State(api_manager): State<ApiManager>,
    Query(model): Query<TournamentQueryModel>
) -> (StatusCode, Json<Tournament>) {
    let res: Result<Option<Tournament>, sqlx::Error> = sqlx::query_as(r#"
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
                    (StatusCode::NO_CONTENT, Json(Tournament::default()))
                }
            }
        },
        Err(failure) => {
            tracing::info!("Failed to fetch tournament: {}", failure.to_string());
            (StatusCode::BAD_REQUEST, Json(Tournament::default()))
        }
    }
}

async fn get_races(
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

async fn get_heroes(
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


async fn create_match(
    State(api_manager): State<ApiManager>,
    Query(model): Query<MatchQueryModel>
) -> Result<Json<Match>, ()> {
    let match_to_create = Match {
        id: uuid::Uuid::new_v4(),
        message: model.message as i64,
        tournament_id: model.tournament,
        first_player: model.first_player,
        second_player: model.second_player
    };
    tracing::info!("Match to create: {:?}", &match_to_create);
    let res = sqlx::query(
        r#"
                INSERT INTO matches
                (id, tournament_id, message, first_player, second_player)
                VALUES($1, $2, $3, $4, $5)
                ON CONFLICT(message) DO NOTHING;
            "#)
            .bind(&match_to_create.id)
            .bind(&match_to_create.tournament_id)
            .bind(match_to_create.message)
            .bind(&match_to_create.first_player)
            .bind(&match_to_create.second_player)
            .execute(&api_manager.pool)
            .await;
    match res {
        Ok(_success) => {
            Ok(Json(match_to_create))
        },
        Err(_failure) => {
            tracing::info!("Failed to insert match: {}", _failure.to_string());
            Err(())
        }
    }
}

async fn create_game(
    State(api_manager): State<ApiManager>,
    Json(game_data): Json<Game>
) -> Result<(), ()> {
    let res = sqlx::query(
    r#"
            INSERT INTO games
            (id, match_id, first_player_race, first_player_hero, second_player_race, second_player_hero, bargains_color, bargains_amount, result)
            VALUES($1, $2, $3, $4, $5, $6, $7, $8, $9);
        "#)
        .bind(game_data.id)
        .bind(game_data.match_id)
        .bind(game_data.first_player_race)
        .bind(game_data.first_player_hero)
        .bind(game_data.second_player_race)
        .bind(game_data.second_player_hero)
        .bind(game_data.bargains_color)
        .bind(game_data.bargains_amount)
        .bind(game_data.result)
        .execute(&api_manager.pool)
        .await;
    match res {
        Ok(_success) => {
            tracing::info!("Game inserted successfully");
            Ok(())
        },
        Err(failure) => {
            tracing::info!("Failed to insert game: {}", failure.to_string());
            Err(())
        }
    }
}

async fn get_tournaments(
    State(api_manager): State<ApiManager>
) -> Result<Json<Vec<Tournament>>, ()> {
    let res: Result<Vec<Tournament>, sqlx::Error> = sqlx::query_as(
        r#"
            SELECT * FROM tournaments;
        "#)
        .fetch_all(&api_manager.pool)
        .await;
    match res {
        Ok(tournaments) => {
            //tracing::info!("Got tournaments: {:?}", &tournaments);
            Ok(Json(tournaments))
        },
        Err(failure) => {
            tracing::error!("Failed to get existing tournaments: {}", failure.to_string());
            Err(())
        }
    }
}

async fn get_matches(
    State(api_manager): State<ApiManager>,
    Path(tournament_id): Path<Uuid>
) -> Result<Json<Vec<Match>>, ()> {
    let res: Result<Vec<Match>, sqlx::Error> = sqlx::query_as(
        r#"
            SELECT * FROM matches WHERE tournament_id=$1;
        "#)
        .bind(&tournament_id)
        .fetch_all(&api_manager.pool)
        .await;
    match res {
        Ok(matches) => {
            //tracing::info!("Got matches: {:?}", &matches);
            Ok(Json(matches))
        },
        Err(failure) => {
            tracing::info!("Failed to fetch matches for tournament {}: {}", &tournament_id, failure.to_string());
            Err(())
        }
    }
}

async fn get_games(
    State(api_manager): State<ApiManager>,
    Path(match_id): Path<Uuid>
) -> Result<Json<Vec<Game>>, ()> {
    let res: Result<Vec<Game>, sqlx::Error> = sqlx::query_as(
    r#"
            SELECT * FROM games WHERE match_id=$1;
        "#)
        .bind(&match_id)
        .fetch_all(&api_manager.pool)
        .await;
    match res {
        Ok(games) => {
            Ok(Json(games))
        },
        Err(failure) => {
            tracing::info!("Failed to fetch games for match {}: {}", &match_id, failure.to_string());
            Err(())
        }
    }
}

#[derive(Debug, Serialize, Deserialize, Default)]
pub struct MatchUpdateModel {
    pub id: Uuid,
    pub tournament_id: Uuid,
    pub first_player: String,
    pub second_player: String
}

async fn update_match(
    State(api_manager): State<ApiManager>,
    Json(match_data): Json<MatchUpdateModel>
) -> StatusCode {
    tracing::info!("Here with data: {:?}", &match_data);
    let res: Result<Match, sqlx::Error> = sqlx::query_as(
    r#"
            UPDATE matches
            SET first_player=$1, second_player=$2
            WHERE id=$3
            RETURNING *;
        "#)
        .bind(&match_data.first_player)
        .bind(&match_data.second_player)
        .bind(&match_data.id)
        .fetch_one(&api_manager.pool)
        .await;
    match res {
        Ok(_) => {
            tracing::info!("Match {} updated successfully", &match_data.id);
            StatusCode::OK
        },
        Err(failure) => {
            tracing::info!("Failed to update match {}: {}", &match_data.id, failure.to_string());
            StatusCode::NO_CONTENT
        }
    }
}


#[derive(Debug, Serialize, Deserialize)]
pub struct GameUpdateModel {
    pub id: Uuid,
    pub first_player_race: i32,
    pub first_player_hero: i32,
    pub second_player_race: i32,
    pub second_player_hero: i32,
    pub bargains_color: i16,
    pub bargains_amount: i16,
    pub result: i16
}

async fn update_game(
    State(api_manager): State<ApiManager>,
    Json(game_data): Json<GameUpdateModel>
) -> StatusCode {
    let res: Result<Game, sqlx::Error> = sqlx::query_as(
    r#"
            UPDATE games
            SET first_player_race=$1, first_player_hero=$2, second_player_race=$3, second_player_hero=$4, bargains_color=$5, bargains_amount=$6, result=$7
            WHERE id=$8
            RETURNING *;
        "#)
        .bind(&game_data.first_player_race)
        .bind(&game_data.first_player_hero)
        .bind(&game_data.second_player_race)
        .bind(&game_data.second_player_hero)
        .bind(&game_data.bargains_color)
        .bind(&game_data.bargains_amount)
        .bind(&game_data.result)
        .bind(&game_data.id)
        .fetch_one(&api_manager.pool)
        .await;
    match res {
        Ok(_) => {
            tracing::info!("Game {} updated successfully", &game_data.id);
            StatusCode::OK
        },
        Err(failure) => {
            tracing::info!("Failed to update game {}: {}", &game_data.id, failure.to_string());
            StatusCode::NO_CONTENT
        }
    }
}