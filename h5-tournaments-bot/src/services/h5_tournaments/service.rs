use std::str::FromStr;

use graphql_client::{GraphQLQuery, Response};
use h5_tournaments_api::prelude::{Hero, ModType, Race, Tournament};
use uuid::Uuid;

use crate::{
    graphql::queries::{
        self, CreateGamesBulk, CreateMatchMutation, CreateOrganizer, CreateParticipant,
        CreateTournamentBuilder, CreateTournamentMutation, CreateUserMutation, DeleteParticipant,
        GamesCount, GetHeroQuery, GetHeroesQuery, GetMatchQuery, GetOperatorDataQuery,
        GetOperatorSectionQuery, GetOrganizer, GetParticipant, GetTournamentBuilder,
        GetTournamentQuery, GetTournamentUsers, GetTournaments, GetUserQuery, GetUsersQuery,
        GetUsersResult, UpdateMatch, UpdateParticipantsBulk, UpdateTournament,
        UpdateTournamentBuilder, UpdateUser, UpdateUsersBulk,
        create_games_bulk::{self, CreateGameModel},
        create_organizer, create_participant,
        create_tournament_builder::{self, CreateTournamentBuilderCreateTournamentBuilder},
        create_tournament_mutation,
        create_user_mutation::{self, CreateUserMutationCreateUser, ResponseData},
        delete_participant, games_count,
        get_hero_query::{self, GetHeroQueryHero},
        get_heroes_query::{self, GetHeroesQueryHeroes},
        get_match_query::GetMatchQueryGetMatch,
        get_operator_data_query::{self, GetOperatorDataQueryOperator},
        get_organizer::{self, GetOrganizerOrganizer},
        get_participant::{self, GetParticipantParticipant},
        get_tournament_builder::{self, GetTournamentBuilderTournamentBuilder},
        get_tournament_query,
        get_tournament_users::{self, GetTournamentUsersTournamentUsers},
        get_tournaments::{self, GetTournamentsTournaments},
        get_user_query::{self, GetUserQueryUser},
        update_match,
        update_participants_bulk::{self, UpdateParticipant},
        update_tournament,
        update_tournament_builder::{self, UpdateTournamentBuilderUpdateTournamentBuilder},
        update_user, update_users_bulk,
    },
    parser::service::ParsedData,
    types::payloads::{GetMatch, GetTournament, GetUser},
};

use super::payloads::{
    CreateOrganizerPayload, CreateParticipantPayload, CreateTournamentPayload, CreateUserPayload, DeleteParticipantPayload, GetOperatorPayload, GetOrganizerPayload, GetParticipantPayload, GetTournamentBuilderPayload, UpdateTournamentBuilderPayload, UpdateTournamentPayload
};

pub struct RaceNew {
    pub id: i64,
    pub name: String,
}

pub struct H5TournamentsService {
    client: tokio::sync::RwLock<reqwest::Client>,
    url: String,
    pub races: Vec<RaceNew>,
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
                    id: 2,
                },
                RaceNew {
                    name: "Некрополис".to_string(),
                    id: 3,
                },
                RaceNew {
                    name: "Лесной союз".to_string(),
                    id: 4,
                },
                RaceNew {
                    name: "Лига теней".to_string(),
                    id: 5,
                },
                RaceNew {
                    name: "Академия волшебства".to_string(),
                    id: 6,
                },
                RaceNew {
                    name: "Северные кланы".to_string(),
                    id: 7,
                },
                RaceNew {
                    name: "Великая орда".to_string(),
                    id: 8,
                },
            ],
        }
    }

    pub async fn init_tournament(
        &self,
        tournament_data: &serde_json::Value,
    ) -> Result<String, crate::Error> {
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
            }
            Err(failure) => {
                tracing::error!(
                    "Failed to send tournament creation request: {}",
                    failure.to_string()
                );
                Err(crate::Error::from(failure))
            }
        }
    }

    pub async fn get_tournament(&self, id: String) -> Result<Tournament, crate::Error> {
        let client = self.client.read().await;

        let response = client
            .get(format!(
                "{}tournament/get/{}",
                &self.url,
                Uuid::from_str(&id).unwrap()
            ))
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
                    }
                    Err(json_error) => {
                        tracing::error!(
                            "Failed to parse tournament json: {}",
                            &json_error.to_string()
                        );
                        Err(crate::Error::from(json_error))
                    }
                }
            }
            Err(response_error) => {
                tracing::error!(
                    "Failed to send get tournament request: {}",
                    &response_error.to_string()
                );
                Err(crate::Error::from(response_error))
            }
        }
    }

    pub async fn load_races(&self) -> Result<Vec<Race>, crate::Error> {
        let client = self.client.read().await;
        let races_response = client.get(format!("{}races", &self.url)).send().await;

        match races_response {
            Ok(success) => {
                let races_json_data = success.json().await;
                match races_json_data {
                    Ok(races) => Ok(races),
                    Err(json_error) => Err(crate::Error::from(json_error)),
                }
            }
            Err(failure) => Err(crate::Error::from(failure)),
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
                    }
                    Err(json_error) => {
                        tracing::error!(
                            "Failed to process heroes json: {}",
                            json_error.to_string()
                        );
                        Err(crate::Error::from(json_error))
                    }
                }
            }
            Err(failure) => {
                tracing::error!("Failed to get heroes response: {}", failure.to_string());
                Err(crate::Error::from(failure))
            }
        }
    }

    pub async fn send_match<'a>(
        &self,
        parsed_data: &'a mut ParsedData<'a>,
        tournament_id: Uuid,
        message_id: i64,
    ) -> Result<(), crate::Error> {
        let client = self.client.read().await;

        let match_registration_response = client
            .post(format!(
                "{}match/register?tournament_id={}&first_player={}&second_player={}&message_id={}",
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
            }
            _ => {}
        }

        Ok(())
    }

    pub async fn create_user(
        &self,
        payload: CreateUserPayload,
    ) -> Result<CreateUserMutationCreateUser, crate::Error> {
        let client = self.client.read().await;
        let query = CreateUserMutation::build_query(create_user_mutation::Variables::from(payload));
        let response = client.post(&self.url).json(&query).send().await;
        match response {
            Ok(response) => {
                let result = response
                    .json::<Response<create_user_mutation::ResponseData>>()
                    .await;
                match result {
                    Ok(result) => {
                        if let Some(data) = result.data {
                            Ok(data.create_user)
                        } else {
                            if let Some(errors) = result.errors {
                                Err(errors
                                    .iter()
                                    .map(|e| e.to_string())
                                    .collect::<Vec<String>>()
                                    .concat()
                                    .into())
                            } else {
                                Err(crate::Error::from(
                                    "Unknown interaction: no data and no errors returned"
                                        .to_string(),
                                ))
                            }
                        }
                    }
                    Err(error) => Err(crate::Error::from(error)),
                }
            }
            Err(response_error) => Err(crate::Error::from(response_error)),
        }
    }

    pub async fn get_operator_section(&self, id: Uuid) -> Result<i64, crate::Error> {
        let variables = queries::get_operator_section_query::Variables { id: id };

        let client = self.client.read().await;
        let query = GetOperatorSectionQuery::build_query(variables);
        let response = client.post(&self.url).json(&query).send().await;
        match response {
            Ok(response) => {
                let result = response
                    .json::<Response<queries::get_operator_section_query::ResponseData>>()
                    .await;
                match result {
                    Ok(result) => {
                        if let Some(data) = result.data {
                            Ok(data.operator.unwrap().section)
                        } else {
                            Err(crate::Error::from(
                                "Unknown error: got successful response but incorrect data"
                                    .to_string(),
                            ))
                        }
                    }
                    Err(json_error) => Err(crate::Error::from(json_error)),
                }
            }
            Err(response_error) => Err(crate::Error::from(response_error)),
        }
    }

    pub async fn get_operator_data(&self, payload: GetOperatorPayload) -> Result<GetOperatorDataQueryOperator, crate::Error> {
        let client = self.client.read().await;
        let query = GetOperatorDataQuery::build_query(get_operator_data_query::Variables::from(payload));
        let response = client.post(&self.url).json(&query).send().await;
        match response {
            Ok(response) => {
                let result = response.json::<Response<queries::get_operator_data_query::ResponseData>>().await;
                tracing::info!("Operator fetch result: {:?}", &result);
                match result {
                    Ok(result) => {
                        if let Some(data) = result.data {
                            Ok(data.operator.unwrap())
                        } else {
                            Err(crate::Error::from(
                                "Unknown error: got successful response but incorrect data".to_string(),
                            ))
                        }
                    }
                    Err(json_error) => Err(crate::Error::from(json_error)),
                }
            }
            Err(response_error) => Err(crate::Error::from(response_error)),
        }
    }

    pub async fn create_tournament(
        &self,
        payload: CreateTournamentPayload,
    ) -> Result<String, crate::Error> {
        let client = self.client.read().await;
        let query = CreateTournamentMutation::build_query(
            create_tournament_mutation::Variables::from(payload),
        );
        let response = client.post(&self.url).json(&query).send().await;
        match response {
            Ok(response) => {
                let result = response.json::<Response<crate::graphql::queries::create_tournament_mutation::ResponseData>>().await;
                match result {
                    Ok(result) => {
                        if let Some(data) = result.data {
                            Ok(data.create_tournament)
                        } else {
                            Err(crate::Error::from(
                                "Unknown error: got successful response but incorrect data"
                                    .to_string(),
                            ))
                        }
                    }
                    Err(json_error) => Err(crate::Error::from(json_error)),
                }
            }
            Err(response_error) => Err(crate::Error::from(response_error)),
        }
    }

    pub async fn get_tournament_data(
        &self,
        payload: GetTournament,
    ) -> Result<Option<queries::GetTournamentResult>, crate::Error> {
        let client = self.client.read().await;
        let query = GetTournamentQuery::build_query(get_tournament_query::Variables::from(payload));
        let response = client.post(&self.url).json(&query).send().await;
        match response {
            Ok(response) => {
                let result = response
                    .json::<Response<queries::get_tournament_query::ResponseData>>()
                    .await;
                match result {
                    Ok(result) => {
                        tracing::info!("Got tournament response: {:?}", &result);
                        if let Some(data) = result.data {
                            Ok(data.tournament)
                        } else {
                            Err(crate::Error::from(
                                "Unknown error: got successful response but incorrect data"
                                    .to_string(),
                            ))
                        }
                    }
                    Err(json_error) => Err(crate::Error::from(json_error)),
                }
            }
            Err(response_error) => Err(crate::Error::from(response_error)),
        }
    }

    pub async fn get_user(
        &self,
        payload: GetUser,
    ) -> Result<Option<GetUserQueryUser>, crate::Error> {
        let client = self.client.read().await;
        let query = GetUserQuery::build_query(get_user_query::Variables::from(payload));
        let response = client.post(&self.url).json(&query).send().await;
        match response {
            Ok(response) => {
                let result = response
                    .json::<Response<queries::get_user_query::ResponseData>>()
                    .await;
                match result {
                    Ok(result) => {
                        tracing::info!("Got user response: {:?}", &result);
                        if let Some(data) = result.data {
                            Ok(data.user)
                        } else {
                            Err(crate::Error::from(
                                "Unknown error: got successful response but incorrect data"
                                    .to_string(),
                            ))
                        }
                    }
                    Err(json_error) => Err(crate::Error::from(json_error)),
                }
            }
            Err(response_error) => Err(crate::Error::from(response_error)),
        }
    }

    pub async fn create_match(
        &self,
        tournament_id: Uuid,
        message: u64,
        first_player: Uuid,
        second_player: Uuid,
        challonge_id: String,
    ) -> Result<Uuid, crate::Error> {
        let variables = queries::create_match_mutation::Variables {
            tournament_id: tournament_id,
            message: message as i64,
            first_player: first_player,
            second_player: second_player,
            challonge_id: challonge_id,
        };

        let client = self.client.read().await;
        let query = CreateMatchMutation::build_query(variables);
        let response = client.post(&self.url).json(&query).send().await;
        match response {
            Ok(response) => {
                let result = response
                    .json::<Response<queries::create_match_mutation::ResponseData>>()
                    .await;
                match result {
                    Ok(result) => {
                        if let Some(data) = result.data {
                            Ok(data.create_match)
                        } else {
                            Err(crate::Error::from(
                                "Unknown error: got successful response but incorrect data"
                                    .to_string(),
                            ))
                        }
                    }
                    Err(json_error) => Err(crate::Error::from(json_error)),
                }
            }
            Err(response_error) => Err(crate::Error::from(response_error)),
        }
    }

    pub async fn get_match(&self, id: Uuid) -> Result<Option<GetMatchQueryGetMatch>, crate::Error> {
        let client = self.client.read().await;
        let query = GetMatchQuery::build_query(queries::get_match_query::Variables { id: id });
        let response = client.post(&self.url).json(&query).send().await;
        match response {
            Ok(response) => {
                let result = response
                    .json::<Response<queries::get_match_query::ResponseData>>()
                    .await;
                match result {
                    Ok(result) => {
                        if let Some(data) = result.data {
                            Ok(data.get_match)
                        } else {
                            Err(crate::Error::from(
                                "Unknown error: got successful response but incorrect data"
                                    .to_string(),
                            ))
                        }
                    }
                    Err(json_error) => Err(crate::Error::from(json_error)),
                }
            }
            Err(response_error) => Err(crate::Error::from(response_error)),
        }
    }

    // pub async fn update_match(&self, payload: UpdateMatch) -> Result<String, crate::Error> {
    //     let client = self.client.read().await;
    //     let query = UpdateMatchMutation::build_query(update_match_mutation::Variables::from(payload));
    //     let response = client.post(&self.url).json(&query).send().await;
    //     match response {
    //         Ok(response) => {
    //             let result = response.json::<Response<queries::update_match_mutation::ResponseData>>().await;
    //             match result {
    //                 Ok(result) => {
    //                     if let Some(data) = result.data {
    //                         Ok(data.update_match)
    //                     }
    //                     else {
    //                         Err(crate::Error::from("Unknown error: got successful response but incorrect data".to_string()))
    //                     }
    //                 },
    //                 Err(json_error) => {
    //                     Err(crate::Error::from(json_error))
    //                 }
    //             }
    //         },
    //         Err(response_error) => {
    //             Err(crate::Error::from(response_error))
    //         }
    //     }
    // }

    pub async fn get_users(&self) -> Result<Option<Vec<GetUsersResult>>, crate::Error> {
        let variables = queries::get_users_query::Variables;
        let client = self.client.read().await;
        let query = GetUsersQuery::build_query(variables);
        let response = client.post(&self.url).json(&query).send().await;
        match response {
            Ok(response) => {
                let result = response
                    .json::<Response<queries::get_users_query::ResponseData>>()
                    .await;
                match result {
                    Ok(result) => {
                        if let Some(data) = result.data {
                            Ok(data.users)
                        } else {
                            Err(crate::Error::from(
                                "Unknown error: got successful response but incorrect data"
                                    .to_string(),
                            ))
                        }
                    }
                    Err(json_error) => Err(crate::Error::from(json_error)),
                }
            }
            Err(response_error) => Err(crate::Error::from(response_error)),
        }
    }

    // pub async fn get_game(
    //     &self,
    //     match_id: Uuid,
    //     number: i64
    // ) -> Result<Option<GetGameQueryGame>, crate::Error> {
    //     let variables = get_game_query::Variables {
    //         match_id: match_id,
    //         number: number as i64
    //     };
    //     let client = self.client.read().await;
    //     let query = GetGameQuery::build_query(variables);
    //     let response = client.post(&self.url).json(&query).send().await;
    //     match response {
    //         Ok(response) => {
    //             let result = response.json::<Response<queries::get_game_query::ResponseData>>().await;
    //             match result {
    //                 Ok(result) => {
    //                     if let Some(data) = result.data {
    //                         Ok(data.game)
    //                     }
    //                     else {
    //                         Err(crate::Error::from("Unknown error: got successful response but incorrect data".to_string()))
    //                     }
    //                 },
    //                 Err(json_error) => {
    //                     Err(crate::Error::from(json_error))
    //                 }
    //             }
    //         },
    //         Err(response_error) => {
    //             Err(crate::Error::from(response_error))
    //         }
    //     }
    // }

    // pub async fn create_game(
    //     &self,
    //     match_id: Uuid,
    //     number: i64
    // ) -> Result<CreateGameMutationCreateGame, crate::Error> {
    //     let variables = create_game_mutation::Variables {
    //         match_id: match_id,
    //         number: number as i64
    //     };

    //     let client = self.client.read().await;
    //     let query = CreateGameMutation::build_query(variables);
    //     let response = client.post(&self.url).json(&query).send().await;
    //     match response {
    //         Ok(response) => {
    //             let result = response.json::<Response<queries::create_game_mutation::ResponseData>>().await;
    //             match result {
    //                 Ok(result) => {
    //                     tracing::info!("Game creation result: {:?}", &result);
    //                     if let Some(data) = result.data {
    //                         Ok(data.create_game)
    //                     }
    //                     else {
    //                         Err(crate::Error::from("Unknown error: got successful response but incorrect data".to_string()))
    //                     }
    //                 },
    //                 Err(json_error) => {
    //                     Err(crate::Error::from(json_error))
    //                 }
    //             }
    //         },
    //         Err(response_error) => {
    //             Err(crate::Error::from(response_error))
    //         }
    //     }
    // }

    // pub async fn update_game(&self, payload: UpdateGame) -> Result<String, crate::Error> {
    //     let client = self.client.read().await;
    //     let query = UpdateGameMutation::build_query(update_game_mutation::Variables::from(payload));
    //     let response = client.post(&self.url).json(&query).send().await;
    //     match response {
    //         Ok(response) => {
    //             let result = response.json::<Response<queries::update_game_mutation::ResponseData>>().await;
    //             match result {
    //                 Ok(result) => {
    //                     tracing::info!("Game update result: {:?}", &result);
    //                     if let Some(data) = result.data {
    //                         Ok(data.update_game)
    //                     }
    //                     else {
    //                         Err(crate::Error::from("Unknown error: got successful response but incorrect data".to_string()))
    //                     }
    //                 },
    //                 Err(json_error) => {
    //                     Err(crate::Error::from(json_error))
    //                 }
    //             }
    //         },
    //         Err(response_error) => {
    //             Err(crate::Error::from(response_error))
    //         }
    //     }
    // }

    pub async fn get_heroes(&self, race: i64) -> Result<Vec<GetHeroesQueryHeroes>, crate::Error> {
        let variables = get_heroes_query::Variables { race: race };

        let client = self.client.read().await;
        let query = GetHeroesQuery::build_query(variables);
        let response = client.post(&self.url).json(&query).send().await;
        match response {
            Ok(response) => {
                let result = response
                    .json::<Response<queries::get_heroes_query::ResponseData>>()
                    .await;
                match result {
                    Ok(result) => {
                        tracing::info!("Heroes fetch result: {:?}", &result);
                        if let Some(data) = result.data {
                            Ok(data.heroes)
                        } else {
                            Err(crate::Error::from(
                                "Unknown error: got successful response but incorrect data"
                                    .to_string(),
                            ))
                        }
                    }
                    Err(json_error) => Err(crate::Error::from(json_error)),
                }
            }
            Err(response_error) => Err(crate::Error::from(response_error)),
        }
    }

    pub async fn get_hero(&self, id: i64) -> Result<Option<GetHeroQueryHero>, crate::Error> {
        let variables = get_hero_query::Variables { id: id };

        let client = self.client.read().await;
        let query = GetHeroQuery::build_query(variables);
        let response = client.post(&self.url).json(&query).send().await;
        match response {
            Ok(response) => {
                let result = response
                    .json::<Response<queries::get_hero_query::ResponseData>>()
                    .await;
                match result {
                    Ok(result) => {
                        tracing::info!("Hero fetch result: {:?}", &result);
                        if let Some(data) = result.data {
                            Ok(data.hero)
                        } else {
                            Err(crate::Error::from(
                                "Unknown error: got successful response but incorrect data"
                                    .to_string(),
                            ))
                        }
                    }
                    Err(json_error) => Err(crate::Error::from(json_error)),
                }
            }
            Err(response_error) => Err(crate::Error::from(response_error)),
        }
    }

    // pub async fn get_games(
    //     &self,
    //     match_id: Uuid
    // ) -> Result<Vec<GetGamesQueryGames>, crate::Error> {
    //     let variables = get_games_query::Variables {
    //         match_id: match_id
    //     };

    //     let client = self.client.read().await;
    //     let query = GetGamesQuery::build_query(variables);
    //     let response = client.post(&self.url).json(&query).send().await;
    //     match response {
    //         Ok(response) => {
    //             let result = response.json::<Response<queries::get_games_query::ResponseData>>().await;
    //             match result {
    //                 Ok(result) => {
    //                     tracing::info!("Hero fetch result: {:?}", &result);
    //                     if let Some(data) = result.data {
    //                         Ok(data.games)
    //                     }
    //                     else {
    //                         Err(crate::Error::from("Unknown error: got successful response but incorrect data".to_string()))
    //                     }
    //                 },
    //                 Err(json_error) => {
    //                     Err(crate::Error::from(json_error))
    //                 }
    //             }
    //         },
    //         Err(response_error) => {
    //             Err(crate::Error::from(response_error))
    //         }
    //     }
    // }

    pub async fn create_participant(
        &self,
        payload: CreateParticipantPayload,
    ) -> Result<i64, crate::Error> {
        let client = self.client.read().await;
        let query = CreateParticipant::build_query(create_participant::Variables::from(payload));
        let response = client.post(&self.url).json(&query).send().await;
        match response {
            Ok(response) => {
                let result = response
                    .json::<Response<queries::create_participant::ResponseData>>()
                    .await;
                match result {
                    Ok(result) => {
                        tracing::info!("Create participant result: {:?}", &result);
                        if let Some(data) = result.data {
                            Ok(data.create_participant)
                        } else {
                            Err(crate::Error::from(
                                "Unknown error: got successful response but incorrect data"
                                    .to_string(),
                            ))
                        }
                    }
                    Err(json_error) => Err(crate::Error::from(json_error)),
                }
            }
            Err(response_error) => Err(crate::Error::from(response_error)),
        }
    }

    pub async fn get_participant(
        &self,
        payload: GetParticipantPayload,
    ) -> Result<Option<GetParticipantParticipant>, crate::Error> {
        let client = self.client.read().await;
        let query = GetParticipant::build_query(get_participant::Variables::from(payload));
        let response = client.post(&self.url).json(&query).send().await;
        match response {
            Ok(response) => {
                let result = response
                    .json::<Response<get_participant::ResponseData>>()
                    .await;
                match result {
                    Ok(result) => {
                        tracing::info!("Get participant result: {:?}", &result);
                        if let Some(data) = result.data {
                            Ok(data.participant)
                        } else {
                            Err(crate::Error::from(
                                "Unknown error: got successful response but incorrect data"
                                    .to_string(),
                            ))
                        }
                    }
                    Err(json_error) => Err(crate::Error::from(json_error)),
                }
            }
            Err(response_error) => Err(crate::Error::from(response_error)),
        }
    }

    // pub async fn get_participants(
    //     &self,
    //     tournament_id: Uuid,
    //     group: i64
    // ) -> Result<Vec<GetParticipantsParticipants>, crate::Error> {
    //     let variables = get_participants::Variables {
    //         tournament_id: tournament_id,
    //         group: group
    //     };

    //     let client = self.client.read().await;
    //     let query = GetParticipants::build_query(variables);
    //     let response = client.post(&self.url).json(&query).send().await;
    //     match response {
    //         Ok(response) => {
    //             let result = response.json::<Response<queries::get_participants::ResponseData>>().await;
    //             match result {
    //                 Ok(result) => {
    //                     tracing::info!("Get participants result: {:?}", &result);
    //                     if let Some(data) = result.data {
    //                         Ok(data.participants)
    //                     }
    //                     else {
    //                         Err(crate::Error::from("Unknown error: got successful response but incorrect data".to_string()))
    //                     }
    //                 },
    //                 Err(json_error) => {
    //                     Err(crate::Error::from(json_error))
    //                 }
    //             }
    //         },
    //         Err(response_error) => {
    //             Err(crate::Error::from(response_error))
    //         }
    //     }
    // }

    pub async fn delete_participant(
        &self,
        payload: DeleteParticipantPayload,
    ) -> Result<i64, crate::Error> {
        let client = self.client.read().await;
        let query = DeleteParticipant::build_query(delete_participant::Variables::from(payload));
        let response = client.post(&self.url).json(&query).send().await;
        match response {
            Ok(response) => {
                let result = response
                    .json::<Response<queries::delete_participant::ResponseData>>()
                    .await;
                match result {
                    Ok(result) => {
                        tracing::info!("Delete participant result: {:?}", &result);
                        if let Some(data) = result.data {
                            Ok(data.delete_participant)
                        } else {
                            Err(crate::Error::from(
                                "Unknown error: got successful response but incorrect data"
                                    .to_string(),
                            ))
                        }
                    }
                    Err(json_error) => Err(crate::Error::from(json_error)),
                }
            }
            Err(response_error) => Err(crate::Error::from(response_error)),
        }
    }

    pub async fn update_user(
        &self,
        id: Uuid,
        nickname: Option<String>,
        registered: Option<bool>,
    ) -> Result<String, crate::Error> {
        let variables = update_user::Variables {
            id: id,
            nickname: nickname,
            registered: registered,
        };

        let client = self.client.read().await;
        let query = UpdateUser::build_query(variables);
        let response = client.post(&self.url).json(&query).send().await;
        match response {
            Ok(response) => {
                let result = response
                    .json::<Response<queries::update_user::ResponseData>>()
                    .await;
                match result {
                    Ok(result) => {
                        tracing::info!("Update user result: {:?}", &result);
                        if let Some(data) = result.data {
                            Ok(data.update_user)
                        } else {
                            Err(crate::Error::from(
                                "Unknown error: got successful response but incorrect data"
                                    .to_string(),
                            ))
                        }
                    }
                    Err(json_error) => Err(crate::Error::from(json_error)),
                }
            }
            Err(response_error) => Err(crate::Error::from(response_error)),
        }
    }

    pub async fn create_organizer(
        &self,
        payload: CreateOrganizerPayload,
    ) -> Result<Uuid, crate::Error> {
        let client = self.client.read().await;
        let query = CreateOrganizer::build_query(create_organizer::Variables::from(payload));
        let response = client.post(&self.url).json(&query).send().await;
        match response {
            Ok(response) => {
                let result = response
                    .json::<Response<queries::create_organizer::ResponseData>>()
                    .await;
                match result {
                    Ok(result) => {
                        tracing::info!("Create organizer result: {:?}", &result);
                        if let Some(data) = result.data {
                            Ok(data.create_organizer)
                        } else {
                            Err(crate::Error::from(
                                "Unknown error: got successful response but incorrect data"
                                    .to_string(),
                            ))
                        }
                    }
                    Err(json_error) => Err(crate::Error::from(json_error)),
                }
            }
            Err(response_error) => Err(crate::Error::from(response_error)),
        }
    }

    pub async fn get_organizer(
        &self,
        payload: GetOrganizerPayload,
    ) -> Result<Option<GetOrganizerOrganizer>, crate::Error> {
        let client = self.client.read().await;
        let query = GetOrganizer::build_query(get_organizer::Variables::from(payload));
        let response = client.post(&self.url).json(&query).send().await;
        match response {
            Ok(response) => {
                let result = response
                    .json::<Response<queries::get_organizer::ResponseData>>()
                    .await;
                match result {
                    Ok(result) => {
                        tracing::info!("Get organizer result: {:?}", &result);
                        if let Some(data) = result.data {
                            Ok(data.organizer)
                        } else {
                            Err(crate::Error::from(
                                "Unknown error: got successful response but incorrect data"
                                    .to_string(),
                            ))
                        }
                    }
                    Err(json_error) => Err(crate::Error::from(json_error)),
                }
            }
            Err(response_error) => Err(crate::Error::from(response_error)),
        }
    }

    pub async fn get_tournament_builder(
        &self,
        payload: GetTournamentBuilderPayload,
    ) -> Result<Option<GetTournamentBuilderTournamentBuilder>, crate::Error> {
        let client = self.client.read().await;
        let query =
            GetTournamentBuilder::build_query(get_tournament_builder::Variables::from(payload));
        let response = client.post(&self.url).json(&query).send().await;
        match response {
            Ok(response) => {
                let result = response
                    .json::<Response<queries::get_tournament_builder::ResponseData>>()
                    .await;
                match result {
                    Ok(result) => {
                        tracing::info!("Get tournament builder result: {:?}", &result);
                        if let Some(data) = result.data {
                            Ok(data.tournament_builder)
                        } else {
                            Err(crate::Error::from(
                                "Unknown error: got successful response but incorrect data"
                                    .to_string(),
                            ))
                        }
                    }
                    Err(json_error) => Err(crate::Error::from(json_error)),
                }
            }
            Err(response_error) => Err(crate::Error::from(response_error)),
        }
    }

    pub async fn create_tournament_builder(
        &self,
        message: u64,
    ) -> Result<CreateTournamentBuilderCreateTournamentBuilder, crate::Error> {
        let client = self.client.read().await;
        let query = CreateTournamentBuilder::build_query(create_tournament_builder::Variables {
            message_id: message.to_string(),
        });
        let response = client.post(&self.url).json(&query).send().await;
        match response {
            Ok(response) => {
                let result = response
                    .json::<Response<queries::create_tournament_builder::ResponseData>>()
                    .await;
                match result {
                    Ok(result) => {
                        tracing::info!("Get tournament builder result: {:?}", &result);
                        if let Some(data) = result.data {
                            Ok(data.create_tournament_builder)
                        } else {
                            Err(crate::Error::from(
                                "Unknown error: got successful response but incorrect data"
                                    .to_string(),
                            ))
                        }
                    }
                    Err(json_error) => Err(crate::Error::from(json_error)),
                }
            }
            Err(response_error) => Err(crate::Error::from(response_error)),
        }
    }

    pub async fn update_tournament_builder(
        &self,
        payload: UpdateTournamentBuilderPayload,
    ) -> Result<UpdateTournamentBuilderUpdateTournamentBuilder, crate::Error> {
        let client = self.client.read().await;
        let query = UpdateTournamentBuilder::build_query(
            update_tournament_builder::Variables::from(payload),
        );
        let response = client.post(&self.url).json(&query).send().await;
        match response {
            Ok(response) => {
                let result = response
                    .json::<Response<queries::update_tournament_builder::ResponseData>>()
                    .await;
                match result {
                    Ok(result) => {
                        tracing::info!("Get tournament builder update result: {:?}", &result);
                        if let Some(data) = result.data {
                            Ok(data.update_tournament_builder)
                        } else {
                            Err(crate::Error::from(
                                "Unknown error: got successful response but incorrect data"
                                    .to_string(),
                            ))
                        }
                    }
                    Err(json_error) => Err(crate::Error::from(json_error)),
                }
            }
            Err(response_error) => Err(crate::Error::from(response_error)),
        }
    }

    pub async fn get_all_tournaments(
        &self,
        organizer: Uuid,
    ) -> Result<Vec<GetTournamentsTournaments>, crate::Error> {
        let client = self.client.read().await;
        let query = GetTournaments::build_query(get_tournaments::Variables {
            organizer_id: organizer,
        });
        let response = client.post(&self.url).json(&query).send().await;
        match response {
            Ok(response) => {
                let result = response
                    .json::<Response<queries::get_tournaments::ResponseData>>()
                    .await;
                match result {
                    Ok(result) => {
                        tracing::info!("Get tournaments result: {:?}", &result);
                        if let Some(data) = result.data {
                            Ok(data.tournaments)
                        } else {
                            Err(crate::Error::from(
                                "Unknown error: got successful response but incorrect data"
                                    .to_string(),
                            ))
                        }
                    }
                    Err(json_error) => Err(crate::Error::from(json_error)),
                }
            }
            Err(response_error) => Err(crate::Error::from(response_error)),
        }
    }

    pub async fn update_tournament(
        &self,
        payload: UpdateTournamentPayload,
    ) -> Result<String, crate::Error> {
        let client = self.client.read().await;
        let query = UpdateTournament::build_query(update_tournament::Variables::from(payload));
        let response = client.post(&self.url).json(&query).send().await;
        match response {
            Ok(response) => {
                let result = response
                    .json::<Response<queries::update_tournament::ResponseData>>()
                    .await;
                match result {
                    Ok(result) => {
                        tracing::info!("Update tournament result: {:?}", &result);
                        if let Some(data) = result.data {
                            Ok(data.update_tournament)
                        } else {
                            Err(crate::Error::from(
                                "Unknown error: got successful response but incorrect data"
                                    .to_string(),
                            ))
                        }
                    }
                    Err(json_error) => Err(crate::Error::from(json_error)),
                }
            }
            Err(response_error) => Err(crate::Error::from(response_error)),
        }
    }

    pub async fn get_tournament_users(
        &self,
        tournament_id: Uuid,
    ) -> Result<Vec<GetTournamentUsersTournamentUsers>, crate::Error> {
        let client = self.client.read().await;
        let query = GetTournamentUsers::build_query(get_tournament_users::Variables {
            tournament_id: tournament_id,
        });
        let response = client.post(&self.url).json(&query).send().await;
        match response {
            Ok(response) => {
                let result = response
                    .json::<Response<queries::get_tournament_users::ResponseData>>()
                    .await;
                match result {
                    Ok(result) => {
                        tracing::info!("Get tournament users result: {:?}", &result);
                        if let Some(data) = result.data {
                            Ok(data.tournament_users)
                        } else {
                            Err(crate::Error::from(
                                "Unknown error: got successful response but incorrect data"
                                    .to_string(),
                            ))
                        }
                    }
                    Err(json_error) => Err(crate::Error::from(json_error)),
                }
            }
            Err(response_error) => Err(crate::Error::from(response_error)),
        }
    }

    pub async fn update_participants_bulk(
        &self,
        participants: Vec<update_participants_bulk::UpdateParticipant>,
    ) -> Result<String, crate::Error> {
        let client = self.client.read().await;
        let query = UpdateParticipantsBulk::build_query(update_participants_bulk::Variables {
            participants: participants,
        });
        let response = client.post(&self.url).json(&query).send().await;
        match response {
            Ok(response) => {
                let result = response
                    .json::<Response<update_participants_bulk::ResponseData>>()
                    .await;
                match result {
                    Ok(result) => {
                        tracing::info!("Get participants bulk update result: {:?}", &result);
                        if let Some(data) = result.data {
                            Ok(data.update_participants_bulk)
                        } else {
                            Err(crate::Error::from(
                                "Unknown error: got successful response but incorrect data"
                                    .to_string(),
                            ))
                        }
                    }
                    Err(json_error) => Err(crate::Error::from(json_error)),
                }
            }
            Err(response_error) => Err(crate::Error::from(response_error)),
        }
    }

    pub async fn create_games_bulk(
        &self,
        games: Vec<CreateGameModel>,
    ) -> Result<String, crate::Error> {
        let client = self.client.read().await;
        let query = CreateGamesBulk::build_query(create_games_bulk::Variables { games: games });
        let response = client.post(&self.url).json(&query).send().await;
        match response {
            Ok(response) => {
                let result = response
                    .json::<Response<create_games_bulk::ResponseData>>()
                    .await;
                match result {
                    Ok(result) => {
                        tracing::info!("Get create games bulk result: {:?}", &result);
                        if let Some(data) = result.data {
                            Ok(data.create_games_bulk)
                        } else {
                            Err(crate::Error::from(
                                "Unknown error: got successful response but incorrect data"
                                    .to_string(),
                            ))
                        }
                    }
                    Err(json_error) => Err(crate::Error::from(json_error)),
                }
            }
            Err(response_error) => Err(crate::Error::from(response_error)),
        }
    }

    pub async fn update_users_bulk(
        &self,
        users: Vec<update_users_bulk::UserBulkUpdatePayload>,
    ) -> Result<String, crate::Error> {
        let client = self.client.read().await;
        let query = UpdateUsersBulk::build_query(update_users_bulk::Variables { users: users });
        let response = client.post(&self.url).json(&query).send().await;
        match response {
            Ok(response) => {
                let result = response
                    .json::<Response<update_users_bulk::ResponseData>>()
                    .await;
                match result {
                    Ok(result) => {
                        tracing::info!("Get update users bulk result: {:?}", &result);
                        if let Some(data) = result.data {
                            Ok(data.update_users_bulk)
                        } else {
                            Err(crate::Error::from(
                                "Unknown error: got successful response but incorrect data"
                                    .to_string(),
                            ))
                        }
                    }
                    Err(json_error) => Err(crate::Error::from(json_error)),
                }
            }
            Err(response_error) => Err(crate::Error::from(response_error)),
        }
    }

    pub async fn get_games_of_match_count(&self, match_id: Uuid) -> Result<i64, crate::Error> {
        let client = self.client.read().await;
        let query = GamesCount::build_query(games_count::Variables { match_id: match_id });
        let response = client.post(&self.url).json(&query).send().await;
        match response {
            Ok(response) => {
                let result = response.json::<Response<games_count::ResponseData>>().await;
                match result {
                    Ok(result) => {
                        tracing::info!("Got games count result: {:?}", &result);
                        if let Some(data) = result.data {
                            Ok(data.games_count)
                        } else {
                            Err(crate::Error::from(
                                "Unknown error: got successful response but incorrect data"
                                    .to_string(),
                            ))
                        }
                    }
                    Err(json_error) => Err(crate::Error::from(json_error)),
                }
            }
            Err(response_error) => Err(crate::Error::from(response_error)),
        }
    }

    pub async fn update_match(
        &self,
        id: Uuid,
        report_link: String,
    ) -> Result<String, crate::Error> {
        let client = self.client.read().await;
        let query = UpdateMatch::build_query(update_match::Variables {
            id: id,
            report_link: report_link,
        });
        let response = client.post(&self.url).json(&query).send().await;
        match response {
            Ok(response) => {
                let result = response
                    .json::<Response<update_match::ResponseData>>()
                    .await;
                match result {
                    Ok(result) => {
                        tracing::info!("Got update match result: {:?}", &result);
                        if let Some(data) = result.data {
                            Ok(data.update_match)
                        } else {
                            Err(crate::Error::from(
                                "Unknown error: got successful response but incorrect data"
                                    .to_string(),
                            ))
                        }
                    }
                    Err(json_error) => Err(crate::Error::from(json_error)),
                }
            }
            Err(response_error) => Err(crate::Error::from(response_error)),
        }
    }
}
