use sea_orm::prelude::*;

pub type OrganizerModel = Model;

#[derive(Clone, Debug, PartialEq, Eq, DeriveEntityModel)]
#[sea_orm(table_name = "tournament_organizers")]
pub struct Model {
    #[sea_orm(primary_key)]
    pub id: Uuid,
    pub discord_id: i64,
    pub challonge_api_key: String
}

#[derive(Copy, Clone, Debug, EnumIter, DeriveRelation)]
pub enum Relation {}

impl ActiveModelBehavior for ActiveModel {}

#[async_graphql::Object]
impl OrganizerModel {
    async fn id(&self) -> Uuid {
        self.id
    }

    async fn discord(&self) -> i64 {
        self.discord_id
    }

    async fn challonge(&self) -> String {
        self.challonge_api_key.clone()
    }
}