use sea_orm::prelude::*;

pub type GameBuilderModel = Model;

#[derive(Debug, EnumIter, DeriveActiveEnum, Clone, Copy, PartialEq, Eq, async_graphql::Enum)]
#[sea_orm(rs_type = "i32", db_type = "Integer")]
pub enum GameEditState {
    NotSelected = 0,
    PlayerData = 1,
    OpponentData = 2,
    ResultData = 3
}


#[derive(Clone, Debug, PartialEq, Eq, DeriveEntityModel)]
#[sea_orm(table_name = "games_new")]
pub struct Model {
    #[sea_orm(primary_key)]
    pub id: Uuid,
    pub match_id: Uuid,
    pub number: i16,
    pub edit_state: Option<GameEditState>,
    pub first_player_race: Option<i32>,
    pub first_player_hero: Option<i32>,
    pub second_player_race: Option<i32>,
    pub second_player_hero: Option<i32>,
    pub bargains_amount: Option<i32>
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
impl GameBuilderModel {
    async fn id(&self) -> Uuid {
        self.id
    }

    async fn match_id(&self) -> Uuid {
        self.match_id
    }

    async fn number(&self) -> i16 {
        self.number
    }

    async fn edit_state(&self) -> Option<GameEditState> {
        self.edit_state
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

    async fn bargains_amount(&self) -> Option<i32> {
        self.bargains_amount
    }
}