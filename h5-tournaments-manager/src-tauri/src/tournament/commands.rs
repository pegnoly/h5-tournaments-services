use h5_stats_generator::utils::StatsGenerator;
use h5_stats_types::{Game, Hero, Match, Race, Tournament};
use reqwest::Client;
use tauri::State;
use tokio::sync::{Mutex, RwLock};
use uuid::Uuid;

use super::utils::{GameFrontendModel, HeroFrontendModel, RaceFrontendModel, TournamentFrontendModel};

pub struct LocalAppManager {
    client: RwLock<Client>,
    generator: Mutex<StatsGenerator>
}

impl LocalAppManager {
    pub fn new() -> Self {
        LocalAppManager { 
            client: RwLock::new(Client::new()),
            generator: Mutex::new(StatsGenerator::new()) 
        }
    }
}

#[tauri::command]
pub async fn load_heroes(
    app_manager: State<'_, LocalAppManager>
) -> Result<Vec<HeroFrontendModel>, ()> {
    let client = app_manager.client.read().await;
    let response = client.get("https://h5-tournaments-api.shuttleapp.rs/heroes")
        .send()
        .await;
    match response {
        Ok(success) => {
            let json: Result<Vec<Hero>, reqwest::Error> = success.json().await;
            match json {
                Ok(heroes) => {
                    let mut generator_locked = app_manager.generator.lock().await;
                    generator_locked.heroes_data = heroes.clone();
                    Ok(heroes.into_iter().map(|h| HeroFrontendModel::from(h)).collect())
                },
                Err(json_error) => {
                    println!("Failed to parse heroes json: {}", json_error.to_string());
                    Err(())
                }
            }
        },
        Err(failure) => {
            println!("Failed to send existing heroes request: {}", failure.to_string());
            Err(())
        }
    }
}

#[tauri::command]
pub async fn load_races(
    app_manager: State<'_, LocalAppManager>
) -> Result<Vec<RaceFrontendModel>, ()> {
    let client = app_manager.client.read().await;
    let response = client.get("https://h5-tournaments-api.shuttleapp.rs/races")
        .send()
        .await;
    match response {
        Ok(success) => {
            let json: Result<Vec<Race>, reqwest::Error> = success.json().await;
            match json {
                Ok(races) => {
                    let mut generator_locked = app_manager.generator.lock().await;
                    generator_locked.races_data = races.clone();
                    Ok(races.into_iter().map(|r| RaceFrontendModel::from(r)).collect())
                },
                Err(json_error) => {
                    println!("Failed to parse races json: {}", json_error.to_string());
                    Err(())
                }
            }
        },
        Err(failure) => {
            println!("Failed to send existing races request: {}", failure.to_string());
            Err(())
        }
    }
}

#[tauri::command]
pub async fn load_tournaments(
    app_manager: State<'_, LocalAppManager>
) -> Result<Vec<TournamentFrontendModel>, ()> {
    let client = app_manager.client.read().await;
    let response = client.get("https://h5-tournaments-api.shuttleapp.rs/tournaments")
        .send()
        .await;
    match response {
        Ok(success) => {
            let json: Result<Vec<Tournament>, reqwest::Error> = success.json().await;
            match json {
                Ok(tournaments) => {
                    Ok(tournaments.into_iter().map(|t| TournamentFrontendModel::from(t)).collect())
                },
                Err(json_error) => {
                    println!("Failed to parse tournaments json: {}", json_error.to_string());
                    Err(())
                }
            }
        },
        Err(failure) => {
            println!("Failed to send existing tournaments request: {}", failure.to_string());
            Err(())
        }
    }
}

#[tauri::command]
pub async fn load_matches(
    app_manager: State<'_, LocalAppManager>,
    tournament_id: Uuid
) -> Result<Vec<Match>, ()> {
    let client = app_manager.client.read().await;
    let response = client.get(format!("https://h5-tournaments-api.shuttleapp.rs/matches/{}", &tournament_id))
        .send()
        .await;
    match response {
        Ok(success) => {
            let json: Result<Vec<Match>, reqwest::Error> = success.json().await;
            match json {
                Ok(matches) => {
                    //println!("Got existing matches for tournament: {:?}", &matches);
                    let mut generator_locked = app_manager.generator.lock().await;
                    generator_locked.matches_data = matches.clone();
                    Ok(matches)
                },
                Err(json_error) => {
                    println!("Failed to parse matches json: {}", json_error.to_string());
                    Err(())
                }
            }
        },
        Err(failure) => {
            println!("Failed to send existing matches request: {}", failure.to_string());
            Err(())
        }
    }
}

#[tauri::command]
pub async fn load_games(
    app_manager: State<'_, LocalAppManager>,
    match_id: Uuid
) -> Result<Vec<GameFrontendModel>, ()> {
    let client = app_manager.client.read().await;
    let response = client.get(format!("https://h5-tournaments-api.shuttleapp.rs/games/{}", &match_id))
        .send()
        .await;
    match response {
        Ok(success) => {
            let json: Result<Vec<Game>, reqwest::Error> = success.json().await;
            match json {
                Ok(games) => {
                    //println!("Got existing games for tournament: {:?}", &games);
                    Ok(games.into_iter().map(|g| GameFrontendModel::from(g)).collect())
                },
                Err(json_error) => {
                    println!("Failed to parse games json: {}", json_error.to_string());
                    Err(())
                }
            }
        },
        Err(failure) => {
            println!("Failed to send existing games request: {}", failure.to_string());
            Err(())
        }
    }
}

#[tauri::command]
pub async fn create_game(
    app_manager: State<'_, LocalAppManager>,
    match_id: Uuid
) -> Result<(), ()> {
    let mut new_game = Game::default();
    new_game.match_id = match_id;
    let client = app_manager.client.read().await;
    let response = client.post("https://h5-tournaments-api.shuttleapp.rs/game")
        .json(&new_game)
        .send()
        .await;
    match response {
        Ok(success) => {
            println!("Game created");
            Ok(())
        },
        Err(failure) => {
            print!("Failed to create game: {}", failure.to_string());
            Err(())
        }
    }
}

#[tauri::command]
pub async fn update_game(
    app_manager: State<'_, LocalAppManager>,
    game: GameFrontendModel 
) -> Result<(), ()> {
    let client = app_manager.client.read().await;
    let response = client.patch("https://h5-tournaments-api.shuttleapp.rs/game")
        .json(&game)
        .send()
        .await;
    match response {
        Ok(success) => {
            println!("Game with id {} updated successfully", game.id);
            Ok(())
        },
        Err(failure) => {
            print!("Failed to update game with id {}: {}", game.id, failure.to_string());
            Err(())
        }
    }
}

#[tauri::command]
pub async fn update_match(
    app_manager: State<'_, LocalAppManager>,
    match_to_update: Match 
) -> Result<(), ()> {
    println!("Got match to update: {:?}", &match_to_update);
    let client = app_manager.client.read().await;
    let response = client.patch("https://h5-tournaments-api.shuttleapp.rs/match")
        .json(&match_to_update)
        .send()
        .await;
    match response {
        Ok(success) => {
            println!("Match with id {} updated successfully", match_to_update.id);
            Ok(())
        },
        Err(failure) => {
            print!("Failed to update match with id {}: {}", match_to_update.id, failure.to_string());
            Err(())
        }
    }
}

#[tauri::command]
pub async fn load_games_for_stats(
    app_manager: State<'_, LocalAppManager>,
    tournament_id: Uuid
) -> Result<(), ()> {
    let client = app_manager.client.read().await;
    let response = client.get(format!("https://h5-tournaments-api.shuttleapp.rs/games/by_tournament/{}", &tournament_id))
        .send()
        .await;
    match response {
        Ok(success) => {
            let json: Result<Vec<Game>, reqwest::Error> = success.json().await;
            match json {
                Ok(games) => {
                    println!("Got games for stats count: {}", &games.len());
                    let mut generator_locked = app_manager.generator.lock().await;
                    generator_locked.games_data = games;
                    generator_locked.process(tournament_id.to_string());
                    generator_locked.save();
                    Ok(())
                },
                Err(json_error) => {
                    println!("Failed to parse games json: {}", json_error.to_string());
                    Err(())
                }
            }
        },
        Err(failure) => {
            println!("Failed to send games request: {}", failure.to_string());
            Err(())
        }
    }
}