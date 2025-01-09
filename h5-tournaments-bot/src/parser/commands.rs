use std::str::FromStr;

use h5_tournaments_api::prelude::Tournament;
use serde_json::json;
use uuid::Uuid;

use crate::parser::source::process_tournament;

pub(crate) const MAIN_URL: &'static str = "https://h5-tournaments-api-5epg.shuttle.app/";


/// This command collects user input and if everything is correct sends tournament creating request
// Correctness check isn't implemented yet and i think won't be cause in new project this won't be used.
#[poise::command(slash_command)]
pub async fn init_tournament(
    context: crate::Context<'_>,
    #[description = "Modification type: Universe(0) or Hrta(1)"]
    mod_type: i32,
    #[description = "Name of tournament"]
    name: String,
    #[description = "Id of channel with tournament reports"]
    channel_id: String,
    #[description = "Id of first message with tournament's data"]
    first_message_id: String,
    #[description = "Id of last message with tournament's data"]
    last_message_id: String 
) -> Result<(), crate::Error> {
    let server_id = context.guild_id().unwrap().get() as i64;
    let channel_id = u64::from_str_radix(&channel_id, 10).unwrap();
    let first_message_id = u64::from_str_radix(&first_message_id, 10).unwrap();
    let last_message_id = u64::from_str_radix(&last_message_id, 10).unwrap();

    let json_data = json!({
        "mod_type": mod_type,
        "name": name,
        "server_id": server_id,
        "channel_id": channel_id as i64,
        "first_message_id": first_message_id as i64,
        "last_message_id": last_message_id as i64,
    });

    tracing::info!("Json data to create tournament: {:?}", &json_data);
    
    let client = context.data().client.read().await;
    let response = client
        .post(format!("{}tournament/create", MAIN_URL))
        .json(&json_data)
        .send()
        .await;

    match response {
        Ok(success) => {
            tracing::info!("Tournament creation response: {:?}", &success);
            let text = success.text().await.unwrap();
            context.say(text).await.unwrap();
            Ok(())
        },
        Err(failure) => {
            tracing::error!("Failed to send tournament creation request: {}", failure.to_string());
            Err(crate::Error::from(failure))
        }
    }
}

/// This command checks is requested tournament registered and if so starts process of its parsing.
#[poise::command(slash_command)]
pub async fn parse_results(
    context: crate::Context<'_>,
    #[description = "Id of tournament to parse results"]
    tournament_id: String
) -> Result<(), crate::Error> {

    let client = context.data().client.read().await;
    let get_tournament_response = client
        .get(format!("{}tournament/get/{}", MAIN_URL, Uuid::from_str(&tournament_id).unwrap()))
        .send()
        .await;

    match get_tournament_response {
        Ok(response) => {
            tracing::info!("Got tournament response: {:?}", &response);
            let tournament: Result<Tournament, reqwest::Error> = response.json().await;
            match tournament {
                Ok(tournament) => {
                    tracing::info!("Tournament json parsed successfully: {:?}", &tournament);
                    let parser_service = &context.data().parser_service;
                    process_tournament(&context, &client, parser_service, &tournament).await;
                    Ok(())
                },
                Err(json_error) => {
                    tracing::error!("Failed to parse tournament json: {}", &json_error.to_string());
                    Err(crate::Error::from(json_error))
                }
            }
        },
        Err(response_error) => {
            tracing::error!("Failed to send get tournament request: {}", &response_error.to_string());
            Err(crate::Error::from(response_error))
        }
    }
}