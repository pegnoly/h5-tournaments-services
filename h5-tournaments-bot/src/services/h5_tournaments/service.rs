use std::str::FromStr;

use graphql_client::{GraphQLQuery, Response};
use h5_tournaments_api::prelude::{Hero, ModType, Race, Tournament};
use uuid::Uuid;

use crate::{
    graphql::queries::{
        self, create_game_mutation::{self, CreateGameMutationCreateGame}, create_organizer, create_participant, create_user_mutation::ResponseData, delete_participant, get_game_query::{self, GetGameQueryGame}, get_games_query::{self, GetGamesQueryGames}, get_hero_query::{self, GetHeroQueryHero}, get_heroes_query::{self, GetHeroesQueryHeroes}, get_operator_data_query::{self, GetOperatorDataQueryOperator}, get_organizer::{self, GetOrganizerOrganizer}, get_participant::{self, GetParticipantParticipant}, get_participants::{self, GetParticipantsParticipants}, get_tournament_query, get_user_query::{self, GetUserQueryUser}, update_game_mutation, update_match_mutation, update_user, CreateGameMutation, CreateMatchMutation, CreateOrganizer, CreateParticipant, CreateTournamentMutation, CreateUserMutation, DeleteParticipant, GetGameQuery, GetGamesQuery, GetHeroQuery, GetHeroesQuery, GetMatchQuery, GetOperatorDataQuery, GetOperatorSectionQuery, GetOrganizer, GetParticipant, GetParticipants, GetTournamentQuery, GetUserQuery, GetUsersQuery, GetUsersResult, UpdateGameMutation, UpdateMatchMutation, UpdateUser
    }, 
    parser::service::ParsedData, 
    types::payloads::{GetMatch, GetTournament, GetUser, UpdateGame, UpdateMatch}
};

use super::payloads::{CreateOrganizerPayload, GetOrganizerPayload};

pub struct RaceNew {
    pub id: i64,
    pub name: String
}

pub struct H5TournamentsService {
    client: tokio::sync::RwLock<reqwest::Client>,
    url: String,
    pub races: Vec<RaceNew>
}

impl H5TournamentsService {
    pub fn new(secrets: &shuttle_runtime::SecretStore) -> Self {
        H5TournamentsService {
            client: tokio::sync::RwLock::new(reqwest::Client::new()),
            url: secrets.get("H5_TOURNAMENTS_URL").unwrap(),
            races: vec![
                RaceNew {
                    name: "Орден порядка".to_string(),
                    id: 1,
                },
                RaceNew {
                    name: "Инферно".to_string(),
                    id: 2
                },
                RaceNew {
                    name: "Некрополис".to_string(),
                    id: 3
                },
                RaceNew {
                    name: "Лесной союз".to_string(),
                    id: 4
                },
                RaceNew {
                    name: "Лига теней".to_string(),
                    id: 5
                },
                RaceNew {
                    name: "Академия волшебства".to_string(),
                    id: 6
                },
                RaceNew {
                    name: "Северные кланы".to_string(),
                    id: 7
                },
                RaceNew {
                    name: "Великая орда".to_string(),
                    id: 8
                },
            ]
        }
    }

    pub async fn init_tournament(&self, tournament_data: &serde_json::Value) -> Result<String, crate::Error> {
        let client = self.client.read().await;

        let response = client
            .post(format!("{}tournament/create", &self.url))
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
            .get(format!("{}tournament/get/{}", &self.url, Uuid::from_str(&id).unwrap()))
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
            .get(format!("{}races", &self.url))
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
            .get(format!("{}heroes/{}", &self.url, mod_type as i16))
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
                &self.url,
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
            .post(format!("{}match/games", &self.url))
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

    pub async fn create_user(&self, nickname: String, id: String, confirm: bool) -> Result<Uuid, crate::Error> {
        let variables = crate::graphql::queries::create_user_mutation::Variables {
            name: nickname,
            discord: id,
            confirm_register: confirm
        };
        
        let client = self.client.read().await;
        let query = CreateUserMutation::build_query(variables);
        let response = client.post(&self.url).json(&query).send().await;
        match response {
            Ok(response) => {
                let result = response.json::<Response<ResponseData>>().await;
                match result {
                    Ok(result) => {
                        if let Some(data) = result.data {
                            Ok(data.create_user)
                        }
                        else {
                            if let Some(errors) = result.errors {
                                Err(errors.iter().map(|e| e.to_string()).collect::<Vec<String>>().concat().into())
                            }
                            else {
                                Err(crate::Error::from("Unknown interaction: no data and no errors returned".to_string()))
                            }
                        }
                    },
                    Err(error) => {
                        Err(crate::Error::from(error))
                    }
                }
            }
            Err(response_error) => {
                Err(crate::Error::from(response_error))
            }
        }
    }

    pub async fn get_operator_section(&self, id: Uuid) -> Result<i64, crate::Error> {
        let variables = queries::get_operator_section_query::Variables {
            id: id
        };

        let client = self.client.read().await;
        let query = GetOperatorSectionQuery::build_query(variables);
        let response = client.post(&self.url).json(&query).send().await;
        match response {
            Ok(response) => {
                let result = response.json::<Response<queries::get_operator_section_query::ResponseData>>().await;
                match result {
                    Ok(result) => {
                        if let Some(data) = result.data {
                            Ok(data.operator.unwrap().section)
                        }
                        else {
                            Err(crate::Error::from("Unknown error: got successful response but incorrect data".to_string()))
                        }
                    },
                    Err(json_error) => {
                        Err(crate::Error::from(json_error))
                    }
                }
            },
            Err(response_error) => {
                Err(crate::Error::from(response_error))
            }
        }
    }

    pub async fn get_operator_data(&self, id: Uuid) -> Result<GetOperatorDataQueryOperator, crate::Error> {
        let client = self.client.read().await;
        let query = GetOperatorDataQuery::build_query(get_operator_data_query::Variables { id: id});
        let response = client.post(&self.url).json(&query).send().await;
        match response {
            Ok(response) => {
                let result = response.json::<Response<queries::get_operator_data_query::ResponseData>>().await;
                tracing::info!("Operator fetch result: {:?}", &result);
                match result {
                    Ok(result) => {
                        if let Some(data) = result.data {
                            Ok(data.operator.unwrap())
                        }
                        else {
                            Err(crate::Error::from("Unknown error: got successful response but incorrect data".to_string()))
                        }
                    },
                    Err(json_error) => {
                        Err(crate::Error::from(json_error))
                    }
                }
            },
            Err(response_error) => {
                Err(crate::Error::from(response_error))
            }
        }
    }


    pub async fn create_tournament(
        &self, name: String, 
        operator_id: Uuid, 
        channel_id: String,
        register_channel: String,
        use_bargains: bool,
        use_foreign_heroes: bool,
        role: String
    ) -> Result<String, crate::Error>{
        let variables = crate::graphql::queries::create_tournament_mutation::Variables {
            name: name.clone(),
            operator_id: operator_id,
            channel_id: channel_id,
            register_channel: register_channel,
            use_bargains: use_bargains,
            use_foreign_heroes: use_foreign_heroes,
            role: role
        };

        let client = self.client.read().await;
        let query = CreateTournamentMutation::build_query(variables);
        let response = client.post(&self.url).json(&query).send().await;
        match response {
            Ok(response) => {
                let result = response.json::<Response<crate::graphql::queries::create_tournament_mutation::ResponseData>>().await;
                match result {
                    Ok(result) => {
                        if let Some(data) = result.data {
                            Ok(data.create_tournament)
                        }
                        else {
                            Err(crate::Error::from("Unknown error: got successful response but incorrect data".to_string()))
                        }
                    },
                    Err(json_error) => {
                        Err(crate::Error::from(json_error))
                    }
                }
            },
            Err(response_error) => {
                Err(crate::Error::from(response_error))
            }
        }
    }

    pub async fn get_tournament_data(&self, payload: GetTournament) -> Result<Option<queries::GetTournamentResult>, crate::Error> {
        let client = self.client.read().await;
        let query = GetTournamentQuery::build_query(get_tournament_query::Variables::from(payload));
        let response = client.post(&self.url).json(&query).send().await;
        match response {
            Ok(response) => {
                let result = response.json::<Response<queries::get_tournament_query::ResponseData>>().await;
                match result {
                    Ok(result) => {
                        tracing::info!("Got tournament response: {:?}", &result);
                        if let Some(data) = result.data {
                            Ok(data.tournament)
                        }
                        else {
                            Err(crate::Error::from("Unknown error: got successful response but incorrect data".to_string()))
                        }
                    },
                    Err(json_error) => {
                        Err(crate::Error::from(json_error))
                    }
                }
            },
            Err(response_error) => {
                Err(crate::Error::from(response_error))
            }
        }
    }

    pub async fn get_user(&self, payload: GetUser) -> Result<Option<GetUserQueryUser>, crate::Error> {
        let client = self.client.read().await;
        let query = GetUserQuery::build_query(get_user_query::Variables::from(payload));
        let response = client.post(&self.url).json(&query).send().await;
        match response {
            Ok(response) => {
                let result = response.json::<Response<queries::get_user_query::ResponseData>>().await;
                match result {
                    Ok(result) => {
                        tracing::info!("Got user response: {:?}", &result);
                        if let Some(data) = result.data {
                            Ok(data.user)
                        }
                        else {
                            Err(crate::Error::from("Unknown error: got successful response but incorrect data".to_string()))
                        }
                    },
                    Err(json_error) => {
                        Err(crate::Error::from(json_error))
                    }
                }
            },
            Err(response_error) => {
                Err(crate::Error::from(response_error))
            }
        }
    }

    pub async fn create_match(
        &self,
        tournament_id: Uuid,
        first_player: Uuid,
        interaction_id: u64
    ) -> Result<String, crate::Error> {
        let variables = queries::create_match_mutation::Variables {
            tournament_id: tournament_id,
            interaction: interaction_id.to_string(),
            first_player: first_player
        };

        let client = self.client.read().await;
        let query = CreateMatchMutation::build_query(variables);
        let response = client.post(&self.url).json(&query).send().await;
        match response {
            Ok(response) => {
                let result = response.json::<Response<queries::create_match_mutation::ResponseData>>().await;
                match result {
                    Ok(result) => {
                        if let Some(data) = result.data {
                            Ok(data.create_match)
                        }
                        else {
                            Err(crate::Error::from("Unknown error: got successful response but incorrect data".to_string()))
                        }
                    },
                    Err(json_error) => {
                        Err(crate::Error::from(json_error))
                    }
                }
            },
            Err(response_error) => {
                Err(crate::Error::from(response_error))
            }
        }
    }

    pub async fn get_match(&self, payload: GetMatch) -> Result<Option<queries::GetMatchResult>, crate::Error> {
        let client = self.client.read().await;
        let query = GetMatchQuery::build_query(queries::get_match_query::Variables::from(payload));
        let response = client.post(&self.url).json(&query).send().await;
        match response {
            Ok(response) => {
                let result = response.json::<Response<queries::get_match_query::ResponseData>>().await;
                match result {
                    Ok(result) => {
                        if let Some(data) = result.data {
                            Ok(data.tournament_match)
                        }
                        else {
                            Err(crate::Error::from("Unknown error: got successful response but incorrect data".to_string()))
                        }
                    },
                    Err(json_error) => {
                        Err(crate::Error::from(json_error))
                    }
                }
            },
            Err(response_error) => {
                Err(crate::Error::from(response_error))
            }
        }
    }

    pub async fn update_match(&self, payload: UpdateMatch) -> Result<String, crate::Error> {
        let client = self.client.read().await;
        let query = UpdateMatchMutation::build_query(update_match_mutation::Variables::from(payload));
        let response = client.post(&self.url).json(&query).send().await;
        match response {
            Ok(response) => {
                let result = response.json::<Response<queries::update_match_mutation::ResponseData>>().await;
                match result {
                    Ok(result) => {
                        if let Some(data) = result.data {
                            Ok(data.update_match)
                        }
                        else {
                            Err(crate::Error::from("Unknown error: got successful response but incorrect data".to_string()))
                        }
                    },
                    Err(json_error) => {
                        Err(crate::Error::from(json_error))
                    }
                }
            },
            Err(response_error) => {
                Err(crate::Error::from(response_error))
            }
        }
    }

    pub async fn get_users(&self) -> Result<Option<Vec<GetUsersResult>>, crate::Error> {
        let variables = queries::get_users_query::Variables;
        let client = self.client.read().await;
        let query = GetUsersQuery::build_query(variables);
        let response = client.post(&self.url).json(&query).send().await;
        match response {
            Ok(response) => {
                let result = response.json::<Response<queries::get_users_query::ResponseData>>().await;
                match result {
                    Ok(result) => {
                        if let Some(data) = result.data {
                            Ok(data.users)
                        }
                        else {
                            Err(crate::Error::from("Unknown error: got successful response but incorrect data".to_string()))
                        }
                    },
                    Err(json_error) => {
                        Err(crate::Error::from(json_error))
                    }
                }
            },
            Err(response_error) => {
                Err(crate::Error::from(response_error))
            }
        }
    }

    pub async fn get_game(
        &self,
        match_id: Uuid,
        number: i64
    ) -> Result<Option<GetGameQueryGame>, crate::Error> {
        let variables = get_game_query::Variables {
            match_id: match_id,
            number: number as i64
        };
        let client = self.client.read().await;
        let query = GetGameQuery::build_query(variables);
        let response = client.post(&self.url).json(&query).send().await;
        match response {
            Ok(response) => {
                let result = response.json::<Response<queries::get_game_query::ResponseData>>().await;
                match result {
                    Ok(result) => {
                        if let Some(data) = result.data {
                            Ok(data.game)
                        }
                        else {
                            Err(crate::Error::from("Unknown error: got successful response but incorrect data".to_string()))
                        }
                    },
                    Err(json_error) => {
                        Err(crate::Error::from(json_error))
                    }
                }
            },
            Err(response_error) => {
                Err(crate::Error::from(response_error))
            }
        }
    }
    
    pub async fn create_game(
        &self,
        match_id: Uuid,
        number: i64
    ) -> Result<CreateGameMutationCreateGame, crate::Error> {
        let variables = create_game_mutation::Variables {
            match_id: match_id,
            number: number as i64
        };

        let client = self.client.read().await;
        let query = CreateGameMutation::build_query(variables);
        let response = client.post(&self.url).json(&query).send().await;
        match response {
            Ok(response) => {
                let result = response.json::<Response<queries::create_game_mutation::ResponseData>>().await;
                match result {
                    Ok(result) => {
                        tracing::info!("Game creation result: {:?}", &result);
                        if let Some(data) = result.data {
                            Ok(data.create_game)
                        }
                        else {
                            Err(crate::Error::from("Unknown error: got successful response but incorrect data".to_string()))
                        }
                    },
                    Err(json_error) => {
                        Err(crate::Error::from(json_error))
                    }
                }
            },
            Err(response_error) => {
                Err(crate::Error::from(response_error))
            }
        }
    }

    pub async fn update_game(&self, payload: UpdateGame) -> Result<String, crate::Error> {
        let client = self.client.read().await;
        let query = UpdateGameMutation::build_query(update_game_mutation::Variables::from(payload));
        let response = client.post(&self.url).json(&query).send().await;
        match response {
            Ok(response) => {
                let result = response.json::<Response<queries::update_game_mutation::ResponseData>>().await;
                match result {
                    Ok(result) => {
                        tracing::info!("Game update result: {:?}", &result);
                        if let Some(data) = result.data {
                            Ok(data.update_game)
                        }
                        else {
                            Err(crate::Error::from("Unknown error: got successful response but incorrect data".to_string()))
                        }
                    },
                    Err(json_error) => {
                        Err(crate::Error::from(json_error))
                    }
                }
            },
            Err(response_error) => {
                Err(crate::Error::from(response_error))
            }
        }
    }

    pub async fn get_heroes(
        &self,
        race: i64
    ) -> Result<Vec<GetHeroesQueryHeroes>, crate::Error> {
        let variables = get_heroes_query::Variables {
            race: race
        };

        let client = self.client.read().await;
        let query = GetHeroesQuery::build_query(variables);
        let response = client.post(&self.url).json(&query).send().await;
        match response {
            Ok(response) => {
                let result = response.json::<Response<queries::get_heroes_query::ResponseData>>().await;
                match result {
                    Ok(result) => {
                        tracing::info!("Heroes fetch result: {:?}", &result);
                        if let Some(data) = result.data {
                            Ok(data.heroes)
                        }
                        else {
                            Err(crate::Error::from("Unknown error: got successful response but incorrect data".to_string()))
                        }
                    },
                    Err(json_error) => {
                        Err(crate::Error::from(json_error))
                    }
                }
            },
            Err(response_error) => {
                Err(crate::Error::from(response_error))
            }
        }
    }

    pub async fn get_hero(
        &self,
        id: i64
    ) -> Result<Option<GetHeroQueryHero>, crate::Error> {
        let variables = get_hero_query::Variables {
            id: id
        };

        let client = self.client.read().await;
        let query = GetHeroQuery::build_query(variables);
        let response = client.post(&self.url).json(&query).send().await;
        match response {
            Ok(response) => {
                let result = response.json::<Response<queries::get_hero_query::ResponseData>>().await;
                match result {
                    Ok(result) => {
                        tracing::info!("Hero fetch result: {:?}", &result);
                        if let Some(data) = result.data {
                            Ok(data.hero)
                        }
                        else {
                            Err(crate::Error::from("Unknown error: got successful response but incorrect data".to_string()))
                        }
                    },
                    Err(json_error) => {
                        Err(crate::Error::from(json_error))
                    }
                }
            },
            Err(response_error) => {
                Err(crate::Error::from(response_error))
            }
        }
    }

    pub async fn get_games(
        &self,
        match_id: Uuid
    ) -> Result<Vec<GetGamesQueryGames>, crate::Error> {
        let variables = get_games_query::Variables {
            match_id: match_id
        };

        let client = self.client.read().await;
        let query = GetGamesQuery::build_query(variables);
        let response = client.post(&self.url).json(&query).send().await;
        match response {
            Ok(response) => {
                let result = response.json::<Response<queries::get_games_query::ResponseData>>().await;
                match result {
                    Ok(result) => {
                        tracing::info!("Hero fetch result: {:?}", &result);
                        if let Some(data) = result.data {
                            Ok(data.games)
                        }
                        else {
                            Err(crate::Error::from("Unknown error: got successful response but incorrect data".to_string()))
                        }
                    },
                    Err(json_error) => {
                        Err(crate::Error::from(json_error))
                    }
                }
            },
            Err(response_error) => {
                Err(crate::Error::from(response_error))
            }
        }
    }

    pub async fn create_participant(
        &self,
        tournament_id: Uuid,
        user_id: Uuid,
        group: i64
    ) -> Result<i64, crate::Error> {
        let variables = create_participant::Variables {
            tournament_id: tournament_id,
            user_id: user_id,
            group: group
        };

        let client = self.client.read().await;
        let query = CreateParticipant::build_query(variables);
        let response = client.post(&self.url).json(&query).send().await;
        match response {
            Ok(response) => {
                let result = response.json::<Response<queries::create_participant::ResponseData>>().await;
                match result {
                    Ok(result) => {
                        tracing::info!("Create participant result: {:?}", &result);
                        if let Some(data) = result.data {
                            Ok(data.create_participant)
                        }
                        else {
                            Err(crate::Error::from("Unknown error: got successful response but incorrect data".to_string()))
                        }
                    },
                    Err(json_error) => {
                        Err(crate::Error::from(json_error))
                    }
                }
            },
            Err(response_error) => {
                Err(crate::Error::from(response_error))
            }
        } 
    }

    pub async fn get_participant(
        &self,
        tournament_id: Uuid,
        user_id: Uuid
    ) -> Result<Option<GetParticipantParticipant>, crate::Error> {
        let variables = get_participant::Variables {
            tournament_id: tournament_id,
            user_id: user_id
        };

        let client = self.client.read().await;
        let query = GetParticipant::build_query(variables);
        let response = client.post(&self.url).json(&query).send().await;
        match response {
            Ok(response) => {
                let result = response.json::<Response<queries::get_participant::ResponseData>>().await;
                match result {
                    Ok(result) => {
                        tracing::info!("Get participant result: {:?}", &result);
                        if let Some(data) = result.data {
                            Ok(data.participant)
                        }
                        else {
                            Err(crate::Error::from("Unknown error: got successful response but incorrect data".to_string()))
                        }
                    },
                    Err(json_error) => {
                        Err(crate::Error::from(json_error))
                    }
                }
            },
            Err(response_error) => {
                Err(crate::Error::from(response_error))
            }
        } 
    }

    pub async fn get_participants(
        &self,
        tournament_id: Uuid,
        group: i64
    ) -> Result<Vec<GetParticipantsParticipants>, crate::Error> {
        let variables = get_participants::Variables {
            tournament_id: tournament_id,
            group: group
        };

        let client = self.client.read().await;
        let query = GetParticipants::build_query(variables);
        let response = client.post(&self.url).json(&query).send().await;
        match response {
            Ok(response) => {
                let result = response.json::<Response<queries::get_participants::ResponseData>>().await;
                match result {
                    Ok(result) => {
                        tracing::info!("Get participants result: {:?}", &result);
                        if let Some(data) = result.data {
                            Ok(data.participants)
                        }
                        else {
                            Err(crate::Error::from("Unknown error: got successful response but incorrect data".to_string()))
                        }
                    },
                    Err(json_error) => {
                        Err(crate::Error::from(json_error))
                    }
                }
            },
            Err(response_error) => {
                Err(crate::Error::from(response_error))
            }
        } 
    }

    pub async fn delete_participant(
        &self,
        tournament_id: Uuid,
        user_id: Uuid
    ) -> Result<String, crate::Error> {
        let variables = delete_participant::Variables {
            tournament_id: tournament_id,
            user_id: user_id
        };

        let client = self.client.read().await;
        let query = DeleteParticipant::build_query(variables);
        let response = client.post(&self.url).json(&query).send().await;
        match response {
            Ok(response) => {
                let result = response.json::<Response<queries::delete_participant::ResponseData>>().await;
                match result {
                    Ok(result) => {
                        tracing::info!("Delete participant result: {:?}", &result);
                        if let Some(data) = result.data {
                            Ok(data.delete_participant)
                        }
                        else {
                            Err(crate::Error::from("Unknown error: got successful response but incorrect data".to_string()))
                        }
                    },
                    Err(json_error) => {
                        Err(crate::Error::from(json_error))
                    }
                }
            },
            Err(response_error) => {
                Err(crate::Error::from(response_error))
            }
        }  
    }

    pub async fn update_user(
        &self,
        id: Uuid,
        nickname: Option<String>,
        registered: Option<bool>
    ) -> Result<String, crate::Error> {
        let variables = update_user::Variables {
            id: id,
            nickname: nickname,
            registered: registered
        };

        let client = self.client.read().await;
        let query = UpdateUser::build_query(variables);
        let response = client.post(&self.url).json(&query).send().await;
        match response {
            Ok(response) => {
                let result = response.json::<Response<queries::update_user::ResponseData>>().await;
                match result {
                    Ok(result) => {
                        tracing::info!("Update user result: {:?}", &result);
                        if let Some(data) = result.data {
                            Ok(data.update_user)
                        }
                        else {
                            Err(crate::Error::from("Unknown error: got successful response but incorrect data".to_string()))
                        }
                    },
                    Err(json_error) => {
                        Err(crate::Error::from(json_error))
                    }
                }
            },
            Err(response_error) => {
                Err(crate::Error::from(response_error))
            }
        }  
    }

    pub async fn create_organizer(
        &self,
        payload: CreateOrganizerPayload
    ) -> Result<Uuid, crate::Error> {
        let client = self.client.read().await;
        let query = CreateOrganizer::build_query(create_organizer::Variables::from(payload));
        let response = client.post(&self.url).json(&query).send().await;
        match response {
            Ok(response) => {
                let result = response.json::<Response<queries::create_organizer::ResponseData>>().await;
                match result {
                    Ok(result) => {
                        tracing::info!("Create organizer result: {:?}", &result);
                        if let Some(data) = result.data {
                            Ok(data.create_organizer)
                        }
                        else {
                            Err(crate::Error::from("Unknown error: got successful response but incorrect data".to_string()))
                        }
                    },
                    Err(json_error) => {
                        Err(crate::Error::from(json_error))
                    }
                }
            },
            Err(response_error) => {
                Err(crate::Error::from(response_error))
            }
        }
    }

    pub async fn get_organizer(
        &self,
        payload: GetOrganizerPayload
    ) -> Result<Option<GetOrganizerOrganizer>, crate::Error> {
        let client = self.client.read().await;
        let query = GetOrganizer::build_query(get_organizer::Variables::from(payload));
        let response = client.post(&self.url).json(&query).send().await;
        match response {
            Ok(response) => {
                let result = response.json::<Response<queries::get_organizer::ResponseData>>().await;
                match result {
                    Ok(result) => {
                        tracing::info!("Get organizer result: {:?}", &result);
                        if let Some(data) = result.data {
                            Ok(data.organizer)
                        }
                        else {
                            Err(crate::Error::from("Unknown error: got successful response but incorrect data".to_string()))
                        }
                    },
                    Err(json_error) => {
                        Err(crate::Error::from(json_error))
                    }
                }
            },
            Err(response_error) => {
                Err(crate::Error::from(response_error))
            }
        }
    }
}