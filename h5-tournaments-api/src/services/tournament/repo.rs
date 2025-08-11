use sea_orm::{sea_query::{expr, OnConflict, SimpleExpr}, ActiveModelTrait, ColumnTrait, Condition, DatabaseConnection, DbErr, EntityTrait, IntoActiveModel, ModelTrait, PaginatorTrait, QueryFilter, Related, Set, TransactionTrait};
use uuid::Uuid;

use crate::{graphql::mutation::UpdateParticipant, services::tournament::{error::Error, models::{self, operator::TournamentOperatorModel, tournament::{GameType, ModType}, user::UserModel}}};

#[derive(Clone)]
pub struct TournamentsRepo;

impl TournamentsRepo {
    pub async fn create_user(
        &self,
        db: &DatabaseConnection,
        name: String,
        discord_id: u64,
        //confirm_register: bool
        discord_nick: String
    ) -> Result<UserModel, Error> {
        let id = Uuid::new_v4();
        let on_conflict = OnConflict::column(models::user::Column::DiscordId)
            .update_column(models::user::Column::Nickname)
            .value(models::user::Column::RegisteredManually, true)
            .to_owned();

        let user_to_insert = models::user::ActiveModel {
            id: Set(id),
            nickname: Set(name.clone()),
            discord_id: Set(discord_id as i64),
            registered_manually: Set(true),
            discord_nick: Set(discord_nick)
        };

        let model = models::user::Entity::insert(user_to_insert).on_conflict(on_conflict.clone()).exec_with_returning(db).await?;
        Ok(model)
    }

    pub async fn update_user(
        &self,
        db: &DatabaseConnection,
        id: Uuid,
        nickname: Option<String>,
        registered: Option<bool>
    ) -> Result<(), Error> {
        let current_user = models::user::Entity::find_by_id(id).one(db).await.unwrap();
        if let Some(current_user) = current_user {

            let mut user_to_update = current_user.into_active_model();

            if let Some(nickname) = nickname {
                user_to_update.nickname = Set(nickname);
            }

            if let Some(registered) = registered {
                user_to_update.registered_manually = Set(registered);
            }

            user_to_update.update(db).await?;
        }

        Ok(())
    }

    pub async fn get_operator(
        &self,
        db: &DatabaseConnection,
        id: Option<Uuid>,
        server_id: Option<i64>
    ) -> Result<Option<TournamentOperatorModel>, Error> {
        let conditions = Condition::all()
            .add_option(id.map(|id| expr::Expr::col(models::operator::Column::Id).eq(id)))
            .add_option(server_id.map(|server_id| expr::Expr::col(models::operator::Column::ServerId).eq(server_id)));

        Ok(models::operator::Entity::find().filter(conditions).one(db).await?)
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

    pub async fn get_tournaments(
        &self,
        db: &DatabaseConnection
    ) -> Result<Vec<TournamentModel>, DbErr> {
        Ok(tournament::Entity::find().all(db).await?)
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
        if let Some(existing_match) = match_structure::Entity::find().filter(match_structure::Column::ChallongeId.eq(&challonge_id)).one(db).await? {
            Ok(existing_match.id)
        } else {
            let id = Uuid::new_v4();
            let match_to_create = match_structure::ActiveModel {
                id: Set(id),
                tournament_id: Set(tournament_id),
                message_id: Set(message),
                first_player: Set(first_player),
                second_player: Set(second_player),
                challonge_id: Set(challonge_id),
                report_link: Set(None)
            };
            match_to_create.insert(db).await?;
            Ok(id)
        }
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

    pub async fn get_matches(
        &self,
        db: &DatabaseConnection,
        tournament_id: Uuid,
        user_id: Option<Uuid>
    ) -> Result<Vec<MatchModel>, DbErr> {
        let conditions = Condition::all()
            .add(expr::Expr::col(match_structure::Column::TournamentId).eq(tournament_id))
            .add_option(if user_id.is_some() {
                Some(expr::Expr::col(match_structure::Column::FirstPlayer).eq(user_id.unwrap())
                    .or(expr::Expr::col(match_structure::Column::SecondPlayer).eq(user_id.unwrap())))
            } else {
                None::<SimpleExpr>
            });
        
        let matches = match_structure::Entity::find().filter(conditions).all(db).await?;
        Ok(matches)
    }

    pub async fn get_users(
        &self, 
        db: &DatabaseConnection,
        tournament_id: Uuid
    ) -> Result<Vec<UserModel>, DbErr> {
        let users = participant::Entity::find_related()
            .filter(participant::Column::TournamentId.eq(tournament_id))
            .all(db)
            .await?;

        Ok(users)
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

    pub async fn update_game(
        &self,
        db: &DatabaseConnection,
        id: Uuid,
        first_player_race: Option<i32>,
        first_player_hero: Option<i32>,
        second_player_race: Option<i32>,
        second_player_hero: Option<i32>,
        bargains_color: Option<BargainsColor>,
        bargains_amount: Option<i32>,
        result: Option<GameResult>,
        outcome: Option<GameOutcome>
    ) -> Result<String, String> {
        let current_game = game_builder::Entity::find()
            .filter(game_builder::Column::Id.eq(id))
            .one(db)
            .await.unwrap();

        if let Some(game) = current_game {
            let mut game_to_update: game_builder::ActiveModel = game.into();
            if let Some(first_player_race) = first_player_race {
                game_to_update.first_player_race = Set(Some(first_player_race));
            }
            if let Some(first_player_hero) = first_player_hero {
                game_to_update.first_player_hero = Set(Some(first_player_hero));
            }
            if let Some(second_player_race) = second_player_race {
                game_to_update.second_player_race = Set(Some(second_player_race));
            }
            if let Some(second_player_hero) = second_player_hero {
                game_to_update.second_player_hero = Set(Some(second_player_hero));
            }
            if let Some(bargains_color) = bargains_color {
                game_to_update.bargains_color = Set(Some(bargains_color));
            }
            if let Some(bargains_amount) = bargains_amount {
                game_to_update.bargains_amount = Set(Some(bargains_amount));
            }
            if let Some(result) = result {
                game_to_update.result = Set(result);
            }
            if let Some(outcome) = outcome {
                game_to_update.outcome = Set(outcome);
            }

            let res = game_to_update.update(db).await;
            match res {
                Ok(_success) => {
                    Ok("Game updated successfully".to_string())
                },
                Err(error) => {
                    Err(error.to_string())
                }
            }
        }
        else {
            Err("Failed to find game".to_string())
        }
    }

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

    pub async fn get_games(
        &self,
        db: &DatabaseConnection,
        match_id: Uuid
    ) -> Result<Vec<GameModel>, DbErr> {
        let games = game_builder::Entity::find()
            .filter(game_builder::Column::MatchId.eq(match_id))
            .all(db)
            .await?;
        Ok(games)
    }

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

    pub async fn get_all_games(&self, db: &DatabaseConnection, tournament_id: Uuid) -> Result<Vec<GameModel>, DbErr> {
        let games = match_structure::Entity::find_related()
            .filter(match_structure::Column::TournamentId.eq(tournament_id))
            .all(db)
            .await?;
        Ok(games)
    } 
}