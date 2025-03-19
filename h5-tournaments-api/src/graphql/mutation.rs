use async_graphql::Context;
use sea_orm::DatabaseConnection;
use serde::{Serialize, Deserialize};
use uuid::Uuid;

use crate::{prelude::{ModType, TournamentService}, services::tournament::models::{game_builder::{BargainsColor, CreateGameModel, GameOutcome, GameResult}, organizer::OrganizerModel, tournament::{self, GameType}, tournament_builder::{TournamentBuilderModel, TournamentEditState}, user::{UserBulkUpdatePayload, UserModel}}};

pub struct Mutation;

#[derive(Debug, Serialize, Deserialize, async_graphql::InputObject)]
pub struct UpdateParticipant {
    pub user_id: Uuid,
    pub tournament_id: Uuid,
    pub challonge_id: String
}

#[async_graphql::Object]
impl Mutation {
    async fn create_user<'a>(
        &self,
        context: &Context<'a>,
        #[graphql(desc = "User's nickname")]
        name: String,
        #[graphql(desc = "User's discord id")]
        discord_id: u64,
        // #[graphql(desc = "Defines was user registered themselves or via bot command")]
        // confirm_register: bool
        #[graphql(desc = "User's discord nickname")]
        discord_nick: String
    ) -> Result<UserModel, String> {
        let service = context.data::<TournamentService>().unwrap();
        let db = context.data::<DatabaseConnection>().unwrap();
        match service.create_user(db, name, discord_id, discord_nick).await {
            Ok(model) => {
                Ok(model)
            },
            Err(db_error) => {
                Err(db_error.to_string())
            }
        }
    }

    async fn update_user<'a>(
        &self,
        context: &Context<'a>,
        id: Uuid,
        nickname: Option<String>,
        registered: Option<bool>
    ) -> Result<String, String> {
        let service = context.data::<TournamentService>().unwrap();
        let db = context.data::<DatabaseConnection>().unwrap();
        let res = service.update_user(db, id, nickname, registered).await;
        match res {
            Ok(_res) => {
                Ok("User updated successfully".to_string())
            },
            Err(error) => {
                Err(error)
            }
        }
    }
    
    async fn create_tournament<'a>(
        &self,
        context: &Context<'a>,
        name: String,
        operator_id: Uuid,
        channel_id: String,
        register_channel: String,
        bargains: bool,
        bargains_color: bool,
        foreign_heroes: bool,
        role: String,
        organizer: Uuid,
        game_type: GameType,
        mod_type: ModType
    ) -> Result<String, String> {
        let service = context.data::<TournamentService>().unwrap();
        let db = context.data::<DatabaseConnection>().unwrap();
        let res = service.create_tournament(
            db, 
            name, 
            operator_id, 
            channel_id, 
            register_channel, 
            bargains, 
            bargains_color, 
            foreign_heroes, 
            role,
            organizer,
            game_type,
            mod_type
        ).await;
        match res {
            Ok(res) => {
                Ok(res)
            },
            Err(error) => {
                Err(error)
            }
        }
    }

    async fn update_tournament<'a>(
        &self,
        context: &Context<'a>,
        id: Uuid,
        stage: Option<tournament::TournamentStage>,
        challonge_id: Option<String>
    ) -> Result<String, String> {
        let service = context.data::<TournamentService>().unwrap();
        let db = context.data::<DatabaseConnection>().unwrap();
        let res = service.update_tournament(db, id, stage, challonge_id).await;
        match res {
            Ok(_res) => {
                Ok("Tournament was updated.".to_string())
            },
            Err(error) => {
                Err(error)
            }
        }
    }

    async fn create_match<'a>(
        &self,
        context: &Context<'a>,
        tournament_id: Uuid,
        message: i64,
        first_player: Uuid,
        second_player: Uuid,
        challonge_id: String
    ) -> Result<Uuid, String> {
        let service = context.data::<TournamentService>().unwrap();
        let db = context.data::<DatabaseConnection>().unwrap();
        let res = service.create_match(db, tournament_id, message, first_player, second_player, challonge_id).await;
        match res {
            Ok(res) => {
                Ok(res)
            },
            Err(error) => {
                Err(error.to_string())
            }
        }
    }

    async fn update_match<'a>(
        &self,
        context: &Context<'a>,
        id: Uuid,
        report_link: String
    ) -> Result<String, String> {
        let service = context.data::<TournamentService>().unwrap();
        let db = context.data::<DatabaseConnection>().unwrap();
        let res = service.update_match(db, id, report_link).await;
        match res {
            Ok(_res) => {
                Ok("Match updated".to_string())
            },
            Err(error) => {
                Err(error.to_string())
            }
        }
    }

    // async fn create_game<'a>(
    //     &self,
    //     context: &Context<'a>,
    //     match_id: Uuid,
    //     number: i16
    // ) -> Result<GameBuilderModel, String> {
    //     let service = context.data::<TournamentService>().unwrap();
    //     let db = context.data::<DatabaseConnection>().unwrap();
    //     let res = service.create_game(db, match_id, number).await;
    //     match res {
    //         Ok(_res) => {
    //             Ok(_res)
    //         },
    //         Err(error) => {
    //             Err(error)
    //         }
    //     }
    // }

    async fn update_game<'a>(
        &self,
        context: &Context<'a>,
        match_id: Uuid,
        first_player_race: Option<i32>,
        first_player_hero: Option<i32>,
        second_player_race: Option<i32>,
        second_player_hero: Option<i32>,
        bargains_color: Option<BargainsColor>,
        bargains_amount: Option<i32>,
        result: Option<GameResult>,
        outcome: Option<GameOutcome>
    ) -> Result<String, String> {
        let service = context.data::<TournamentService>().unwrap();
        let db = context.data::<DatabaseConnection>().unwrap();
        let res = service.update_game(
            db, 
            match_id, 
            first_player_race, 
            first_player_hero, 
            second_player_race, 
            second_player_hero, 
            bargains_color,
            bargains_amount,
            result,
            outcome
        ).await;
        match res {
            Ok(_res) => {
                Ok(_res)
            },
            Err(error) => {
                Err(error)
            }
        }
    }

    async fn create_games_bulk<'a>(
        &self,
        context: &Context<'a>,
        games: Vec<CreateGameModel>
    ) -> Result<String, String> {
        let service = context.data::<TournamentService>().unwrap();
        let db = context.data::<DatabaseConnection>().unwrap();
        let res = service.create_games_bulk(db, games).await;

        match res {
            Ok(_res) => {
                Ok("Games bulk inserted ok".to_string())
            },
            Err(error) => {
                Err(error)
            }
        }
    }

    async fn create_participant<'a>(
        &self,
        context: &Context<'a>,
        tournament_id: Uuid,
        user_id: Uuid,
        challonge_id: String
    ) -> Result<u64, String> {
        let service = context.data::<TournamentService>().unwrap();
        let db = context.data::<DatabaseConnection>().unwrap();
        match service.create_participant(db, tournament_id, user_id, challonge_id).await {
            Ok(participants_count) => {
                Ok(participants_count)
            },
            Err(db_error) => {
                Err(db_error.to_string())
            }
        }
    }

    async fn update_participant<'a>(
        &self,
        context: &Context<'a>,
        id: Uuid,
        group: Option<i32>,
        challonge_id: Option<String>
    ) -> Result<String, String> {
        let service = context.data::<TournamentService>().unwrap();
        let db = context.data::<DatabaseConnection>().unwrap();
        let res = service.update_participant(db, id, group, challonge_id).await;

        match res {
            Ok(_res) => {
                Ok("Participant updated".to_string())
            },
            Err(error) => {
                Err(error)
            }
        }
    }

    async fn delete_participant<'a>(
        &self,
        context: &Context<'a>,
        tournament_id: Uuid,
        id: Option<Uuid>,
        user_id: Option<Uuid>,
        challonge_id: Option<String>
    ) -> Result<u64, String> {
        let service = context.data::<TournamentService>().unwrap();
        let db = context.data::<DatabaseConnection>().unwrap();
        let res = service.delete_participant(db, tournament_id, id, user_id, challonge_id).await;

        match res {
            Ok(res) => {
                Ok(res)
            },
            Err(error) => {
                Err(error.to_string())
            }
        }
    }

    async fn create_tournament_builder<'a>(
        &self,
        context: &Context<'a>,
        message_id: String
    ) -> Result<TournamentBuilderModel, String> {
        let service = context.data::<TournamentService>().unwrap();
        let db = context.data::<DatabaseConnection>().unwrap();
        let res = service.create_tournament_builder(db, message_id).await;

        match res {
            Ok(_res) => {
                Ok(_res)
            },
            Err(error) => {
                Err(error)
            }
        }
    }

    async fn create_organizer<'a>(
        &self,
        context: &Context<'a>,
        discord_id: String,
        challonge_key: String
    ) -> Result<Uuid, String> {
        let service = context.data::<TournamentService>().unwrap();
        let db = context.data::<DatabaseConnection>().unwrap();
        let res = service.create_organizer(db, discord_id, challonge_key).await;

        match res {
            Ok(_res) => {
                Ok(_res)
            },
            Err(error) => {
                Err(error)
            }
        }
    }

    async fn update_tournament_builder<'a>(
        &self,
        context: &Context<'a>,
        id: Uuid,
        name: Option<String>,
        state: Option<TournamentEditState>,
        register_channel: Option<String>,
        reports_channel: Option<String>,
        role: Option<String>,
        use_bargains: Option<bool>,
        use_bargains_color: Option<bool>,
        use_foreign_heroes: Option<bool>
    ) -> Result<TournamentBuilderModel, String> {
        let service = context.data::<TournamentService>().unwrap();
        let db = context.data::<DatabaseConnection>().unwrap();
        let res = service.update_tournament_builder(
            db,
            id,
            name,
            state,
            register_channel,
            reports_channel,
            role,
            use_bargains,
            use_bargains_color,
            use_foreign_heroes
        ).await;

        match res {
            Ok(_res) => {
                Ok(_res)
            },
            Err(error) => {
                Err(error)
            }
        }
    }

    async fn update_participants_bulk<'a>(
        &self,
        context: &Context<'a>,
        participants: Vec<UpdateParticipant>
    ) -> Result<String, String> {
        let service = context.data::<TournamentService>().unwrap();
        let db = context.data::<DatabaseConnection>().unwrap();
        let res = service.participants_bulk_update(db, participants).await;

        match res {
            Ok(_res) => {
                Ok("Participants were updated successfully".to_string())
            },
            Err(error) => {
                Err(error)
            }
        }
    }

    async fn update_users_bulk<'a>(
        &self,
        context: &Context<'a>,
        users: Vec<UserBulkUpdatePayload>
    ) -> Result<String, String> {
        let service = context.data::<TournamentService>().unwrap();
        let db = context.data::<DatabaseConnection>().unwrap();
        let res = service.users_bulk_update(db, users).await;

        match res {
            Ok(_res) => {
                Ok("Users were updated successfully".to_string())
            },
            Err(error) => {
                Err(error)
            }
        }
    }
}