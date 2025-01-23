use async_graphql::Context;
use sea_orm::{error, DatabaseConnection};
use uuid::Uuid;

use crate::{prelude::TournamentService, services::tournament::models::operator::TournamentOperatorModel};

pub struct Query;

#[async_graphql::Object]
impl Query {
    async fn operator<'a>(
        &self, 
        context: &Context<'a>,
        id: Uuid
    ) -> Result<Option<TournamentOperatorModel>, String> {
        let service = context.data::<TournamentService>().unwrap();
        let db = context.data::<DatabaseConnection>().unwrap();
        let res = service.get_operator(db, id).await;

        match res {
            Ok(operator) => {
                Ok(operator)
            },
            Err(error) => {
                Err(error)
            }
        }
    }
}