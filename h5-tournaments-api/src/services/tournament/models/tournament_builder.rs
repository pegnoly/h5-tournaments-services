use sea_orm::prelude::*;

pub type TournamentBuilderModel = Model;

#[derive(Debug, EnumIter, DeriveActiveEnum, Clone, Copy, PartialEq, Eq, async_graphql::Enum)]
#[sea_orm(rs_type = "i32", db_type = "Integer")]
pub enum TournamentEditState {
    NotSelected = 0,
    ChannelsData = 1,
    ReportsData = 2
}

#[derive(Clone, Debug, PartialEq, Eq, DeriveEntityModel)]
#[sea_orm(table_name = "tournament_builders")]
pub struct Model {
    #[sea_orm(primary_key)]
    pub id: Uuid,
    pub message_id: i64,
    pub name: Option<String>,
    pub edit_state: Option<TournamentEditState>,
    pub register_channel: Option<i64>,
    pub reports_channel: Option<i64>,
    pub role: Option<i64>,
    pub use_bargains: Option<bool>,
    pub use_bargains_color: Option<bool>,
    pub use_foreign_heroes: Option<bool>
}

#[derive(Copy, Clone, Debug, EnumIter, DeriveRelation)]
pub enum Relation {}

impl ActiveModelBehavior for ActiveModel {}

#[async_graphql::Object]
impl TournamentBuilderModel {
    async fn id(&self) -> Uuid {
        self.id
    }

    async fn message(&self) -> i64 {
        self.message_id
    }

    async fn name(&self) -> Option<String> {
        self.name.clone()
    }

    async fn edit_state(&self) -> Option<TournamentEditState> {
        self.edit_state
    }

    async fn register_channel(&self) -> Option<i64> {
        self.register_channel
    }

    async fn reports_channel(&self) -> Option<i64> {
        self.reports_channel
    }

    async fn role(&self) -> Option<i64> {
        self.role
    }

    async fn use_bargains(&self) -> Option<bool> {
        self.use_bargains
    }

    async fn use_bargains_color(&self) -> Option<bool> {
        self.use_bargains_color
    }

    async fn use_foreign_heroes(&self) -> Option<bool> {
        self.use_foreign_heroes
    }
}