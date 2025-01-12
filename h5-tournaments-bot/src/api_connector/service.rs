use std::str::FromStr;

use h5_tournaments_api::prelude::{Hero, ModType, Race, Tournament};
use uuid::Uuid;

use crate::parser::service::ParsedData;

pub(self) const MAIN_URL: &'static str = "https://h5-tournaments-api-5epg.shuttle.app/";

pub struct ApiConnectionService {
    client: tokio::sync::RwLock<reqwest::Client>
}

impl ApiConnectionService {
    pub fn new(client: reqwest::Client) -> Self {
        ApiConnectionService {
            client: tokio::sync::RwLock::new(client)
        }
    }

    pub async fn init_tournament(&self, tournament_data: &serde_json::Value) -> Result<String, crate::Error> {
        let client = self.client.read().await;

        let response = client
            .post(format!("{}tournament/create", MAIN_URL))
            .json(tournament_data)
            .send()
            .await;

        match response {
            Ok(success) => {
                tracing::info!("Tournament creation response: {:?}", &success);
                let text = success.text().await.unwrap();
                Ok(text)
            },
            Err(failure) => {
                tracing::error!("Failed to send tournament creation request: {}", failure.to_string());
                Err(crate::Error::from(failure))
            }
        }
    }

    pub async fn get_tournament(&self, id: String) -> Result<Tournament, crate::Error> {
        let client = self.client.read().await;

        let response = client
            .get(format!("{}tournament/get/{}", MAIN_URL, Uuid::from_str(&id).unwrap()))
            .send()
            .await;

        match response {
            Ok(response) => {
                tracing::info!("Got tournament response: {:?}", &response);
                let tournament: Result<Tournament, reqwest::Error> = response.json().await;
                match tournament {
                    Ok(tournament) => {
                        tracing::info!("Tournament json parsed successfully: {:?}", &tournament);
                        Ok(tournament)
                    },
                    Err(json_error) => {
                        tracing::error!("Failed to parse tournament json: {}", &json_error.to_string());
                        Err(crate::Error::from(json_error))
                    }
                }
            },
            Err(response_error) => {
                tracing::error!("Failed to send get tournament request: {}", &response_error.to_string());
                Err(crate::Error::from(response_error))
            }
        }
    } 

    pub async fn load_races(&self) -> Result<Vec<Race>, crate::Error> {
        let client = self.client.read().await;
        let races_response = client
            .get(format!("{}races", MAIN_URL))
            .send()
            .await;
    
        match races_response {
            Ok(success) => {
                let races_json_data = success.json().await;
                match races_json_data {
                    Ok(races) => {
                        Ok(races)
                    },
                    Err(json_error) => {
                        Err(crate::Error::from(json_error))
                    }
                }
            },
            Err(failure) => {
                Err(crate::Error::from(failure))
            }
        }
    }
    
    pub async fn load_heroes(&self, mod_type: ModType) -> Result<Vec<Hero>, crate::Error> {
        let client = self.client.read().await;
        let heroes_response = client
            .get(format!("{}heroes/{}", MAIN_URL, mod_type as i16))
            .send()
            .await;
    
        match heroes_response {
            Ok(success) => {
                tracing::info!("Got response for heroes: {:?}", &success);
                let heroes_json_data = success.json().await;
                match heroes_json_data {
                    Ok(heroes) => {
                        tracing::info!("Heroes json processed successfully");
                        Ok(heroes)
                    },
                    Err(json_error) => {
                        tracing::error!("Failed to process heroes json: {}", json_error.to_string());
                        Err(crate::Error::from(json_error))
                    }
                }
            },
            Err(failure) => {
                tracing::error!("Failed to get heroes response: {}", failure.to_string());
                Err(crate::Error::from(failure))
            }
        }
    }

    pub async fn send_match<'a>(&self, parsed_data: &'a mut ParsedData<'a>, tournament_id: Uuid, message_id: i64) -> Result<(), crate::Error> {
        let client = self.client.read().await;

        let match_registration_response = client
            .post(format!("{}match/register?tournament_id={}&first_player={}&second_player={}&message_id={}",
                MAIN_URL,
                tournament_id,
                parsed_data.first_player,
                parsed_data.second_player,
                message_id
            ))
            .send()
            .await?;

        let registered_id: Uuid = match_registration_response.json().await?;

        for game in &mut parsed_data.games {
            game.match_id = registered_id;
        }

        let games_registration_response = client
            .post(format!("{}match/games", MAIN_URL))
            .json(&parsed_data.games)
            .send()
            .await;

        match games_registration_response {
            Ok(success) => {
                tracing::info!("Got response for game uploading: {:?}", &success);
            },
            _=> {}
        }

        Ok(())
    }    
}