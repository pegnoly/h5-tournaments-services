use anyhow::Context as _;
use poise::serenity_prelude::{ClientBuilder, GatewayIntents};
use reqwest::Client;
use shuttle_runtime::SecretStore;
use shuttle_serenity::ShuttleSerenity;
use tokio::sync::RwLock;

pub mod parser;

struct Data {
    pub client: RwLock<Client>
} // User data, which is stored and accessible in all command invocations
type Error = Box<dyn std::error::Error + Send + Sync>;
type Context<'a> = poise::Context<'a, Data, Error>;

/// Responds with "world!"
#[poise::command(slash_command)]
async fn hello(ctx: Context<'_>) -> Result<(), Error> {
    ctx.say("world!").await?;
    Ok(())
}

#[shuttle_runtime::main]
async fn main(#[shuttle_runtime::Secrets] secret_store: SecretStore) -> ShuttleSerenity {
    // Get the discord token set in `Secrets.toml`
    let discord_token = secret_store
        .get("DISCORD_TOKEN")
        .context("'DISCORD_TOKEN' was not found")?;

    let framework = poise::Framework::builder()
        .options(poise::FrameworkOptions {
            commands: vec![
                hello(),
                parser::commands::init_existing_tournament(),
                parser::commands::parse_results()
            ],
            ..Default::default()
        })
        .setup(|ctx, _ready, framework| {
            Box::pin(async move {
                poise::builtins::register_globally(ctx, &framework.options().commands).await?;
                Ok(Data {
                    client: RwLock::new(reqwest::Client::new())
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
