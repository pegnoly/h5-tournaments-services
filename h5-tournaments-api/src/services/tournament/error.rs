use derive_more::derive::From;
use strum::Display;

#[derive(From, Debug, Display)]
pub enum Error {
    SeaOrm(#[from] sea_orm::DbErr),
    GraphQL(#[from] async_graphql::Error)
}