use h5_stats_generator::utils::StatsGenerator;
use h5_stats_types::{Game, Hero, Match, Race};
use reqwest::Client;
use uuid::{uuid, Uuid};

#[tokio::main]
async fn main() {
    let mut generator = StatsGenerator::new();
    let tournament_id = uuid!("9cf9179a-afbb-4d81-b591-9e44e6e480ab");
    load_data(&mut generator, tournament_id).await;
    generator.process();
    generator.save();
}

async fn load_data(generator: &mut StatsGenerator, id: Uuid) {
    let client = Client::new();
    load_heroes(&client, generator).await;
    load_races(&client, generator).await;
    load_matches(&client, generator, id).await;
    load_games(&client, generator, id).await;
}

async fn load_matches(client: &Client, generator: &mut StatsGenerator, tournament_id: Uuid) {
    let response = client.get(format!("https://h5-tournaments-api.shuttleapp.rs/matches/{}", &tournament_id))
        .send()
        .await;
    match response {
        Ok(success) => {
            let json: Result<Vec<Match>, reqwest::Error> = success.json().await;
            match json {
                Ok(matches) => {
                    generator.matches_data = matches;
                },
                Err(json_error) => {
                    println!("Failed to parse matches json: {}", json_error.to_string());
                }
            }
        },
        Err(failure) => {
            println!("Failed to send existing matches request: {}", failure.to_string());
        }
    }
}

async fn load_games(client: &Client, generator: &mut StatsGenerator, tournament_id: Uuid) {
    let response = client.get(format!("https://h5-tournaments-api.shuttleapp.rs/games/by_tournament/{}", &tournament_id))
        .send()
        .await;
    match response {
        Ok(success) => {
            let json: Result<Vec<Game>, reqwest::Error> = success.json().await;
            match json {
                Ok(games) => {
                    println!("Got games for stats count: {}", &games.len());
                    generator.games_data = games;
                },
                Err(json_error) => {
                    println!("Failed to parse games json: {}", json_error.to_string());
                }
            }
        },
        Err(failure) => {
            println!("Failed to send games request: {}", failure.to_string());
        }
    }
}

async fn load_heroes(client: &Client, generator: &mut StatsGenerator) {
    let response = client.get("https://h5-tournaments-api.shuttleapp.rs/heroes")
        .send()
        .await;
    match response {
        Ok(success) => {
            let json: Result<Vec<Hero>, reqwest::Error> = success.json().await;
            match json {
                Ok(heroes) => {
                    generator.heroes_data = heroes;
                },
                Err(json_error) => {
                    println!("Failed to parse heroes json: {}", json_error.to_string());
                }
            }
        },
        Err(failure) => {
            println!("Failed to send existing heroes request: {}", failure.to_string());
        }
    }
}

async fn load_races(client: &Client, generator: &mut StatsGenerator) {
    let response = client.get("https://h5-tournaments-api.shuttleapp.rs/races")
        .send()
        .await;
    match response {
        Ok(success) => {
            let json: Result<Vec<Race>, reqwest::Error> = success.json().await;
            match json {
                Ok(races) => {
                    generator.races_data = races;
                },
                Err(json_error) => {
                    println!("Failed to parse races json: {}", json_error.to_string());
                }
            }
        },
        Err(failure) => {
            println!("Failed to send existing races request: {}", failure.to_string());
        }
    }
}

