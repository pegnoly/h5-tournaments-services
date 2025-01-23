use sea_orm::prelude::*;

pub type HeroModel = Model;

#[derive(Clone, Debug, PartialEq, Eq, DeriveEntityModel)]
#[sea_orm(table_name = "heroes")]
pub struct Model {
    #[sea_orm(primary_key)]
    pub id: i32,
    pub race: i32,
    pub actual_name: String
}

#[derive(Copy, Clone, Debug, EnumIter, DeriveRelation)]
pub enum Relation {}

impl ActiveModelBehavior for ActiveModel {}

#[async_graphql::Object]
impl HeroModel {
    async fn id(&self) -> i32 {
        self.id
    }

    async fn race(&self) -> i32 {
        self.race
    }

    async fn name(&self) -> String {
        self.actual_name.clone()
    }
}