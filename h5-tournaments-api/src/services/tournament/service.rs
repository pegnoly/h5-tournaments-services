use sqlx::{PgPool, Pool, Postgres};
use uuid::Uuid;

use super::types::{Hero, ModType, Race, Tournament};

#[derive(Clone)]
pub struct TournamentService {
    pub pool: PgPool
}

impl TournamentService {
    pub fn new(pool: PgPool) -> Self {
        TournamentService { pool: pool }
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
        let heroes_data = sqlx::query_as(r#"
                SELECT * FROM heroes WHERE mod_type=$1;
            "#)
            .bind(mod_type)
            .fetch_all(&self.pool)
            .await?;

        Ok(heroes_data)
    }
}