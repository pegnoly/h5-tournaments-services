use std::collections::HashMap;

use anyhow::Context as _;
use event_handler::MainEventHandler;
use parser::service::ParserService;
use poise::serenity_prelude::{ClientBuilder, GatewayIntents};
use services::challonge::service::ChallongeService;
use services::h5_tournaments::service::H5TournamentsService;
use shuttle_runtime::SecretStore;
use shuttle_serenity::ShuttleSerenity;

pub mod builders;
pub mod commands;
pub mod event_handler;
pub mod graphql;
pub mod operations;
pub mod parser;
pub mod services;
pub mod types;

pub struct Data {
    pub h5_tournament_service: std::sync::Arc<H5TournamentsService>,
    pub challonge_service: std::sync::Arc<ChallongeService>,
    pub parser_service: ParserService,
} // User data, which is stored and accessible in all command invocations
type Error = Box<dyn std::error::Error + Send + Sync>;
type Context<'a> = poise::Context<'a, Data, Error>;

#[shuttle_runtime::main]
async fn main(#[shuttle_runtime::Secrets] secret_store: SecretStore) -> ShuttleSerenity {
    let discord_token = secret_store
        .get("DISCORD_TOKEN")
        .context("'DISCORD_TOKEN' was not found")?;
    let h5_tournaments_service = std::sync::Arc::new(H5TournamentsService::new(&secret_store));
    let challonge_service = std::sync::Arc::new(ChallongeService::new(&secret_store));

    let h5_service_cloned = h5_tournaments_service.clone();
    let challonge_service_cloned = challonge_service.clone();
    let framework = poise::Framework::builder()
        .options(poise::FrameworkOptions {
            commands: vec![
                commands::init_tournament(),
                // commands::parse_results(),
                // commands::init_services(),
                //commands::create_user(),
                commands::setup_tournament(),
                commands::delete_unused(),
                //commands::register_in_tournament(),
                commands::get_tournaments(),
                commands::test_challonge_participant_add(),
                commands::build_administration_panel(),
                commands::sync_users_nicknames(),
                commands::deprecated_get_messages()
            ],
            ..Default::default()
        })
        .setup(|ctx, _ready, framework| {
            Box::pin(async move {
                poise::builtins::register_globally(ctx, &framework.options().commands).await?;
                Ok(Data {
                    h5_tournament_service: h5_service_cloned,
                    challonge_service: challonge_service_cloned,
                    parser_service: ParserService {},
                })
            })
        })
        .build();

    let client = ClientBuilder::new(discord_token, GatewayIntents::all())
        .framework(framework)
        .event_handler(MainEventHandler::new(
            h5_tournaments_service,
            challonge_service,
        ))
        .await
        .map_err(shuttle_runtime::CustomError::new)?;

    Ok(client.into())
}
