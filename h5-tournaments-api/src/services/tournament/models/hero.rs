use sea_orm::{prelude::*, FromJsonQueryResult};
use serde::{Deserialize, Serialize};

use crate::services::tournament::models::tournament::ModType;

pub type HeroesModel = Model;

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct Hero {
    pub id: i32,
    pub race: i32,
    pub name: String
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize, FromJsonQueryResult)]
pub struct Heroes {
    pub entities: Vec<Hero>
}

#[derive(Clone, Debug, PartialEq, Eq, DeriveEntityModel)]
#[sea_orm(table_name = "heroes_new")]
pub struct Model {
    #[sea_orm(primary_key)]
    pub id: i32,
    pub mod_type: ModType,
    pub heroes: Heroes
}

#[derive(Copy, Clone, Debug, EnumIter, DeriveRelation)]
pub enum Relation {}

impl ActiveModelBehavior for ActiveModel {}

#[async_graphql::Object]
impl Hero {
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
impl Heroes {
    async fn entities(&self) -> &Vec<Hero> {
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

    async fn heroes(&self) -> &Heroes {
        &self.heroes
    }
}