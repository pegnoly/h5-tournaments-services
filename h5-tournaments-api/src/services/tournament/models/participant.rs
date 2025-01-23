use sea_orm::prelude::*;

pub type TournamentParticipantModel = Model;

#[derive(Clone, Debug, PartialEq, Eq, DeriveEntityModel)]
#[sea_orm(table_name = "participants")] 
pub struct Model {
    #[sea_orm(primary_key)]
    pub id: Uuid,
    pub tournament_id: Uuid,
    pub user_id: Uuid
}

#[derive(Copy, Clone, Debug, EnumIter, DeriveRelation)]
pub enum Relation {}

impl ActiveModelBehavior for ActiveModel {}

#[async_graphql::Object]
impl TournamentParticipantModel {
    async fn id(&self) -> Uuid {
        self.id
    }

    async fn tournament(&self) -> Uuid {
        self.tournament_id
    }

    async fn user(&self) -> Uuid {
        self.user_id
    }
}