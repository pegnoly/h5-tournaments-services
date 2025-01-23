use sea_orm::prelude::*;

pub type UserModel = Model;

#[derive(Clone, Debug, PartialEq, Eq, DeriveEntityModel)]
#[sea_orm(table_name = "users")] 
pub struct Model {
    #[sea_orm(primary_key)]
    pub id: Uuid,
    pub discord_id: i64,
    pub nickname: String
}

#[derive(Copy, Clone, Debug, EnumIter, DeriveRelation)]
pub enum Relation {}

impl ActiveModelBehavior for ActiveModel {}

#[async_graphql::Object]
impl UserModel {
    async fn id(&self) -> Uuid {
        self.id
    }

    async fn discord_id(&self) -> i64 {
        self.discord_id
    }

    async fn nickname(&self) -> String {
        self.nickname.clone()
    }
}