use uuid::Uuid;

use crate::graphql::queries::{get_tournament_query, get_user_query};

#[derive(Debug, Default)]
pub struct GetMatch {
    pub id: Option<Uuid>,
    pub interaction_id: Option<String>,
    pub message_id: Option<String>,
}

impl GetMatch {
    pub fn with_id(mut self, id: Uuid) -> Self {
        self.id = Some(id);
        self
    }

    pub fn with_interaction_id(mut self, interaction_id: String) -> Self {
        self.interaction_id = Some(interaction_id);
        self
    }

    pub fn with_message_id(mut self, message_id: u64) -> Self {
        self.message_id = Some(message_id.to_string());
        self
    }
}

// impl From<GetMatch> for get_match_query::Variables {
//     fn from(value: GetMatch) -> Self {
//         get_match_query::Variables { id: value.id, data_message: value.message_id, interaction: value.interaction_id }
//     }
// }

// #[derive(Debug)]
// pub struct UpdateGame {
//     pub match_id: Uuid,
//     pub game_number: i64,
//     pub edit_state: Option<update_game_mutation::GameEditState>,
//     pub first_player_race: Option<i64>,
//     pub first_player_hero: Option<i64>,
//     pub second_player_race: Option<i64>,
//     pub second_player_hero: Option<i64>,
//     pub bargains_amount: Option<i64>,
//     pub result: Option<update_game_mutation::GameResult>
// }

// impl UpdateGame {
//     pub fn new(match_id: Uuid, game_number: i64) -> Self {
//         UpdateGame {
//             match_id: match_id,
//             game_number: game_number,
//             edit_state: None,
//             first_player_race: None,
//             first_player_hero: None,
//             second_player_race: None,
//             second_player_hero: None,
//             bargains_amount: None,
//             result: None
//         }
//     }

//     pub fn with_edit_state(mut self, state: update_game_mutation::GameEditState) -> Self {
//         self.edit_state = Some(state);
//         self
//     }

//     pub fn with_first_player_race(mut self, race: i64) -> Self {
//         self.first_player_race = Some(race);
//         self
//     }

//     pub fn with_first_player_hero(mut self, hero: i64) -> Self {
//         self.first_player_hero = Some(hero);
//         self
//     }

//     pub fn with_second_player_race(mut self, race: i64) -> Self {
//         self.second_player_race = Some(race);
//         self
//     }

//     pub fn with_second_player_hero(mut self, hero: i64) -> Self {
//         self.second_player_hero = Some(hero);
//         self
//     }

//     pub fn with_bargains_amount(mut self, amount: i64) -> Self {
//         self.bargains_amount = Some(amount);
//         self
//     }

//     pub fn with_result(mut self, result: update_game_mutation::GameResult) -> Self {
//         self.result = Some(result);
//         self
//     }
// }

// impl From<UpdateGame> for update_game_mutation::Variables {
//     fn from(value: UpdateGame) -> Self {
//         update_game_mutation::Variables {
//             match_id: value.match_id,
//             number: value.game_number,
//             edit_state: value.edit_state,
//             first_player_race: value.first_player_race,
//             first_player_hero: value.first_player_hero,
//             second_player_race: value.second_player_race,
//             second_player_hero: value.second_player_hero,
//             bargains_amount: value.bargains_amount,
//             result: value.result
//         }
//     }
// }

#[derive(Debug)]
pub struct UpdateMatch {
    pub id: Uuid,
    pub message: Option<String>,
    pub games_count: Option<i64>,
    pub second_player: Option<Uuid>,
    pub current_game: Option<i64>,
}

impl UpdateMatch {
    pub fn new(id: Uuid) -> Self {
        UpdateMatch {
            id: id,
            message: None,
            games_count: None,
            second_player: None,
            current_game: None,
        }
    }

    pub fn with_message(mut self, message: String) -> Self {
        self.message = Some(message);
        self
    }

    pub fn with_games_count(mut self, count: i64) -> Self {
        self.games_count = Some(count);
        self
    }

    pub fn with_second_player(mut self, player: Uuid) -> Self {
        self.second_player = Some(player);
        self
    }

    pub fn with_current_game(mut self, game: i64) -> Self {
        self.current_game = Some(game);
        self
    }
}

// impl From<UpdateMatch> for update_match_mutation::Variables {
//     fn from(value: UpdateMatch) -> Self {
//         update_match_mutation::Variables {
//             id: value.id,
//             games_count: value.games_count,
//             second_player: value.second_player,
//             data_message: value.message,
//             current_game: value.current_game
//         }
//     }
// }

#[derive(Debug, Default)]
pub struct GetTournament {
    pub id: Option<Uuid>,
    pub reports_channel_id: Option<String>,
    pub register_channel_id: Option<String>,
}

impl GetTournament {
    pub fn with_id(mut self, id: Uuid) -> Self {
        self.id = Some(id);
        self
    }

    pub fn with_reports_channel(mut self, channel: String) -> Self {
        self.reports_channel_id = Some(channel);
        self
    }

    pub fn with_register_channel(mut self, channel: String) -> Self {
        self.register_channel_id = Some(channel);
        self
    }
}

impl From<GetTournament> for get_tournament_query::Variables {
    fn from(value: GetTournament) -> Self {
        get_tournament_query::Variables {
            id: value.id,
            reports_channel_id: value.reports_channel_id,
            register_channel_id: value.register_channel_id,
        }
    }
}

#[derive(Debug, Default)]
pub struct GetUser {
    pub id: Option<Uuid>,
    pub discord_id: Option<String>,
}

impl GetUser {
    pub fn with_id(mut self, id: Uuid) -> Self {
        self.id = Some(id);
        self
    }

    pub fn with_discord_id(mut self, discord: String) -> Self {
        self.discord_id = Some(discord);
        self
    }
}

impl From<GetUser> for get_user_query::Variables {
    fn from(value: GetUser) -> Self {
        get_user_query::Variables {
            id: value.id,
            discord_id: value.discord_id,
        }
    }
}
