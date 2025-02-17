use serde::Serialize;

use super::{payloads::{ChallongeData, ChallongeParticipantAttributes, ChallongeParticipantPayload}, types::{ChallongeTournamentSimpleData, ChallongeTournamentsSimple}};

pub struct ChallongeService {
    client: ChallongeClient
}

pub (self) struct ChallongeClient {
    client: tokio::sync::RwLock<reqwest::Client>,
    url: String
}

impl ChallongeClient {
    pub fn new(url: String) -> Self {
        ChallongeClient {
            client: tokio::sync::RwLock::new(reqwest::Client::new()),
            url: url
        }
    }

    pub async fn get(&self, api_key: String, params: &str) -> Result<reqwest::Response, reqwest::Error> {
        let client_locked = self.client.read().await;
        let response = client_locked.get(format!("{}{}", self.url, &params))
            .header("Accept", "application/json")
            .header("Content-Type", "application/vnd.api+json")
            .header("Authorization-Type", "v1")
            .header("Authorization", api_key)
            .send()
            .await;
        response
    }

    pub async fn post<T: Serialize>(&self, api_key: String, params: &str, payload: ChallongeData<T>) -> Result<reqwest::Response, reqwest::Error> {
        let client_locked = self.client.read().await;
        let response = client_locked.post(format!("{}{}", self.url, &params))
            .header("Accept", "application/json")
            .header("Content-Type", "application/vnd.api+json")
            .header("Authorization-Type", "v1")
            .header("Authorization", api_key)
            .json(&payload)
            .send()
            .await;

        response
    }
}

impl ChallongeService {
    pub fn new(secret_store: &shuttle_runtime::SecretStore) -> Self {
        ChallongeService {
            client: ChallongeClient::new(secret_store.get("CHALLONGE_URL").unwrap())
        }
    }

    pub async fn get_tournaments(&self, api_key: String) -> Result<Vec<ChallongeTournamentSimpleData>, crate::Error> {
        let response = self.client.get(api_key, "tournaments.json?page=1&per_page=25").await;
        match response {
            Ok(success) => {
                match success.json::<ChallongeTournamentsSimple>().await {
                    Ok(tournaments) => {
                        Ok(tournaments.data)
                    },
                    Err(error) => {
                        tracing::error!("Error deserializing tournaments data: {}", error.to_string());
                        Err(crate::Error::from("Error deserializing tournaments data"))
                    }
                }
            },
            Err(failure) => {
                tracing::error!("Failed to fetch all user's tournaments: {}", failure.to_string());
                Err(crate::Error::from("Failed to fetch all user's tournaments"))
            }
        }
    }

    pub async fn add_participant(&self, api_key: String, tournament_id: String, participant_id: String, participant_name: String) -> Result<(), crate::Error> {
        let payload = ChallongeParticipantPayload {
            _type: super::payloads::ChallongePayloadType::Participants,
            attributes: Some(ChallongeParticipantAttributes {
                name: participant_name,
                seed: Some(1),
                misc: Some(participant_id),
                email: Some(String::new()),
                username: Some(String::new())
            }),
        };

        let response = self.client.post(
            api_key,
            &format!("tournaments/{}/participants.json", tournament_id), 
            ChallongeData { data: payload }
        ).await;

        match response {
            Ok(success) => {
                tracing::info!("Participant added successfully: {:?}", &success.text().await);
            },
            Err(failure) => {
                tracing::error!("Failed to send add participant request: {}", failure.to_string());
            }
        }

        Ok(())
    }
}