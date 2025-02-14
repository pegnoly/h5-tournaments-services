use reqwest::header::{HeaderMap, HeaderValue};
use serde::Serialize;
use uuid::Uuid;

use super::payloads::{ChallongeData, ChallongeParticipantAttributes, ChallongeParticipantPayload};

pub struct ChallongeService {
    client: ChallongeClient
}

pub (self) struct ChallongeClient {
    client: tokio::sync::RwLock<reqwest::Client>,
    url: String,
    api_key: String
}

impl ChallongeClient {
    pub fn new(url: String, api_key: String) -> Self {
        ChallongeClient {
            client: tokio::sync::RwLock::new(reqwest::Client::new()),
            url: url,
            api_key: api_key
        }
    }

    pub async fn get(&self, params: &str) -> Result<reqwest::Response, reqwest::Error> {
        let client_locked = self.client.read().await;
        let response = client_locked.get(format!("{}{}", self.url, &params))
            //.headers(self.default_headers())
            .header("Accept", "application/json")
            .header("Content-Type", "application/vnd.api+json")
            .header("Authorization-Type", "v1")
            .header("Authorization", &self.api_key)
            .send()
            .await;
        response
    }

    pub async fn post<T: Serialize>(&self, params: &str, payload: ChallongeData<T>) -> Result<reqwest::Response, reqwest::Error> {
        let client_locked = self.client.read().await;
        let response = client_locked.post(format!("{}{}", self.url, &params))
            .header("Accept", "application/json")
            .header("Content-Type", "application/vnd.api+json")
            .header("Authorization-Type", "v1")
            .header("Authorization", &self.api_key)
            .json(&payload)
            .send()
            .await;

        response
    }

    fn default_headers(&self) -> HeaderMap<HeaderValue> {
        let mut headers = HeaderMap::new();
        headers.insert("Accept", HeaderValue::from_static("application/json")).unwrap();
        headers.insert("Content-Type", HeaderValue::from_static("application/vnd.api+json")).unwrap();
        headers.insert("Authorization-Type", HeaderValue::from_static("v1")).unwrap();
        headers.insert("Authorization", HeaderValue::from_str(&self.api_key).unwrap()).unwrap();
        headers
    }
}

impl ChallongeService {
    pub fn new(secret_store: &shuttle_runtime::SecretStore) -> Self {
        ChallongeService {
            client: ChallongeClient::new(secret_store.get("CHALLONGE_URL").unwrap(), secret_store.get("CHALLONGE_API_KEY").unwrap())
        }
    }

    pub async fn get_tournaments(&self) -> Result<(), crate::Error> {
        let response = self.client.get("tournaments.json?page=1&per_page=25").await;
        match response {
            Ok(success) => {
                tracing::info!("User's tournaments data: {:?}", &success.text().await);
            },
            Err(failure) => {
                tracing::error!("Failed to fetch all user's tournaments: {}", failure.to_string());
            }
        }
        Ok(())
    }

    pub async fn add_participant(&self, tournament_id: String, participant_id: String, participant_name: String) -> Result<(), crate::Error> {
        let payload = ChallongeParticipantPayload {
            _type: super::payloads::ChallongePayloadType::Participants,
            attributes: Some(ChallongeParticipantAttributes {
                name: participant_name,
                seed: None,
                misc: Some(participant_id),
                email: None,
                username: None
            }),
        };

        let response = self.client.post(
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