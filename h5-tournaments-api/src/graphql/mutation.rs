use async_graphql::Context;
use sea_orm::DatabaseConnection;
use uuid::Uuid;

use crate::{prelude::TournamentService, services::tournament::models::{game_builder::{GameBuilderModel, GameEditState, GameResult}, organizer::OrganizerModel, tournament, tournament_builder::{TournamentBuilderModel, TournamentEditState}}};

pub struct Mutation;

#[async_graphql::Object]
impl Mutation {
    async fn create_user<'a>(
        &self,
        context: &Context<'a>,
        #[graphql(desc = "User's nickname")]
        name: String,
        #[graphql(desc = "User's discord id")]
        discord: String,
        #[graphql(desc = "Defines was user registered themselves or via bot command")]
        confirm_register: bool
    ) -> Result<Uuid, String> {
        let service = context.data::<TournamentService>().unwrap();
        let db = context.data::<DatabaseConnection>().unwrap();
        let res = service.create_user(db, name, discord, confirm_register).await;
        match res {
            Ok(res) => {
                Ok(res)
            },
            Err(error) => {
                Err(error)
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
        operator_id: Option<Uuid>,
        channel_id: String,
        register_channel: String,
        bargains: bool,
        bargains_color: bool,
        foreign_heroes: bool,
        role: String,
        organizer: Uuid
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
            organizer
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
        interaction: String,
        first_player: Uuid
    ) -> Result<String, String> {
        let service = context.data::<TournamentService>().unwrap();
        let db = context.data::<DatabaseConnection>().unwrap();
        let res = service.create_match(db, tournament_id, interaction, first_player).await;
        match res {
            Ok(_res) => {
                Ok("Match created".to_string())
            },
            Err(error) => {
                Err(error)
            }
        }
    }

    async fn update_match<'a>(
        &self,
        context: &Context<'a>,
        id: Uuid,
        games_count: Option<i32>,
        second_player: Option<Uuid>,
        data_message: Option<String>,
        current_game: Option<i32>
    ) -> Result<String, String> {
        let service = context.data::<TournamentService>().unwrap();
        let db = context.data::<DatabaseConnection>().unwrap();
        let res = service.update_match(db, id, games_count, second_player, data_message, current_game).await;
        match res {
            Ok(_res) => {
                Ok("Match updated".to_string())
            },
            Err(error) => {
                Err(error)
            }
        }
    }

    async fn create_game<'a>(
        &self,
        context: &Context<'a>,
        match_id: Uuid,
        number: i16
    ) -> Result<GameBuilderModel, String> {
        let service = context.data::<TournamentService>().unwrap();
        let db = context.data::<DatabaseConnection>().unwrap();
        let res = service.create_game(db, match_id, number).await;
        match res {
            Ok(_res) => {
                Ok(_res)
            },
            Err(error) => {
                Err(error)
            }
        }
    }

    async fn update_game<'a>(
        &self,
        context: &Context<'a>,
        match_id: Uuid,
        number: i32,
        edit_state: Option<GameEditState>,
        first_player_race: Option<i32>,
        first_player_hero: Option<i32>,
        second_player_race: Option<i32>,
        second_player_hero: Option<i32>,
        bargains_amount: Option<i32>,
        result: Option<GameResult>
    ) -> Result<String, String> {
        let service = context.data::<TournamentService>().unwrap();
        let db = context.data::<DatabaseConnection>().unwrap();
        let res = service.update_game(
            db, 
            match_id, 
            number, 
            edit_state, 
            first_player_race, 
            first_player_hero, 
            second_player_race, 
            second_player_hero, 
            bargains_amount,
            result
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

    async fn create_participant<'a>(
        &self,
        context: &Context<'a>,
        tournament_id: Uuid,
        user_id: Uuid,
        group: i32,
        challonge_id: Option<String>
    ) -> Result<i64, String> {
        let service = context.data::<TournamentService>().unwrap();
        let db = context.data::<DatabaseConnection>().unwrap();
        let res = service.create_participant(db, tournament_id, user_id, group, challonge_id).await;

        match res {
            Ok(_res) => {
                Ok(_res)
            },
            Err(error) => {
                Err(error)
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
        user_id: Uuid
    ) -> Result<String, String> {
        let service = context.data::<TournamentService>().unwrap();
        let db = context.data::<DatabaseConnection>().unwrap();
        let res = service.delete_participant(db, tournament_id, user_id).await;

        match res {
            Ok(_res) => {
                Ok(_res)
            },
            Err(error) => {
                Err(error)
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
}