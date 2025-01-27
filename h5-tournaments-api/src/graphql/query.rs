use async_graphql::Context;
use sea_orm::{error, DatabaseConnection};
use uuid::Uuid;

use crate::{prelude::TournamentService, services::tournament::models::{match_structure::MatchModel, operator::TournamentOperatorModel, tournament::TournamentModel, user::UserModel}};

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

    async fn tournament<'a>(
        &self,
        context: &Context<'a>,
        #[graphql(desc = "Id of tournament")]
        id: Option<Uuid>,
        #[graphql(desc = "Unique reports channel of tournament")]
        reports_channel_id: Option<String>
    ) -> Result<Option<TournamentModel>, String> {
        let service = context.data::<TournamentService>().unwrap();
        let db = context.data::<DatabaseConnection>().unwrap();
        let res = service.get_tournament(db, id, reports_channel_id).await;

        match res {
            Ok(tournament) => {
                Ok(tournament)
            },
            Err(error) => {
                Err(error)
            }
        }
    }

    async fn user<'a>(
        &self,
        context: &Context<'a>,
        #[graphql(desc = "User's id")]
        id: Option<Uuid>,
        #[graphql(desc = "User's discord id")]
        discord_id: Option<String>
    ) -> Result<Option<UserModel>, String> {
        let service = context.data::<TournamentService>().unwrap();
        let db = context.data::<DatabaseConnection>().unwrap();
        let res = service.get_user(db, id, discord_id).await;

        match res {
            Ok(user) => {
                Ok(user)
            },
            Err(error) => {
                Err(error)
            }
        }
    }

    async fn tournament_match<'a>(
        &self,
        context: &Context<'a>,
        id: Option<Uuid>,
        data_message: Option<String>,
        interaction: Option<String>
    ) -> Result<Option<MatchModel>, String> {
        let service = context.data::<TournamentService>().unwrap();
        let db = context.data::<DatabaseConnection>().unwrap();
        let res = service.get_match(db, id, data_message, interaction).await;

        match res {
            Ok(match_model) => {
                Ok(match_model)
            },
            Err(error) => {
                Err(error)
            }
        }
    }

    async fn users<'a>(
        &self,
        context: &Context<'a>
    ) -> Result<Option<Vec<UserModel>>, String> {
        let service = context.data::<TournamentService>().unwrap();
        let db = context.data::<DatabaseConnection>().unwrap();
        let res = service.get_users(db).await;
        
        match res {
            Ok(users) => {
                Ok(Some(users))
            },
            Err(error) => {
                Err(error)
            }
        }
    }
}