use sea_orm::prelude::*;

pub type GameModel = Model;

#[derive(Debug, EnumIter, DeriveActiveEnum, Clone, Copy, PartialEq, Eq, async_graphql::Enum)]
#[sea_orm(rs_type = "i32", db_type = "Integer")]
pub enum GameResult {
    NotSelected = 0,
    FirstPlayerWon = 1,
    SecondPlayerWon = 2
}


#[derive(Clone, Debug, PartialEq, Eq, DeriveEntityModel)]
#[sea_orm(table_name = "games")]
pub struct Model {
    #[sea_orm(primary_key)]
    pub id: Uuid,
    pub match_id: Uuid,
    pub first_player_race: Option<i32>,
    pub first_player_hero: Option<i32>,
    pub second_player_race: Option<i32>,
    pub second_player_hero: Option<i32>,
    pub bargains_color: Option<i32>,
    pub bargains_amount: Option<i32>,
    pub result: GameResult
}

#[derive(Copy, Clone, Debug, EnumIter)]
pub enum Relation {
    Match
}

impl RelationTrait for Relation {
    fn def(&self) -> RelationDef {
        match self {
            Self::Match => Entity::belongs_to(super::match_structure::Entity)
                .from(Column::MatchId)
                .to(super::match_structure::Column::Id)
                .into()
        }
    }
}

impl Related<super::match_structure::Entity> for Entity {
    fn to() -> RelationDef {
        Relation::Match.def()
    }
}

impl ActiveModelBehavior for ActiveModel {}

#[async_graphql::Object]
impl GameModel {
    async fn id(&self) -> Uuid {
        self.id
    }

    async fn match_id(&self) -> Uuid {
        self.match_id
    }

    async fn first_player_race(&self) -> Option<i32> {
        self.first_player_race
    }

    async fn first_player_hero(&self) -> Option<i32> {
        self.first_player_hero
    }

    async fn second_player_race(&self) -> Option<i32> {
        self.second_player_race
    }

    async fn second_player_hero(&self) -> Option<i32> {
        self.second_player_hero
    }

    async fn bargains_color(&self) -> Option<i32> {
        self.bargains_color
    }


    async fn bargains_amount(&self) -> Option<i32> {
        self.bargains_amount
    }

    async fn result(&self) -> GameResult {
        self.result
    }
}

#[derive(Debug, async_graphql::InputObject)]
pub struct CreateGameModel {
    pub match_id: Uuid,
    pub first_player_race: Option<i32>,
    pub first_player_hero: Option<i32>,
    pub second_player_race: Option<i32>,
    pub second_player_hero: Option<i32>,
    pub bargains_color: Option<i32>,
    pub bargains_amount: Option<i32>,
    pub result: GameResult
}