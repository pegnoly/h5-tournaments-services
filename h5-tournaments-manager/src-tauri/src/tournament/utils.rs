use h5_stats_types::{Game, Hero, Race, Tournament};
use serde::{Serialize, Deserialize};
use strum::{EnumIter, FromRepr};
use uuid::Uuid;


#[derive(Debug, Serialize, Deserialize)]
pub struct RaceFrontendModel {
    pub id: i32,
    pub actual_name: String
}

impl From<Race> for RaceFrontendModel {
    fn from(value: Race) -> Self {
        RaceFrontendModel {
            id: value.id as i32,
            actual_name: value.actual_name
        }
    }
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct HeroFrontendModel {
    pub id: i32,
    pub race: i32,
    pub actual_name: String
}

impl From<Hero> for HeroFrontendModel {
    fn from(value: Hero) -> Self {
        HeroFrontendModel {
            id: value.id as i32,
            race: value.race as i32,
            actual_name: value.actual_name
        }
    }
}


#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct TournamentFrontendModel {
    pub id: Uuid,
    pub name: String
}

impl From<Tournament> for TournamentFrontendModel {
    fn from(value: Tournament) -> Self {
        TournamentFrontendModel { 
            id: value.id, 
            name: value.name
        }
    }
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct GameFrontendModel {
    pub id: Uuid,
    pub first_player_race: i32,
    pub first_player_hero: i32,
    pub second_player_race: i32,
    pub second_player_hero: i32,
    pub bargains_color: i16,
    pub bargains_amount: i16,
    pub result: i16
}

impl From<Game> for GameFrontendModel {
    fn from(value: Game) -> Self {
        GameFrontendModel {
            id: value.id,
            first_player_race: value.first_player_race as i32,
            first_player_hero: value.first_player_hero as i32,
            second_player_race: value.second_player_race as i32,
            second_player_hero: value.second_player_hero as i32,
            bargains_color: value.bargains_color as i16,
            bargains_amount: value.bargains_amount,
            result: value.result as i16
        }
    }
}