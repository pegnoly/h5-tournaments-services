use rust_decimal::Decimal;
use sea_orm::{sea_query::OnConflict, ActiveModelTrait, ColumnTrait, DatabaseConnection, EntityTrait, QueryFilter, Set};
use sqlx::{PgPool, Pool, Postgres};
use uuid::Uuid;

use crate::routes::models::MatchRegistrationForm;

use self::user::{Column, Entity, UserModel};

use super::{models::{operator::{self, TournamentOperatorModel}, tournament, user}, types::{Game, Hero, Match, ModType, Race, Tournament}};

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
                LEFT JOIN matches
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
        discord_id: String
    ) -> Result<String, String> {
        let id = Uuid::new_v4();
        let discord = i64::from_str_radix(&discord_id, 10).unwrap();

        let on_conflict = OnConflict::column(Column::DiscordId).do_nothing().to_owned();

        let user_to_insert = user::ActiveModel {
            id: Set(id),
            nickname: Set(name.clone()),
            discord_id: Set(discord)
        };

        let res = Entity::insert(user_to_insert).on_conflict(on_conflict.clone()).exec(db).await;

        match res {
            Ok(_model) => {
                Ok(format!("User {} created successfully", &name))
            },
            Err(error) => {
                match error {
                    sea_orm::DbErr::RecordNotFound(_s) => {
                        Ok(format!("User {} with discord id {} already exists", &name, &discord_id))
                    },
                    sea_orm::DbErr::RecordNotInserted => {
                        Ok(format!("User {} with discord id {} already exists", &name, &discord_id))
                    }
                    _=> {
                        Err(error.to_string())
                    }
                }
            }
        }
    }

    pub async fn get_operator(
        &self,
        db: &DatabaseConnection,
        id: Uuid
    ) -> Result<Option<TournamentOperatorModel>, String> {
        let res = operator::Entity::find().filter(operator::Column::Id.eq(id)).one(db).await;
        match res {
            Ok(operator) => {
                Ok(operator)
            },
            Err(error) => {
                Err(error.to_string())
            }
        }
    }

    pub async fn create_tournament(&self, db: &DatabaseConnection, name: String, operator_id: Uuid, reports_channel_id: String) -> Result<String, String> {
        let id = Uuid::new_v4();
        let channel_id = i64::from_str_radix(&reports_channel_id, 10).unwrap();
        let tournament_to_insert = tournament::ActiveModel {
            id: Set(id),
            operator_id: Set(operator_id),
            channel_id: Set(channel_id),
            name: Set(name.clone())
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

}