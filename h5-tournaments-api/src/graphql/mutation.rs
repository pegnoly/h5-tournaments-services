use async_graphql::Context;
use sea_orm::DatabaseConnection;
use uuid::Uuid;

use crate::prelude::TournamentService;

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
        data_message: Option<String>
    ) -> Result<String, String> {
        let service = context.data::<TournamentService>().unwrap();
        let db = context.data::<DatabaseConnection>().unwrap();
        let res = service.update_match(db, id, games_count, second_player, data_message).await;
        match res {
            Ok(_res) => {
                Ok("Match updated".to_string())
            },
            Err(error) => {
                Err(error)
            }
        }
    }
}