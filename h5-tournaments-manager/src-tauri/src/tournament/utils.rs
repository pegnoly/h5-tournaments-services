use serde::{Serialize, Deserialize};
use strum::{EnumIter, FromRepr};
use uuid::Uuid;

#[derive(Debug, Serialize, Deserialize, EnumIter, FromRepr, Clone, Copy)]
#[repr(i32)]
pub enum RaceType {
    NotDetected = 0,
    Heaven = 1,
    Inferno = 2,
    Necropolis = 3,
    Preserve = 4, 
    Dungeon = 5, 
    Academy = 6, 
    Fortress = 7,
    Stronghold = 8
}

#[derive(Debug, Serialize, Deserialize, EnumIter, FromRepr, Clone, Copy)]
#[repr(i32)]
pub enum HeroType {
    NotDetected = 0,
    Orrin = 1,
    Mardigo = 2,
    Nathaniel = 3,
    Maeve = 4,
    Brem = 5,
    Sarge = 6,
    Christian = 7,
    Ving = 8,

    Oddrema = 9,
    Nymus = 10,
    Calid = 11,
    Deleb = 12,
    Grok = 13,
    Marder = 14,
    Efion = 15,
    Jazaz = 16,

    Gles = 17,
    Nemor = 18,
    Aberrar = 19,
    Tamika = 20,
    Pelt = 21,
    Straker = 22,
    Muscip = 23,
    Effig = 24,

    Metlirn = 25,
    Nadaur = 26,
    Diraya = 27,
    Elleshar = 28,
    Ossir = 29,
    Gillion = 30,
    Itil = 31,
    Linaas = 32,

    Almegir = 33,
    Urunir = 34,
    Menel = 35,
    Eruina = 36,
    Dalom = 37,
    Ferigl = 38,
    Ohtarig = 39,
    Inagost = 40,

    Tan = 41,
    Astral = 42,
    Havez = 43,
    Faiz = 44,
    Isher = 45,
    Razzak = 46,
    Nur = 47,
    Sufi = 48,

    Ingvar = 49,
    Bersy = 50,
    Skeggy = 51,
    Brand = 52,
    Ottar = 53,
    Egil = 54,
    Una = 55,
    Vegeyr = 56,
    
    Hero1 = 57,
    Hero2 = 58,
    Hero3 = 59,
    Hero4 = 60,
    Hero6 = 61,
    Hero7 = 62,
    Hero8 = 63,
    Hero9 = 64
}


#[derive(Debug, Serialize, Deserialize)]
pub struct Race {
    pub id: RaceType,
    pub actual_name: String
}

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

#[derive(Debug, Serialize, Deserialize)]
pub struct Hero {
    pub id: HeroType,
    pub race: RaceType,
    pub actual_name: String
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

#[derive(Debug, Serialize, Deserialize)]
pub struct Tournament {
    pub id: Uuid,
    pub server_id: i64,
    pub channel_id: i64,
    pub name: String
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

/// A match between two players in a concrete tournament. Contains Games.
#[derive(Debug, Serialize, Deserialize, Default)]
pub struct Match {
    pub id: Uuid,
    pub tournament_id: Uuid,
    pub first_player: String,
    pub second_player: String
}

/// Possible game outcomes
#[derive(Debug, Serialize, Deserialize, FromRepr)]
#[repr(i16)]
pub enum GameResult {
    NotDetected = 0,
    FirstPlayerWon = 1,
    SecondPlayerWon = 2
}

/// Possible colors used in bargains
#[derive(Debug, Clone, Copy, Serialize, Deserialize, FromRepr)]
#[repr(i16)]
pub enum BargainsColor {
    NotDetected,
    ColorRed,
    ColorBlue
}


/// A single game between two players.
#[derive(Debug, Serialize, Deserialize)]
pub struct Game {
    pub id: Uuid,
    pub match_id: Uuid,
    pub first_player_race: RaceType,
    pub first_player_hero: HeroType,
    pub second_player_race: RaceType,
    pub second_player_hero: HeroType,
    pub bargains_color: BargainsColor,
    pub bargains_amount: i16,
    pub result: GameResult
}

impl Default for Game {
    fn default() -> Self {
        Game {
            first_player_race: RaceType::NotDetected,
            first_player_hero: HeroType::NotDetected,
            second_player_race: RaceType::NotDetected,
            second_player_hero: HeroType::NotDetected,
            bargains_color: BargainsColor::NotDetected,
            result: GameResult::NotDetected,
            id: uuid::Uuid::new_v4(),
            match_id: uuid::Uuid::default(),
            bargains_amount: 0
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