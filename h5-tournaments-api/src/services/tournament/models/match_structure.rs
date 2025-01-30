use sea_orm::prelude::*;

pub type MatchModel = Model;

#[derive(Clone, Debug, PartialEq, Eq, DeriveEntityModel)]
#[sea_orm(table_name = "matches_new")]
pub struct Model {
    #[sea_orm(primary_key)]
    pub id: Uuid,
    pub tournament_id: Uuid,
    // Message that invoked creation of this match
    pub interaction_id: i64,
    // Message contains data of this match
    pub data_message: Option<i64>,
    pub first_player: Uuid,
    pub second_player: Option<Uuid>,
    pub games_count: Option<i32>,
    pub current_game: i32,
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

    async fn interaction(&self) -> i64 {
        self.interaction_id
    }

    async fn data(&self) -> Option<i64> {
        self.data_message
    }

    async fn first_player(&self) -> Uuid {
        self.first_player
    }

    async fn second_player(&self) -> Option<Uuid> {
        self.second_player
    }

    async fn games_count(&self) -> Option<i32> {
        self.games_count
    }

    async fn current_game(&self) -> i32 {
        self.current_game
    }
}