use async_graphql::Context;
use sea_orm::DatabaseConnection;
use uuid::Uuid;

use crate::{prelude::TournamentService, services::tournament::models::game_builder::{GameBuilderModel, GameEditState, GameResult}};

pub struct Mutation;

#[async_graphql::Object]
impl Mutation {
    async fn create_user<'a>(
        &self,
        context: &Context<'a>,
        #[graphql(desc = "User's nickname")]
        name: String,
        #[graphql(desc = "User's discord id")]
        discord: String
    ) -> Result<String, String> {
        let service = context.data::<TournamentService>().unwrap();
        let db = context.data::<DatabaseConnection>().unwrap();
        let res = service.create_user(db, name, discord).await;
        tracing::info!("Insert res: {:?}", &res);
        match res {
            Ok(res) => {
                Ok(res)
            },
            Err(error) => {
                Err(error)
            }
        }
    }
    
    async fn create_tournament<'a>(
        &self,
        context: &Context<'a>,
        name: String,
        operator_id: Uuid,
        channel_id: String
    ) -> Result<String, String> {
        let service = context.data::<TournamentService>().unwrap();
        let db = context.data::<DatabaseConnection>().unwrap();
        let res = service.create_tournament(db, name, operator_id, channel_id).await;
        match res {
            Ok(res) => {
                Ok(res)
            },
            Err(error) => {
                Err(error)
            }
        }
    }

    async fn create_match<'a>(
        &self,
        context: &Context<'a>,
        tournament_id: Uuid,
        interaction: String,
        first_player: Uuid
    ) -> Result<String, String> {
        let service = context.data::<TournamentService>().unwrap();
        let db = context.data::<DatabaseConnection>().unwrap();
        let res = service.create_match(db, tournament_id, interaction, first_player).await;
        match res {
            Ok(_res) => {
                Ok("Match created".to_string())
            },
            Err(error) => {
                Err(error)
            }
        }
    }

    async fn update_match<'a>(
        &self,
        context: &Context<'a>,
        id: Uuid,
        games_count: Option<i32>,
        second_player: Option<Uuid>,
        data_message: Option<String>,
        current_game: Option<i32>
    ) -> Result<String, String> {
        let service = context.data::<TournamentService>().unwrap();
        let db = context.data::<DatabaseConnection>().unwrap();
        let res = service.update_match(db, id, games_count, second_player, data_message, current_game).await;
        match res {
            Ok(_res) => {
                Ok("Match updated".to_string())
            },
            Err(error) => {
                Err(error)
            }
        }
    }

    async fn create_game<'a>(
        &self,
        context: &Context<'a>,
        match_id: Uuid,
        number: i16
    ) -> Result<GameBuilderModel, String> {
        let service = context.data::<TournamentService>().unwrap();
        let db = context.data::<DatabaseConnection>().unwrap();
        let res = service.create_game(db, match_id, number).await;
        match res {
            Ok(_res) => {
                Ok(_res)
            },
            Err(error) => {
                Err(error)
            }
        }
    }

    async fn update_game<'a>(
        &self,
        context: &Context<'a>,
        match_id: Uuid,
        number: i32,
        edit_state: Option<GameEditState>,
        first_player_race: Option<i32>,
        first_player_hero: Option<i32>,
        second_player_race: Option<i32>,
        second_player_hero: Option<i32>,
        bargains_amount: Option<i32>,
        result: Option<GameResult>
    ) -> Result<String, String> {
        let service = context.data::<TournamentService>().unwrap();
        let db = context.data::<DatabaseConnection>().unwrap();
        let res = service.update_game(
            db, 
            match_id, 
            number, 
            edit_state, 
            first_player_race, 
            first_player_hero, 
            second_player_race, 
            second_player_hero, 
            bargains_amount,
            result
        ).await;
        match res {
            Ok(_res) => {
                Ok(_res)
            },
            Err(error) => {
                Err(error)
            }
        }
    }

    async fn create_participant<'a>(
        &self,
        context: &Context<'a>,
        tournament_id: Uuid,
        user_id: Uuid,
        group: i32
    ) -> Result<String, String> {
        let service = context.data::<TournamentService>().unwrap();
        let db = context.data::<DatabaseConnection>().unwrap();
        let res = service.create_participant(db, tournament_id, user_id, group).await;

        match res {
            Ok(_res) => {
                Ok(_res)
            },
            Err(error) => {
                Err(error)
            }
        }
    }
}