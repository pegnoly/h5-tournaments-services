use sea_orm::{prelude::*, FromJsonQueryResult};
use serde::{Deserialize, Serialize};

use crate::prelude::ModType;

pub type HeroesModel = Model;

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct HeroNew {
    pub id: i32,
    pub race: i32,
    pub name: String
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize, FromJsonQueryResult)]
pub struct HeroesNew {
    pub entities: Vec<HeroNew>
}

#[derive(Clone, Debug, PartialEq, Eq, DeriveEntityModel)]
#[sea_orm(table_name = "heroes_new")]
pub struct Model {
    #[sea_orm(primary_key)]
    pub id: i32,
    pub mod_type: ModType,
    pub heroes: HeroesNew
}

#[derive(Copy, Clone, Debug, EnumIter, DeriveRelation)]
pub enum Relation {}

impl ActiveModelBehavior for ActiveModel {}

#[async_graphql::Object]
impl HeroNew {
    async fn id(&self) -> i32 {
        self.id
    }

    async fn name(&self) -> String {
        self.name.clone()
    }

    async fn race(&self) -> i32 {
        self.race
    }
}

#[async_graphql::Object]
impl HeroesNew {
    async fn entities(&self) -> &Vec<HeroNew> {
        &self.entities
    }
}

#[async_graphql::Object]
impl HeroesModel {
    async fn id(&self) -> i32 {
        self.id
    }

    async fn mod_type(&self) -> ModType {
        self.mod_type
    }

    async fn heroes(&self) -> &HeroesNew {
        &self.heroes
    }
}