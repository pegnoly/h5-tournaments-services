use anyhow::Context as _;
use api_connector::service::ApiConnectionService;
use parser::service::ParserService;
use poise::serenity_prelude::{ClientBuilder, GatewayIntents};
use shuttle_runtime::SecretStore;
use shuttle_serenity::ShuttleSerenity;

pub mod commands;
pub mod parser;
pub mod api_connector;
pub mod graphql;

struct Data {
    pub api_connection_service: ApiConnectionService,
    pub parser_service: ParserService
} // User data, which is stored and accessible in all command invocations
type Error = Box<dyn std::error::Error + Send + Sync>;
type Context<'a> = poise::Context<'a, Data, Error>;


#[shuttle_runtime::main]
async fn main(#[shuttle_runtime::Secrets] secret_store: SecretStore) -> ShuttleSerenity {
    // Get the discord token set in `Secrets.toml`
    let discord_token = secret_store
        .get("DISCORD_TOKEN")
        .context("'DISCORD_TOKEN' was not found")?;

    let framework = poise::Framework::builder()
        .options(poise::FrameworkOptions {
            commands: vec![
                commands::init_tournament(),
                commands::parse_results(),
                commands::init_services(),
                commands::create_user()
            ],
            ..Default::default()
        })
        .setup(|ctx, _ready, framework| {
            Box::pin(async move {
                poise::builtins::register_globally(ctx, &framework.options().commands).await?;
                Ok(Data {
                    api_connection_service: ApiConnectionService::new(reqwest::Client::new()),
                    parser_service: ParserService {}
                })
            })
        })
        .build();

    let client = ClientBuilder::new(discord_token, GatewayIntents::all())
        .framework(framework)
        .await
        .map_err(shuttle_runtime::CustomError::new)?;

    Ok(client.into())
}
