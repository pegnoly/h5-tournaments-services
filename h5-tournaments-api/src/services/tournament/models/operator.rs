use sea_orm::prelude::*;

#[derive(Clone, Debug, PartialEq, Eq, DeriveEntityModel)]
#[sea_orm(table_name = "tournament_operators")]
pub struct Model {
    #[sea_orm(primary_key)]
    pub id: Uuid,
    pub name: String,
    pub server_id: i64,
    pub heroes: Vec<i32>,
    pub section_id: i64,
    pub generated_channel: i64
}

#[derive(Copy, Clone, Debug, EnumIter, DeriveRelation)]
pub enum Relation {}

impl ActiveModelBehavior for ActiveModel {}

pub type TournamentOperatorModel = Model;

#[async_graphql::Object]
impl TournamentOperatorModel {
    async fn id(&self) -> Uuid {
        self.id
    }

    async fn name(&self) -> String {
        self.name.clone()
    }

    async fn server(&self) -> i64 {
        self.server_id
    }

    async fn heroes(&self) -> Vec<i32> {
        self.heroes.clone()
    }
 
    async fn section(&self) -> i64 {
        self.section_id
    }

    async fn generated(&self) -> i64 {
        self.generated_channel
    }
}