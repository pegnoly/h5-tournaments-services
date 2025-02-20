use sea_orm::prelude::*;

pub type MatchModel = Model;

#[derive(Clone, Debug, PartialEq, Eq, DeriveEntityModel)]
#[sea_orm(table_name = "matches")]
pub struct Model {
    #[sea_orm(primary_key)]
    pub id: Uuid,
    pub tournament_id: Uuid,
    // Message that invoked creation of this match
    pub message_id: i64,
    pub first_player: Uuid,
    pub second_player: Uuid,
    pub challonge_id: String
}

#[derive(Copy, Clone, Debug, EnumIter)]
pub enum Relation {
    Game
}

impl RelationTrait for Relation {
    fn def(&self) -> RelationDef {
        match self {
            Self::Game => Entity::has_many(super::game_builder::Entity).into()
        }
    }
}

impl Related<super::game_builder::Entity> for Entity {
    fn to() -> RelationDef {
        Relation::Game.def()
    }
}

impl ActiveModelBehavior for ActiveModel {}

#[async_graphql::Object]
impl MatchModel {
    async fn id(&self) -> Uuid {
        self.id
    }

    async fn tournament(&self) -> Uuid {
        self.tournament_id
    }

    async fn message(&self) -> i64 {
        self.message_id
    }

    async fn first_player(&self) -> Uuid {
        self.first_player
    }

    async fn second_player(&self) -> Uuid {
        self.second_player
    }

    async fn challonge(&self) -> String {
        self.challonge_id.clone()
    }
}