use h5_tournaments_api::prelude::*;

pub struct StatsGeneratorDataModel {
    pub heroes_data: Vec<Hero>,
    pub races_data: Vec<Race>,
    pub games_data: Vec<Game>,
    pub matches_data: Vec<Match>,
    pub bargains_data: Vec<BargainsColorModel>,
    pub results_data: Vec<GameResultModel>,
}

impl StatsGeneratorDataModel {

    pub fn new() -> Self {
        StatsGeneratorDataModel {
            heroes_data: vec![],
            races_data: vec![],
            matches_data: vec![],
            games_data: vec![],
            bargains_data: vec![
                BargainsColorModel { id: BargainsColor::ColorRed, name: "Красный".to_string() }, 
                BargainsColorModel { id: BargainsColor::ColorBlue, name: "Синий".to_string() }
            ],
            results_data: vec![
                GameResultModel { id: GameResult::FirstPlayerWon, name: "Победа 1 игрока".to_string() },
                GameResultModel { id: GameResult::SecondPlayerWon, name: "Победа 2 игрока".to_string() }
            ]
        }
    }
}