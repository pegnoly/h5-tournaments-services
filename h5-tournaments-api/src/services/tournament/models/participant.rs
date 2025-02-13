use sea_orm::prelude::*;

pub type TournamentParticipantModel = Model;

#[derive(Clone, Debug, PartialEq, Eq, DeriveEntityModel)]
#[sea_orm(table_name = "participants")] 
pub struct Model {
    #[sea_orm(primary_key)]
    pub id: Uuid,
    pub tournament_id: Uuid,
    pub user_id: Uuid,
    pub group_number: i32
}

#[derive(Copy, Clone, Debug, EnumIter)]
pub enum Relation {
    User
}

impl RelationTrait for Relation {
    fn def(&self) -> RelationDef {
        match self {
            Self::User => Entity::belongs_to(super::user::Entity)
                .from(Column::UserId)
                .to(super::user::Column::Id)
                .into()
        }
    }
}

impl Related<super::user::Entity> for Entity {
    fn to() -> RelationDef {
        Relation::User.def()
    }
}

impl ActiveModelBehavior for ActiveModel {}

#[async_graphql::Object]
impl TournamentParticipantModel {
    async fn id(&self) -> Uuid {
        self.id
    }

    async fn tournament(&self) -> Uuid {
        self.tournament_id
    }

    async fn user(&self) -> Uuid {
        self.user_id
    }

    async fn group(&self) -> i32 {
        self.group_number
    }
}