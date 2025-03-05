use rust_decimal::Decimal;
use sea_orm::{sea_query::{expr, OnConflict, SimpleExpr}, ActiveModelTrait, ColumnTrait, Condition, DatabaseConnection, DbErr, EntityTrait, ModelTrait, PaginatorTrait, QueryFilter, Related, Set, TransactionTrait};
use sqlx::{PgPool, Pool, Postgres};
use uuid::Uuid;

use crate::{graphql::mutation::UpdateParticipant, routes::models::MatchRegistrationForm};

use self::{game_builder::GameResult, match_structure::MatchModel, tournament::TournamentModel, user::{Column, Entity, UserModel}};

use super::{models::{game_builder::{self, CreateGameModel, GameModel, GameOutcome}, hero::{self, HeroModel}, heroes::{self, HeroesModel}, match_structure, operator::{self, TournamentOperatorModel}, organizer::{self, OrganizerModel}, participant, tournament::{self, GameType}, tournament_builder::{self, TournamentBuilderModel, TournamentEditState}, user::{self, UserBulkUpdatePayload}}, types::{Game, Hero, Match, ModType, Race, Tournament}};

#[derive(Clone)]
pub struct LegacyTournamentService {
    pub pool: PgPool
}

impl LegacyTournamentService {
    pub fn new(pool: PgPool) -> Self {
        LegacyTournamentService { pool: pool }
    }

    pub async fn create_tournament(
        &self, 
        mod_type: i32,
        server_id: i64, 
        channel_id: i64, 
        start_message_id: i64,
        last_message_id: i64,
        name: String
    ) -> Result<String, super::error::Error> {

        let existing_tournament: Option<(Uuid, )> = sqlx::query_as(r#"
                SELECT id FROM tournaments 
                WHERE channel_id=$1 AND first_message_id=$2 AND last_message_id=$3;
            "#)
            .bind(channel_id as i64)
            .bind(start_message_id as i64)
            .bind(last_message_id as i64)
            .fetch_optional(&self.pool)
            .await?;

        if let Some(tournament) = existing_tournament {
            Ok(format!("Tournament already exists with id {}", tournament.0))
        }
        else {
            let id = Uuid::new_v4();
            let _res = sqlx::query(r#"
                    INSERT INTO tournaments 
                    (id, mod_type, server_id, channel_id, first_message_id, last_message_id, name)
                    VALUES ($1, $2, $3, $4, $5, $6, $7);
                "#)
                .bind(id)
                .bind(mod_type as i32)
                .bind(server_id as i64)
                .bind(channel_id as i64)
                .bind(start_message_id as i64)
                .bind(last_message_id as i64)
                .bind(name)
                .execute(&self.pool)
                .await?;
            
            Ok(format!("Tournament was created with id {}", id))
        }
    }

    pub async fn get_tournament_by_id(&self, id: Uuid) -> Result<Tournament, super::error::Error> {
        let tournament: Result<Tournament, sqlx::Error> = sqlx::query_as(r#"
                SELECT * FROM tournaments WHERE id=$1;
            "#)
            .bind(id)
            .fetch_one(&self.pool)
            .await;

        match tournament {
            Ok(tournament) => {
                Ok(tournament)
            },
            Err(error) => {
                tracing::error!("Sqlx: failed to fetch tournament with id {}: {}", id, error.to_string());
                Err(super::error::Error::SqlxError(error))
            }
        }

        //Ok(tournament)
    }

    pub async fn load_races(&self) -> Result<Vec<Race>, super::error::Error> {
        let races_data: Vec<Race> = sqlx::query_as(r#"
                SELECT * FROM races;
            "#)
            .fetch_all(&self.pool)
            .await?;

        Ok(races_data)
    }

    pub async fn load_heroes_for_mod(&self, mod_type: ModType) -> Result<Vec<Hero>, super::error::Error> {
        let heroes_data: Result<Vec<Hero>, sqlx::Error> = sqlx::query_as(r#"
                SELECT * FROM heroes WHERE mod_type=0 OR mod_type=$1;
            "#)
            .bind(mod_type)
            .fetch_all(&self.pool)
            .await;

        match heroes_data {
            Ok(heroes_data) => {
                Ok(heroes_data)
            },
            Err(error) => {
                tracing::error!("Sqlx: failed to fetch heroes: {}", error.to_string());
                Err(super::error::Error::SqlxError(error))
            }
        }
    }

    pub async fn register_match(&self, match_data: &MatchRegistrationForm) -> Result<Uuid, super::error::Error> {
        let id = Uuid::new_v4();
        let res = sqlx::query(r#"
                INSERT INTO matches
                (id, tournament_id, first_player, second_player, message_id)
                VALUES ($1, $2, $3, $4, $5)
            "#)
            .bind(id)
            .bind(match_data.tournament_id)
            .bind(&match_data.first_player)
            .bind(&match_data.second_player)
            .bind(match_data.message_id)
            .execute(&self.pool)
            .await;

        match res {
            Ok(_) => {
                Ok(id)
            },
            Err(error) => {
                tracing::error!("Failed to insert match: {}", error.to_string());
                Err(super::error::Error::SqlxError(error))
            }
        }
    }

    pub async fn upload_games(&self, games_data: &Vec<Game>) -> Result<(), super::error::Error> {
        let mut transaction = self.pool.begin().await?;

        for game in games_data {
            let res = sqlx::query(r#"
                    INSERT INTO games
                    (id, match_id, first_player_race, first_player_hero, second_player_race, second_player_hero, bargains_color, bargains_amount, result)
                    VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9)
                "#)
                .bind(game.id)
                .bind(game.match_id)
                .bind(game.first_player_race)
                .bind(game.first_player_hero)
                .bind(game.second_player_race)
                .bind(game.second_player_hero)
                .bind(game.bargains_color)
                .bind(game.bargains_amount)
                .bind(&game.result)
                .execute(&mut *transaction)
                .await;
            match res {
                Ok(_) => {},
                Err(error) => {
                    tracing::error!("Failed to insert game: {}", error.to_string());
                }
            }
        }

        transaction.commit().await?;

        Ok(())
    }

    pub async fn load_existing_tournaments(&self) -> Result<Vec<Tournament>, super::error::Error> {
        let tournaments = sqlx::query_as(r#"
                SELECT * FROM tournaments;
            "#)
            .fetch_all(&self.pool)
            .await?;

        Ok(tournaments)
    }

    pub async fn load_matches_for_tournament(&self, tournament_id: Uuid) -> Result<Vec<Match>, super::error::Error> {
        let matches = sqlx::query_as(r#"
                SELECT * FROM matches WHERE tournament_id=$1;
            "#)
            .bind(tournament_id)
            .fetch_all(&self.pool)
            .await?;

        Ok(matches)
    }

    pub async fn load_games_for_match(&self, match_id: Uuid) -> Result<Vec<Game>, super::error::Error> {
        let games = sqlx::query_as(r#"
                SELECT * FROM games WHERE match_id=$1;
            "#)
            .bind(match_id)
            .fetch_all(&self.pool)
            .await?;

        Ok(games)
    }

    pub async fn create_game(&self, game: Game) -> Result<(), super::error::Error> {
        let res = sqlx::query(r#"
                INSERT INTO games
                (id, match_id, first_player_race, first_player_hero, second_player_race, second_player_hero, bargains_color, bargains_amount, result)
                VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9)
            "#)
            .bind(game.id)
            .bind(game.match_id)
            .bind(game.first_player_race)
            .bind(game.first_player_hero)
            .bind(game.second_player_race)
            .bind(game.second_player_hero)
            .bind(game.bargains_color)
            .bind(game.bargains_amount)
            .bind(&game.result)
            .execute(&self.pool)
            .await?;

        Ok(())
    }

    pub async fn update_game(&self, game: Game) -> Result<(), super::error::Error> {
        let _res: Game = sqlx::query_as(
        r#"
                UPDATE games
                SET first_player_race=$1, first_player_hero=$2, second_player_race=$3, second_player_hero=$4, bargains_color=$5, bargains_amount=$6, result=$7
                WHERE id=$8
                RETURNING *;
            "#)
            .bind(&game.first_player_race)
            .bind(&game.first_player_hero)
            .bind(&game.second_player_race)
            .bind(&game.second_player_hero)
            .bind(&game.bargains_color)
            .bind(&game.bargains_amount)
            .bind(&game.result)
            .bind(&game.id)
            .fetch_one(&self.pool)
            .await?;

        Ok(())
    }

    pub async fn update_match(&self, match_to_update: Match) -> Result<(), super::error::Error> {
        let _res: Match = sqlx::query_as(r#"
                UPDATE matches
                SET first_player=$1, second_player=$2
                WHERE id=$3
                RETURNING *;
            "#)
            .bind(&match_to_update.first_player)
            .bind(&match_to_update.second_player)
            .bind(&match_to_update.id)
            .fetch_one(&self.pool)
            .await?;

        Ok(())
    }

    pub async fn get_all_games_for_tournament(&self, tournament_id: Uuid) -> Result<Vec<Game>, super::error::Error> {
        let games = sqlx::query_as(r#"
                SELECT * FROM games 
                INNER JOIN matches
                ON (games.match_id = matches.id AND matches.tournament_id = $1);         
            "#)
            .bind(tournament_id)
            .fetch_all(&self.pool)
            .await?;

        Ok(games)
    }
}

#[derive(Clone)]
pub struct TournamentService;

impl TournamentService {
    pub async fn create_user(
        &self,
        db: &DatabaseConnection,
        name: String,
        discord_id: u64,
        //confirm_register: bool
        discord_nick: String
    ) -> Result<UserModel, DbErr> {
        let id = Uuid::new_v4();
        let on_conflict = OnConflict::column(Column::DiscordId)
            .update_column(Column::Nickname)
            .value(Column::RegisteredManually, true)
            .to_owned();

        let user_to_insert = user::ActiveModel {
            id: Set(id),
            nickname: Set(name.clone()),
            discord_id: Set(discord_id as i64),
            registered_manually: Set(true),
            discord_nick: Set(discord_nick)
        };

        let model = Entity::insert(user_to_insert).on_conflict(on_conflict.clone()).exec_with_returning(db).await?;
        Ok(model)
    }

    pub async fn update_user(
        &self,
        db: &DatabaseConnection,
        id: Uuid,
        nickname: Option<String>,
        registered: Option<bool>
    ) -> Result<(), String> {
        let current_user = user::Entity::find_by_id(id).one(db).await.unwrap();
        if let Some(current_user) = current_user {

            let mut user_to_update: user::ActiveModel = current_user.into();

            if let Some(nickname) = nickname {
                user_to_update.nickname = Set(nickname);
            }

            if let Some(registered) = registered {
                user_to_update.registered_manually = Set(registered);
            }

            user_to_update.update(db).await.unwrap();
        }

        Ok(())
    }

    pub async fn get_operator(
        &self,
        db: &DatabaseConnection,
        id: Option<Uuid>,
        server_id: Option<i64>
    ) -> Result<Option<TournamentOperatorModel>, DbErr> {
        let conditions = Condition::all()
            .add_option(if id.is_some() {
                Some(expr::Expr::col(operator::Column::Id).eq(id.unwrap()))
            } else {
                None::<SimpleExpr>
            })
            .add_option(if server_id.is_some() {
                Some(expr::Expr::col(operator::Column::ServerId).eq(server_id.unwrap()))
            } else {
                None::<SimpleExpr>
            });

        match operator::Entity::find().filter(conditions).one(db).await {
            Ok(operator) => {
                Ok(operator)
            },
            Err(error) => {
                Err(error)
            }
        }
    }

    pub async fn create_tournament(
        &self, db: &DatabaseConnection, 
        name: String, 
        operator_id: Uuid, 
        reports_channel_id: String,
        register_channel_id: String,
        use_bargains: bool,
        use_bargains_color: bool,
        use_foreign_heroes: bool,
        role_id: String,
        organizer: Uuid,
        game_type: GameType,
        mod_type: ModType
    ) -> Result<String, String> {
        let id = Uuid::new_v4();
        let channel_id = i64::from_str_radix(&reports_channel_id, 10).unwrap();
        let register_channel = i64::from_str_radix(&register_channel_id, 10).unwrap();
        let role = i64::from_str_radix(&role_id, 10).unwrap();
        let tournament_to_insert = tournament::ActiveModel {
            id: Set(id),
            operator_id: Set(operator_id),
            channel_id: Set(channel_id),
            name: Set(name.clone()),
            stage: Set(Some(tournament::TournamentStage::Unknown)),
            register_channel: Set(register_channel),
            with_bargains: Set(use_bargains),
            with_bargains_color: Set(use_bargains_color),
            with_foreign_heroes: Set(use_foreign_heroes),
            role_id: Set(role),
            challonge_id: Set(None),
            organizer: Set(organizer),
            game_type: Set(game_type),
            mod_type: Set(mod_type)
        };

        let res = tournament_to_insert.insert(db).await;

        match res {
            Ok(_model) => {
                Ok(format!("Tournament {} created with id {}", &name, &id))
            },
            Err(error) => {
                Err(error.to_string())
            }
        }
    }

    pub async fn update_tournament(
        &self,
        db: &DatabaseConnection,
        id: Uuid,
        stage: Option<tournament::TournamentStage>,
        challonge_id: Option<String>
    ) -> Result<(), String> {
        let current_tournament = tournament::Entity::find_by_id(id).one(db).await.unwrap();
        if let Some(current_tournament) = current_tournament {

            let mut tournament_to_update: tournament::ActiveModel = current_tournament.into();

            if let Some(stage) = stage {
                tournament_to_update.stage = Set(Some(stage));
            }

            if let Some(challonge_id) = challonge_id {
                tournament_to_update.challonge_id = Set(Some(challonge_id));
            }

            tournament_to_update.update(db).await.unwrap();
        }

        Ok(())
    }

    pub async fn get_tournament(
        &self,
        db: &DatabaseConnection,
        id: Option<Uuid>,
        reports_channel_id: Option<String>,
        register_channel_id: Option<String>
    ) -> Result<Option<TournamentModel>, String> {
        let conditions = Condition::all()
            .add_option( if id.is_some() { 
                Some(expr::Expr::col(tournament::Column::Id).eq(id.unwrap())) 
            } else { 
                None::<SimpleExpr> 
            })
            .add_option( if reports_channel_id.is_some() { 
                Some(expr::Expr::col(tournament::Column::ChannelId).eq(i64::from_str_radix(&reports_channel_id.unwrap(), 10).unwrap()))
            } else {
                None::<SimpleExpr>
            })
            .add_option( if register_channel_id.is_some() {
                Some(expr::Expr::col(tournament::Column::RegisterChannel).eq(i64::from_str_radix(&register_channel_id.unwrap(), 10).unwrap()))
            } else {
                None::<SimpleExpr>
            }
        );

        let res = tournament::Entity::find()
            .filter(conditions)
            .one(db)
            .await;

        match res {
            Ok(tournament) => {
                Ok(tournament)
            },
            Err(error) => {
                Err(error.to_string())
            }
        }
    }

    pub async fn get_user(
        &self,
        db: &DatabaseConnection,
        id: Option<Uuid>,
        discord_id: Option<String>
    ) -> Result<Option<UserModel>, String> {

        let conditions = Condition::all()
            .add_option(if id.is_some() { 
                Some(expr::Expr::col(user::Column::Id).eq(id.unwrap()))
            } else {
                None::<SimpleExpr>
            })
            .add_option(if discord_id.is_some() {
                Some(expr::Expr::col(user::Column::DiscordId).eq(i64::from_str_radix(&discord_id.unwrap(), 10).unwrap()))
            } else {
                None::<SimpleExpr>
            });

        let res = user::Entity::find()
            .filter(conditions)
            .one(db)
            .await;

        match res {
            Ok(user) => {
                Ok(user)
            },
            Err(error) => {
                Err(error.to_string())
            }
        }
    }

    pub async fn create_match(
        &self,
        db: &DatabaseConnection,
        tournament_id: Uuid,
        message: i64,
        first_player: Uuid,
        second_player: Uuid,
        challonge_id: String
    ) -> Result<Uuid, DbErr> {
        let id = Uuid::new_v4();
        let on_conflict = OnConflict::column(match_structure::Column::ChallongeId).do_nothing().to_owned();
        let match_to_create = match_structure::ActiveModel {
            id: Set(id),
            tournament_id: Set(tournament_id),
            message_id: Set(message),
            first_player: Set(first_player),
            second_player: Set(second_player),
            challonge_id: Set(challonge_id),
            report_link: Set(None)
        };
        let insert_result = match_structure::Entity::insert(match_to_create)
            .on_conflict(on_conflict)
            .exec_with_returning(db)
            .await?;
        Ok(insert_result.id)
    }

    pub async fn update_match(
        &self,
        db: &DatabaseConnection,
        id: Uuid,
        report_link: String
    ) -> Result<(), DbErr> {
        if let Some(current_match) = match_structure::Entity::find_by_id(id).one(db).await? {
            let mut match_to_update: match_structure::ActiveModel = current_match.into();
            match_to_update.report_link = Set(Some(report_link));
            match_to_update.update(db).await?;
        }
        Ok(())
    }

    pub async fn get_match(
        &self,
        db: &DatabaseConnection,
        id: Uuid,
    ) -> Result<Option<MatchModel>, String> {
        let res = match_structure::Entity::find()
            .filter(match_structure::Column::Id.eq(id))
            .one(db)
            .await;

        match res {
            Ok(model) => {
                Ok(model)
            },
            Err(error) => {
                Err(error.to_string())
            }
        }
    }

    pub async fn get_users(
        &self, 
        db: &DatabaseConnection
    ) -> Result<Vec<UserModel>, String> {
        let res = user::Entity::find().all(db).await;
        match res {
            Ok(users) => {
                Ok(users)
            },
            Err(error) => {
                Err(error.to_string())
            }
        }
    }

    pub async fn create_games_bulk(
        &self,
        db: &DatabaseConnection,
        games: Vec<CreateGameModel>
    ) -> Result<(), String> {
        let transaction = db.begin().await.unwrap();
        for game in games {
            let id = Uuid::new_v4();
            let game_to_insert = game_builder::ActiveModel {
                id: Set(id),
                match_id: Set(game.match_id),
                first_player_race: Set(game.first_player_race),
                first_player_hero: Set(game.first_player_hero),
                second_player_race: Set(game.second_player_race),
                second_player_hero: Set(game.second_player_hero),
                result: Set(game.result),
                bargains_color: Set(game.bargains_color),
                bargains_amount: Set(game.bargains_amount),
                outcome: Set(if game.outcome.is_some() { game.outcome.unwrap() } else { GameOutcome::FinalBattleVictory })
            };
            game_to_insert.insert(db).await.unwrap();
        }
        let res = transaction.commit().await;
        match res {
            Ok(_res) => {
                Ok(())
            },
            Err(error) => {
                Err(error.to_string())
            }
        }
    }

    // pub async fn update_game(
    //     &self,
    //     db: &DatabaseConnection,
    //     match_id: Uuid,
    //     number: i32,
    //     edit_state: Option<GameEditState>,
    //     first_player_race: Option<i32>,
    //     first_player_hero: Option<i32>,
    //     second_player_race: Option<i32>,
    //     second_player_hero: Option<i32>,
    //     bargains_amount: Option<i32>,
    //     result: Option<GameResult>
    // ) -> Result<String, String> {
    //     let current_game = game_builder::Entity::find()
    //         .filter(
    //             Condition::all()
    //                 .add(game_builder::Column::MatchId.eq(match_id))
    //                 .add(game_builder::Column::Number.eq(number))
    //         )
    //         .one(db)
    //         .await.unwrap();

    //     if let Some(game) = current_game {
    //         let mut game_to_update: game_builder::ActiveModel = game.into();
    //         if let Some(edit_state) = edit_state {
    //             game_to_update.edit_state = Set(Some(edit_state));
    //         }
    //         if let Some(first_player_race) = first_player_race {
    //             game_to_update.first_player_race = Set(Some(first_player_race));
    //         }
    //         if let Some(first_player_hero) = first_player_hero {
    //             game_to_update.first_player_hero = Set(Some(first_player_hero));
    //         }
    //         if let Some(second_player_race) = second_player_race {
    //             game_to_update.second_player_race = Set(Some(second_player_race));
    //         }
    //         if let Some(second_player_hero) = second_player_hero {
    //             game_to_update.second_player_hero = Set(Some(second_player_hero));
    //         }
    //         if let Some(bargains_amount) = bargains_amount {
    //             game_to_update.bargains_amount = Set(bargains_amount);
    //         }
    //         if let Some(result) = result {
    //             game_to_update.result = Set(result);
    //         }

    //         let res = game_to_update.update(db).await;
    //         match res {
    //             Ok(_success) => {
    //                 Ok("Game updated successfully".to_string())
    //             },
    //             Err(error) => {
    //                 Err(error.to_string())
    //             }
    //         }
    //     }
    //     else {
    //         Err("Failed to find game".to_string())
    //     }
    // }

    // pub async fn get_game(
    //     &self,
    //     db: &DatabaseConnection,
    //     match_id: Uuid,
    //     number: i32
    // ) -> Result<Option<GameBuilderModel>, String> {
    //     let res = game_builder::Entity::find()
    //         .filter(
    //             Condition::all()
    //             .add(game_builder::Column::MatchId.eq(match_id))
    //             .add(game_builder::Column::Number.eq(number))
    //         )
    //         .one(db)
    //         .await;

    //     match res {
    //         Ok(game) => { 
    //             Ok(game)
    //         },
    //         Err(error) => {
    //             Err(error.to_string())
    //         }
    //     }
    // }

    pub async fn get_heroes(
        &self,
        db: &DatabaseConnection,
        race: i32
    ) -> Result<Vec<HeroModel>, String> {
        let res = hero::Entity::find()
            .filter(hero::Column::Race.eq(race))
            .all(db)
            .await;

        match res {
            Ok(heroes) => {
                Ok(heroes)
            },
            Err(error) => {
                Err(error.to_string())
            }
        }
    }

    pub async fn get_hero(
        &self,
        db: &DatabaseConnection,
        id: i32
    ) -> Result<Option<HeroModel>, String> {
        let res = hero::Entity::find()
            .filter(hero::Column::Id.eq(id))
            .one(db)
            .await;

        match res {
            Ok(hero) => {
                Ok(hero)
            },
            Err(error) => {
                Err(error.to_string())
            }
        }
    }

    // pub async fn get_games(
    //     &self,
    //     db: &DatabaseConnection,
    //     match_id: Uuid
    // ) -> Result<Vec<GameBuilderModel>, String> {
    //     let res = game_builder::Entity::find()
    //         .filter(game_builder::Column::MatchId.eq(match_id))
    //         .all(db)
    //         .await;

    //     match res {
    //         Ok(games) => {
    //             Ok(games)
    //         },
    //         Err(error) => {
    //             Err(error.to_string())
    //         }
    //     }
    // }

    pub async fn get_participants(
        &self,
        db: &DatabaseConnection,
        tournament_id: Uuid,
        group: i32 
    ) -> Result<Vec<UserModel>, String> {
        let res = participant::Entity::find_related()
            .filter(
                Condition::all()
                    .add(participant::Column::TournamentId.eq(tournament_id))
                    .add(participant::Column::GroupNumber.eq(group))
            )
            .all(db)
            .await;
            
        match res {
            Ok(users) => {
                Ok(users)
            },
            Err(error) => {
                Err(error.to_string())
            }
        }
    }

    pub async fn get_participant(
        &self,
        db: &DatabaseConnection,
        user_id: Option<Uuid>,
        tournament_id: Option<Uuid>,
        challonge_id: Option<String>
    ) -> Result<Option<participant::Model>, String> {
        let conditions = Condition::all()
            .add_option(if user_id.is_some() {
                Some(expr::Expr::col(participant::Column::UserId).eq(user_id.unwrap()))
            } else {
                None::<SimpleExpr>
            })
            .add_option(if tournament_id.is_some() {
                Some(expr::Expr::col(participant::Column::TournamentId).eq(tournament_id.unwrap()))
            } else {
                None::<SimpleExpr>
            })
            .add_option(if challonge_id.is_some() {
                Some(expr::Expr::col(participant::Column::ChallongeId).eq(challonge_id.unwrap()))
            } else {
                None::<SimpleExpr>
            });
        let res = participant::Entity::find()
            .filter(conditions)
            .one(db)
            .await;

        match res {
            Ok(participant) => {
                Ok(participant)
            },
            Err(error) => {
                Err(error.to_string())
            }
        }
    }

    pub async fn create_participant(
        &self,
        db: &DatabaseConnection,
        tournament_id: Uuid,
        user_id: Uuid,
        challonge_id: String
    ) -> Result<u64, DbErr> {
        let participant_to_insert = participant::ActiveModel {
            id: Set(Uuid::new_v4()),
            tournament_id: Set(tournament_id),
            user_id: Set(user_id),
            group_number: Set(0),
            challonge_id: Set(Some(challonge_id))
        };

        participant_to_insert.insert(db).await?;
        let count = participant::Entity::find()
            .filter(participant::Column::TournamentId.eq(tournament_id)).count(db).await?;
        Ok(count)
    }

    pub async fn update_participant(
        &self,
        db: &DatabaseConnection,
        id: Uuid,
        group: Option<i32>,
        challonge_id: Option<String>
    ) -> Result<(), String> {
        let current_participant = participant::Entity::find_by_id(id).one(db).await.unwrap();
        if let Some(current_participant) = current_participant {

            let mut participant_to_update: participant::ActiveModel = current_participant.into();

            if let Some(group) = group {
                participant_to_update.group_number = Set(group);
            }

            if let Some(challonge_id) = challonge_id {
                participant_to_update.challonge_id = Set(Some(challonge_id));
            }

            participant_to_update.update(db).await.unwrap();
        }

        Ok(())
    }

    pub async fn delete_participant(
        &self,
        db: &DatabaseConnection,
        tournament_id: Uuid,
        id: Option<Uuid>,
        user_id: Option<Uuid>,
        challonge_id: Option<String>
    ) -> Result<u64, DbErr> {
        let conditions = Condition::all()
            .add(participant::Column::TournamentId.eq(tournament_id))
            .add_option(if id.is_some() {
                Some(expr::Expr::col(participant::Column::Id).eq(id.unwrap()))
            } else {
                None::<SimpleExpr>
            })
            .add_option(if user_id.is_some() {
                Some(expr::Expr::col(participant::Column::UserId).eq(user_id.unwrap()))
            } else {
                None::<SimpleExpr>
            })
            .add_option(if challonge_id.is_some() {
                Some(expr::Expr::col(participant::Column::ChallongeId).eq(challonge_id.unwrap()))
            } else {
                None::<SimpleExpr>
            });
        let participant_to_delete = participant::Entity::find()
            .filter(conditions)
            .one(db)
            .await?;
        if let Some(model_to_delete) = participant_to_delete {
            model_to_delete.delete(db).await?;
            let count = participant::Entity::find()
                .filter(participant::Column::TournamentId.eq(tournament_id))
                .count(db)
                .await?;
            Ok(count)
        } else {
            Err(DbErr::RecordNotFound("No participant to delete found".to_string()))
        }
    }

    pub async fn create_organizer(
        &self,
        db: &DatabaseConnection,
        discord_id: String,
        challonge_key: String
    ) -> Result<Uuid, String> {
        let id = Uuid::new_v4();

        let model = organizer::ActiveModel {
            id: Set(id),
            discord_id: Set(i64::from_str_radix(&discord_id, 10).unwrap()),
            challonge_api_key: Set(challonge_key)
        };

        let res = model.insert(db).await;
        match res {
            Ok( _res) => {
                Ok(id)
            },
            Err(error) => {
                Err(error.to_string())
            }
        }
    }

    pub async fn get_organizer(
        &self,
        db: &DatabaseConnection,
        id: Option<Uuid>,
        discord_id: Option<i64>,
        challonge_key: Option<String>
    ) -> Result<Option<OrganizerModel>, String> {
        let condition = Condition::all()
            .add_option(if id.is_some() {
                Some(expr::Expr::col(organizer::Column::Id).eq(id.unwrap()))
            } else {
                None::<SimpleExpr>
            })
            .add_option(if discord_id.is_some() {
                Some(expr::Expr::col(organizer::Column::DiscordId).eq(discord_id.unwrap()))
            } else {
                None::<SimpleExpr>
            })
            .add_option(if challonge_key.is_some() {
                Some(expr::Expr::col(organizer::Column::ChallongeApiKey).eq(challonge_key.unwrap()))
            } else {
                None::<SimpleExpr>
            });

        let res = organizer::Entity::find()
            .filter(condition)
            .one(db)
            .await;

        match res {
            Ok(model) => {
                Ok(model)
            },
            Err(error) => {
                Err(error.to_string())
            }
        }
    }

    pub async fn create_tournament_builder(
        &self,
        db: &DatabaseConnection,
        message_id: String
    ) -> Result<TournamentBuilderModel, String> {
        let id = Uuid::new_v4();

        let model = tournament_builder::ActiveModel {
            id: Set(id),
            message_id: Set(i64::from_str_radix(&message_id, 10).unwrap()),
            name: Set(None),
            edit_state: Set(Some(TournamentEditState::NotSelected)),
            register_channel: Set(None),
            reports_channel: Set(None),
            role: Set(None),
            use_bargains: Set(None),
            use_bargains_color: Set(None),
            use_foreign_heroes: Set(None)
        };

        let res = model.insert(db).await;
        match res {
            Ok(res) => {
                Ok(res)
            },
            Err(error) => {
                Err(error.to_string())
            }
        }
    }

    pub async fn get_tournament_builder(
        &self,
        db: &DatabaseConnection,
        id: Option<Uuid>,
        message_id: Option<i64>
    ) -> Result<Option<TournamentBuilderModel>, String> {
        let condition = Condition::all()
            .add_option(if id.is_some() {
                Some(expr::Expr::col(tournament_builder::Column::Id).eq(id.unwrap()))
            } else {
                None::<SimpleExpr>
            })
            .add_option(if message_id.is_some() {
                Some(expr::Expr::col(tournament_builder::Column::MessageId).eq(message_id.unwrap()))
            } else {
                None::<SimpleExpr>
            });

        let res = tournament_builder::Entity::find()
            .filter(condition)
            .one(db)
            .await;

        match res {
            Ok(model) => {
                Ok(model)
            },
            Err(error) => {
                Err(error.to_string())
            }
        }
    }

    pub async fn update_tournament_builder(
        &self,
        db: &DatabaseConnection,
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
        let current_model = tournament_builder::Entity::find_by_id(id).one(db).await.unwrap();
        if let Some(model) = current_model {

            let mut model_to_update: tournament_builder::ActiveModel = model.into();

            if let Some(name) = name {
                model_to_update.name = Set(Some(name))
            }

            if let Some(state) = state {
                model_to_update.edit_state = Set(Some(state))
            }

            if let Some(register_channel) = register_channel {
                model_to_update.register_channel = Set(Some(i64::from_str_radix(&register_channel, 10).unwrap()));
            }

            if let Some(reports_channel) = reports_channel {
                model_to_update.reports_channel = Set(Some(i64::from_str_radix(&reports_channel, 10).unwrap()));
            }

            if let Some(role) = role {
                model_to_update.role = Set(Some(i64::from_str_radix(&role, 10).unwrap()));
            }

            if let Some(use_bargains) = use_bargains {
                model_to_update.use_bargains = Set(Some(use_bargains));
            }

            if let Some(use_bargains_color) = use_bargains_color {
                model_to_update.use_bargains_color = Set(Some(use_bargains_color));
            }

            if let Some(use_foreign_heroes) = use_foreign_heroes {
                model_to_update.use_foreign_heroes = Set(Some(use_foreign_heroes));
            }

            let res = model_to_update.update(db).await;
            match res {
                Ok(updated_model) => {
                    Ok(updated_model)
                },
                Err(error) => {
                    Err(error.to_string())
                }
            }
        } else {
            Err(format!("No tournament_builder model found with id {}", id))
        }
    }

    pub async fn get_tournaments_by_organizer(&self, db: &DatabaseConnection, organizer: Uuid) -> Result<Vec<TournamentModel>, String> {
        let res = tournament::Entity::find()
            .filter(tournament::Column::Organizer.eq(organizer))
            .all(db)
            .await;

        match res {
            Ok(models) => {
                Ok(models)
            },
            Err(error) => {
                Err(error.to_string())
            }
        }
    }

    pub async fn get_users_by_tournament(&self, db: &DatabaseConnection, tournament_id: Uuid) -> Result<Vec<UserModel>, String> {
        let res = user::Entity::find()
            .inner_join(participant::Entity)
            .filter(participant::Column::TournamentId.eq(tournament_id))
            .all(db)
            .await;

        match res {
            Ok(models) => {
                Ok(models)
            },
            Err(error) => {
                Err(error.to_string())
            }
        }
    }

    pub async fn participants_bulk_update(&self, db: &DatabaseConnection, data: Vec<UpdateParticipant>) -> Result<(), String> {
        let transaction = db.begin().await.unwrap();
        for update_data in data {
            let current_model = participant::Entity::find()
                .filter(participant::Column::TournamentId.eq(update_data.tournament_id))
                .filter(participant::Column::UserId.eq(update_data.user_id))
                .one(db)
                .await.unwrap();
            if let Some(model) = current_model {
                let mut model_to_update: participant::ActiveModel = model.into();
                model_to_update.challonge_id = Set(Some(update_data.challonge_id));
                model_to_update.update(db).await.unwrap();
            }
        }
        let res = transaction.commit().await;
        match res {
            Ok(res) => {
                tracing::info!("Participants were updated");
                Ok(())
            },
            Err(error) => {
                Err(error.to_string())
            }
        }
    }

    pub async fn users_bulk_update(&self, db: &DatabaseConnection, data: Vec<UserBulkUpdatePayload>) -> Result<(), String> {
        let transaction = db.begin().await.unwrap();
        for update_data in data {
            let current_model = user::Entity::find().filter(user::Column::Id.eq(update_data.id)).one(db).await.unwrap();
            if let Some(model) = current_model {
                let mut model_to_update: user::ActiveModel = model.into();
                if let Some(discord_nick) = update_data.discord_nick {
                    model_to_update.discord_nick = Set(discord_nick);
                }
                model_to_update.update(db).await.unwrap();
            }
        }
        let res = transaction.commit().await;
        match res {
            Ok(res) => {
                tracing::info!("Users were updated");
                Ok(())
            },
            Err(error) => {
                Err(error.to_string())
            }
        }
    }

    pub async fn get_games_count(&self, db: &DatabaseConnection, match_id: Uuid) -> Result<u64, DbErr> {
        let count = game_builder::Entity::find()
            .filter(game_builder::Column::MatchId.eq(match_id))
            .count(db)
            .await?;
        Ok(count)
    }

    pub async fn get_heroes_new(&self, db: &DatabaseConnection, mod_type: ModType) -> Result<HeroesModel, DbErr> {
        match heroes::Entity::find()
            .filter(heroes::Column::ModType.eq(mod_type))
            .one(db)
            .await? 
        { Some(model) => {
            Ok(model)
        } _ => {
            Err(DbErr::RecordNotFound(format!("No heroes found for mod {:?}", mod_type)))
        }}
    }
}