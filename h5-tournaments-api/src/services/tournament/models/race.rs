use sea_orm::prelude::*;

pub type RaceModel = Model;

#[derive(Clone, Debug, PartialEq, Eq, DeriveEntityModel)]
#[sea_orm(table_name = "races")]
pub struct Model {
    #[sea_orm(primary_key)]
    pub id: i32,
    pub name: String
}

#[derive(Copy, Clone, Debug, EnumIter, DeriveRelation)]
pub enum Relation {}

impl ActiveModelBehavior for ActiveModel {}

#[async_graphql::Object]
impl RaceModel {
    async fn id(&self) -> i32 {
        self.id
    }

    async fn name(&self) -> String {
        self.name.clone()
    }
}