use async_graphql::Context;
use sea_orm::{error, DatabaseConnection};
use uuid::Uuid;

use crate::{prelude::{ModType, TournamentService}, routes::tournament, services::tournament::models::{game_builder::GameModel, hero::HeroModel, heroes::HeroesModel, match_structure::MatchModel, operator::TournamentOperatorModel, organizer::OrganizerModel, participant, tournament::TournamentModel, tournament_builder::TournamentBuilderModel, user::UserModel}};

pub struct Query;

#[async_graphql::Object]
impl Query {
    async fn operator<'a>(
        &self, 
        context: &Context<'a>,
        id: Option<Uuid>,
        server_id: Option<i64>
    ) -> Result<Option<TournamentOperatorModel>, String> {
        let service = context.data::<TournamentService>().unwrap();
        let db = context.data::<DatabaseConnection>().unwrap();
        match service.get_operator(db, id, server_id).await {
            Ok(operator) => {
                Ok(operator)
            },
            Err(error) => {
                Err(error.to_string())
            }
        }
    }

    async fn tournament<'a>(
        &self,
        context: &Context<'a>,
        #[graphql(desc = "Id of tournament")]
        id: Option<Uuid>,
        #[graphql(desc = "Unique reports channel of tournament")]
        reports_channel_id: Option<String>,
        #[graphql(desc = "Unique register channel of tournament")]
        register_channel_id: Option<String>
    ) -> Result<Option<TournamentModel>, String> {
        let service = context.data::<TournamentService>().unwrap();
        let db = context.data::<DatabaseConnection>().unwrap();
        let res = service.get_tournament(db, id, reports_channel_id, register_channel_id).await;

        match res {
            Ok(tournament) => {
                Ok(tournament)
            },
            Err(error) => {
                Err(error)
            }
        }
    }

    async fn tournaments_all<'a>(
        &self,
        context: &Context<'a>
    ) -> Result<Vec<TournamentModel>, String> {
        let service = context.data::<TournamentService>().unwrap();
        let db = context.data::<DatabaseConnection>().unwrap();
        let res = service.get_tournaments(db).await;

        match res {
            Ok(tournaments) => {
                Ok(tournaments)
            },
            Err(error) => {
                Err(error.to_string())
            }
        }
    }

    async fn user<'a>(
        &self,
        context: &Context<'a>,
        #[graphql(desc = "User's id")]
        id: Option<Uuid>,
        #[graphql(desc = "User's discord id")]
        discord_id: Option<String>
    ) -> Result<Option<UserModel>, String> {
        let service = context.data::<TournamentService>().unwrap();
        let db = context.data::<DatabaseConnection>().unwrap();
        let res = service.get_user(db, id, discord_id).await;

        match res {
            Ok(user) => {
                Ok(user)
            },
            Err(error) => {
                Err(error)
            }
        }
    }

    async fn get_match<'a>(
        &self,
        context: &Context<'a>,
        id: Uuid
    ) -> Result<Option<MatchModel>, String> {
        let service = context.data::<TournamentService>().unwrap();
        let db = context.data::<DatabaseConnection>().unwrap();
        let res = service.get_match(db, id).await;

        match res {
            Ok(model) => {
                Ok(model)
            },
            Err(error) => {
                Err(error)
            }
        }
    }

    async fn matches<'a>(
        &self,
        context: &Context<'a>,
        tournament_id: Uuid,
        user_id: Option<Uuid>
    ) -> Result<Vec<MatchModel>, String> {
        let service = context.data::<TournamentService>().unwrap();
        let db = context.data::<DatabaseConnection>().unwrap();
        let res = service.get_matches(db, tournament_id, user_id).await;
        match res {
            Ok(matches) => {
                Ok(matches)
            },
            Err(error) => {
                Err(error.to_string())
            }
        }
    }

    async fn users<'a>(
        &self,
        context: &Context<'a>,
        tournament_id: Uuid
    ) -> Result<Vec<UserModel>, String> {
        let service = context.data::<TournamentService>().unwrap();
        let db = context.data::<DatabaseConnection>().unwrap();
        let res = service.get_users(db, tournament_id).await;
        
        match res {
            Ok(users) => {
                Ok(users)
            },
            Err(error) => {
                Err(error.to_string())
            }
        }
    }

    // async fn game<'a>(
    //     &self,
    //     context: &Context<'a>,
    //     match_id: Uuid,
    //     number: i32
    // ) -> Result<Option<GameBuilderModel>, String> {
    //     let service = context.data::<TournamentService>().unwrap();
    //     let db = context.data::<DatabaseConnection>().unwrap();
    //     let res = service.get_game(db, match_id, number).await;
        
    //     match res {
    //         Ok(game) => {
    //             Ok(game)
    //         },
    //         Err(error) => {
    //             Err(error)
    //         }
    //     }
    // }

    async fn heroes<'a>(
        &self,
        context: &Context<'a>,
        race: i32
    ) -> Result<Vec<HeroModel>, String> {
        let service = context.data::<TournamentService>().unwrap();
        let db = context.data::<DatabaseConnection>().unwrap();
        let res = service.get_heroes(db, race).await;

        match res {
            Ok(heroes) => {
                Ok(heroes)
            },
            Err(error) => {
                Err(error)
            }
        }
    }

    async fn hero<'a>(
        &self,
        context: &Context<'a>,
        id: i32
    ) -> Result<Option<HeroModel>, String> {
        let service = context.data::<TournamentService>().unwrap();
        let db = context.data::<DatabaseConnection>().unwrap();
        let res = service.get_hero(db, id).await;

        match res {
            Ok(hero) => {
                Ok(hero)
            },
            Err(error) => {
                Err(error)
            }
        }
    }

    async fn games<'a>(
        &self,
        context: &Context<'a>,
        match_id: Uuid
    ) -> Result<Vec<GameModel>, String> {
        let service = context.data::<TournamentService>().unwrap();
        let db = context.data::<DatabaseConnection>().unwrap();
        let res = service.get_games(db, match_id).await;

        match res {
            Ok(games) => {
                Ok(games)
            },
            Err(error) => {
                Err(error.to_string())
            }
        }
    }

    async fn participants<'a>(
        &self,
        context: &Context<'a>,
        tournament_id: Uuid,
        group: i32
    ) -> Result<Vec<UserModel>, String> {
        let service = context.data::<TournamentService>().unwrap();
        let db = context.data::<DatabaseConnection>().unwrap();
        let res = service.get_participants(db, tournament_id, group).await;

        match res {
            Ok(users) => {
                Ok(users)
            },
            Err(error) => {
                Err(error)
            }
        } 
    }

    async fn participant<'a>(
        &self,
        context: &Context<'a>,
        tournament_id: Option<Uuid>,
        user_id: Option<Uuid>,
        challonge: Option<String>
    ) -> Result<Option<participant::Model>, String> {
        let service = context.data::<TournamentService>().unwrap();
        let db = context.data::<DatabaseConnection>().unwrap();
        let res = service.get_participant(db, user_id, tournament_id, challonge).await;

        match res {
            Ok(user) => {
                Ok(user)
            },
            Err(error) => {
                Err(error)
            }
        } 
    }

    async fn organizer<'a>(
        &self,
        context: &Context<'a>,
        id: Option<Uuid>,
        discord_id: Option<i64>,
        challonge_key: Option<String> 
    ) -> Result<Option<OrganizerModel>, String> {
        let service = context.data::<TournamentService>().unwrap();
        let db = context.data::<DatabaseConnection>().unwrap();
        let res = service.get_organizer(db, id, discord_id, challonge_key).await;

        match res {
            Ok(model) => {
                Ok(model)
            },
            Err(error) => {
                Err(error)
            }
        } 
    }

    async fn tournament_builder<'a>(
        &self,
        context: &Context<'a>,
        id: Option<Uuid>,
        message: Option<i64>
    ) -> Result<Option<TournamentBuilderModel>, String> {
        let service = context.data::<TournamentService>().unwrap();
        let db = context.data::<DatabaseConnection>().unwrap();
        let res = service.get_tournament_builder(db, id, message).await;

        match res {
            Ok(model) => {
                Ok(model)
            },
            Err(error) => {
                Err(error)
            }
        } 
    }

    async fn tournaments<'a>(
        &self,
        context: &Context<'a>,
        organizer_id: Uuid
    ) -> Result<Vec<TournamentModel>, String> {
        let service = context.data::<TournamentService>().unwrap();
        let db = context.data::<DatabaseConnection>().unwrap();
        let res = service.get_tournaments_by_organizer(db, organizer_id).await;

        match res {
            Ok(models) => {
                Ok(models)
            },
            Err(error) => {
                Err(error)
            }
        } 
    }

    async fn tournament_users<'a>(
        &self,
        context: &Context<'a>,
        tournament_id: Uuid
    ) -> Result<Vec<UserModel>, String> {
        let service = context.data::<TournamentService>().unwrap();
        let db = context.data::<DatabaseConnection>().unwrap();
        let res = service.get_users_by_tournament(db, tournament_id).await;

        match res {
            Ok(models) => {
                Ok(models)
            },
            Err(error) => {
                Err(error)
            }
        } 
    }

    async fn games_count<'a>(
        &self,
        context: &Context<'a>,
        match_id: Uuid
    ) -> Result<u64, String> {
        let service = context.data::<TournamentService>().unwrap();
        let db = context.data::<DatabaseConnection>().unwrap();
        let res = service.get_games_count(db, match_id).await;

        match res {
            Ok(res) => {
                Ok(res)
            },
            Err(error) => {
                Err(error.to_string())
            }
        }
    }

    async fn heroes_new<'a>(
        &self,
        context: &Context<'a>,
        mod_type: ModType
    ) -> Result<HeroesModel, String> {
        let service = context.data::<TournamentService>().unwrap();
        let db = context.data::<DatabaseConnection>().unwrap();
        match service.get_heroes_new(db, mod_type).await {
            Ok(heroes) => {
                Ok(heroes)
            },
            Err(db_error) => {
                Err(db_error.to_string())
            }
        }
    }

    async fn games_all<'a>(
        &self,
        context: &Context<'a>,
        tournament_id: Uuid
    ) -> Result<Vec<GameModel>, String> {
        let service = context.data::<TournamentService>().unwrap();
        let db = context.data::<DatabaseConnection>().unwrap();
        match service.get_all_games(db, tournament_id).await {
            Ok(games) => {
                Ok(games)
            },
            Err(db_error) => {
                Err(db_error.to_string())
            }
        }
    }
}