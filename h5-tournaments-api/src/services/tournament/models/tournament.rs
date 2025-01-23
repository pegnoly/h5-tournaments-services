use sea_orm::prelude::*;

pub type TournamentModel = Model;

#[derive(Clone, Debug, PartialEq, Eq, DeriveEntityModel)]
#[sea_orm(table_name = "tournaments_new")]
pub struct Model {
    #[sea_orm(primary_key)]
    pub id: Uuid,
    pub operator_id: Uuid,
    pub channel_id: i64,
    pub name: String
}

#[derive(Copy, Clone, Debug, EnumIter, DeriveRelation)]
pub enum Relation {}

impl ActiveModelBehavior for ActiveModel {}

#[async_graphql::Object]
impl TournamentModel {
    async fn id(&self) -> Uuid {
        self.id
    }

    async fn operator(&self) -> Uuid {
        self.operator_id
    }

    async fn channel(&self) -> i64 {
        self.channel_id
    }

    async fn name(&self) -> String {
        self.name.clone()
    }
}