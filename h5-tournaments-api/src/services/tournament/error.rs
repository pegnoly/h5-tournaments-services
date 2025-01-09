use derive_more::derive::From;
use strum::Display;

#[derive(From, Debug, Display)]
pub enum Error {
    SqlxError(sqlx::Error)
}