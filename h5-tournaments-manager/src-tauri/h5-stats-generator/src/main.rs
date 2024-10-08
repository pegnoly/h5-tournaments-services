use h5_stats_generator::{builder::{pair::PairStatsBuilder, player::PlayersStatsBuilder, race::RaceStatsBuilder, StatsBuilder}, utils::StatsGeneratorDataModel};
use h5_stats_types::{Game, Hero, Match, Race};
use reqwest::Client;
use rust_xlsxwriter::Workbook;
use uuid::{uuid, Uuid};

#[tokio::main]
async fn main() {
    let mut data_model = StatsGeneratorDataModel::new();
    let tournament_id = uuid!("9cf9179a-afbb-4d81-b591-9e44e6e480ab");
    let mut workbook = Workbook::new();

    let mut builders: Vec<Box<dyn StatsBuilder>> = vec![Box::new(PairStatsBuilder::new()), Box::new(RaceStatsBuilder::new()), Box::new(PlayersStatsBuilder{})];

    load_data(&mut data_model, tournament_id).await;

    for builder in &mut builders {
        builder.build(&data_model, &mut workbook);
    }

    let path = std::env::current_exe().unwrap().parent().unwrap().join("test.xlsx");
    workbook.save(path).unwrap();
}

async fn load_data(generator: &mut StatsGeneratorDataModel, id: Uuid) {
    let client = Client::new();
    load_heroes(&client, generator).await;
    load_races(&client, generator).await;
    load_matches(&client, generator, id).await;
    load_games(&client, generator, id).await;
}

async fn load_matches(client: &Client, generator: &mut StatsGeneratorDataModel, tournament_id: Uuid) {
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

async fn load_games(client: &Client, generator: &mut StatsGeneratorDataModel, tournament_id: Uuid) {
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

async fn load_heroes(client: &Client, generator: &mut StatsGeneratorDataModel) {
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

async fn load_races(client: &Client, generator: &mut StatsGeneratorDataModel) {
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

