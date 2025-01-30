use std::str::FromStr;

use graphql_client::{reqwest::post_graphql, GraphQLQuery, Response};
use h5_tournaments_api::prelude::{Hero, ModType, Race, Tournament};
use uuid::Uuid;

use crate::{graphql::queries::{self, create_game_mutation::{self, CreateGameMutationCreateGame}, create_user_mutation::ResponseData, get_game_query::{self, GetGameQueryGame}, get_games_query::{self, GetGamesQueryGames}, get_hero_query::{self, GetHeroQueryHero}, get_heroes_query::{self, GetHeroesQueryHeroes}, get_match_query::GetMatchQueryTournamentMatch, get_operator_data_query::GetOperatorDataQueryOperator, get_user_query::GetUserQueryUser, update_game_mutation, CreateGameMutation, CreateMatchMutation, CreateTournamentMutation, CreateUserMutation, GameEditState, GetGameQuery, GetGamesQuery, GetHeroQuery, GetHeroesQuery, GetMatchQuery, GetOperatorDataQuery, GetOperatorSectionQuery, GetTournamentQuery, GetUserQuery, GetUsersQuery, GetUsersResult, UpdateGameMutation, UpdateMatchMutation}, parser::service::ParsedData};

pub(self) const MAIN_URL: &'static str = "https://h5-tournaments-api-5epg.shuttle.app/";

pub struct RaceNew {
    pub id: i64,
    pub name: String
}

pub struct ApiConnectionService {
    client: tokio::sync::RwLock<reqwest::Client>,
    pub races: Vec<RaceNew>
}


impl ApiConnectionService {
    pub fn new(client: reqwest::Client) -> Self {
        ApiConnectionService {
            client: tokio::sync::RwLock::new(client),
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

    pub async fn create_user(&self, nickname: String, id: String) -> Result<String, crate::Error> {
        let variables = crate::graphql::queries::create_user_mutation::Variables {
            name: nickname,
            discord: id
        };
        
        let client = self.client.read().await;
        let query = CreateUserMutation::build_query(variables);
        let response = client.post(MAIN_URL).json(&query).send().await;
        match response {
            Ok(response) => {
                // tracing::info!("Responce: {:?}", &response.text().await.unwrap());
                // Ok("test".to_string())
                let result = response.json::<Response<ResponseData>>().await;
                match result {
                    Ok(result) => {
                        if let Some(data) = result.data {
                            Ok(data.create_user)
                        }
                        else {
                            if let Some(errors) = result.errors {
                                Ok(errors.iter().map(|e| e.to_string()).collect::<Vec<String>>().concat().into())
                            }
                            else {
                                Ok("Unknown interaction: no data and no errors returned".to_string())
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
        let response = client.post(MAIN_URL).json(&query).send().await;
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
        let variables = queries::get_operator_data_query::Variables {
            id: id
        };

        let client = self.client.read().await;
        let query = GetOperatorDataQuery::build_query(variables);
        let response = client.post(MAIN_URL).json(&query).send().await;
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


    pub async fn create_tournament(&self, name: String, operator_id: Uuid, channel_id: i64) -> Result<String, crate::Error>{
        let variables = crate::graphql::queries::create_tournament_mutation::Variables {
            name: name.clone(),
            operator_id: operator_id,
            channel_id: channel_id.to_string()
        };

        let client = self.client.read().await;
        let query = CreateTournamentMutation::build_query(variables);
        let response = client.post(MAIN_URL).json(&query).send().await;
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

    pub async fn get_tournament_data(&self, id: Option<Uuid>, channel_id: Option<String>) -> Result<Option<queries::GetTournamentResult>, crate::Error> {
        let variables = queries::get_tournament_query::Variables {
            reports_channel_id: channel_id,
            id: id
        };

        let client = self.client.read().await;
        let query = GetTournamentQuery::build_query(variables);
        let response = client.post(MAIN_URL).json(&query).send().await;
        match response {
            Ok(response) => {
                let result = response.json::<Response<queries::get_tournament_query::ResponseData>>().await;
                match result {
                    Ok(result) => {
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

    pub async fn get_user(&self, id: Option<Uuid>, discord_id: Option<String>) -> Result<Option<GetUserQueryUser>, crate::Error> {
        let variables = queries::get_user_query::Variables {
            discord_id: discord_id,
            id: id
        };

        let client = self.client.read().await;
        let query = GetUserQuery::build_query(variables);
        let response = client.post(MAIN_URL).json(&query).send().await;
        match response {
            Ok(response) => {
                let result = response.json::<Response<queries::get_user_query::ResponseData>>().await;
                match result {
                    Ok(result) => {
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
        let response = client.post(MAIN_URL).json(&query).send().await;
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

    pub async fn get_match(&self, id: Option<Uuid>, interaction_id: Option<String>, data_message: Option<String>) -> Result<Option<queries::GetMatchResult>, crate::Error> {
        let variables = queries::get_match_query::Variables {
            id: id,
            interaction: interaction_id,
            data_message: data_message
        };

        let client = self.client.read().await;
        let query = GetMatchQuery::build_query(variables);
        let response = client.post(MAIN_URL).json(&query).send().await;
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

    pub async fn update_match(
        &self, 
        id: Uuid, 
        data_message: Option<String>, 
        games_count: Option<i64>, 
        second_player: Option<Uuid>,
        current_game: Option<i64>
    ) -> Result<String, crate::Error> {
        let variables = queries::update_match_mutation::Variables {
            id: id,
            data_message: data_message,
            games_count: games_count,
            second_player: second_player,
            current_game: current_game
        };

        let client = self.client.read().await;
        let query = UpdateMatchMutation::build_query(variables);
        let response = client.post(MAIN_URL).json(&query).send().await;
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
        let response = client.post(MAIN_URL).json(&query).send().await;
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
        let response = client.post(MAIN_URL).json(&query).send().await;
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
        let response = client.post(MAIN_URL).json(&query).send().await;
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

    pub async fn update_game(
        &self,
        match_id: Uuid,
        number: i64,
        edit_state: Option<update_game_mutation::GameEditState>,
        first_player_race: Option<i64>,
        first_player_hero: Option<i64>,
        second_player_race: Option<i64>,
        second_player_hero: Option<i64>,
        bargains_amount: Option<i64>,
        result: Option<update_game_mutation::GameResult>
    ) -> Result<String, crate::Error> {
        let variables = update_game_mutation::Variables {
            match_id: match_id,
            number: number,
            edit_state: edit_state,
            first_player_race: first_player_race,
            first_player_hero: first_player_hero,
            second_player_race: second_player_race,
            second_player_hero: second_player_hero,
            bargains_amount: bargains_amount,
            result: result
        };

        let client = self.client.read().await;
        let query = UpdateGameMutation::build_query(variables);
        let response = client.post(MAIN_URL).json(&query).send().await;
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
        let response = client.post(MAIN_URL).json(&query).send().await;
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
        let response = client.post(MAIN_URL).json(&query).send().await;
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
        let response = client.post(MAIN_URL).json(&query).send().await;
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
}