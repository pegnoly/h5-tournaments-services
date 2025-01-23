use async_graphql::Context;
use rust_decimal::Decimal;
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
}