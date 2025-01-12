use h5_tournaments_api::prelude::ModType;
use poise::serenity_prelude::{ChannelId, GetMessages, Message, MessageId};
use serde_json::json;

use crate::parser::{types::HrtaParser, utils::ParsingDataModel};

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

    let api_connection_service = &context.data().api_connection_service;
    let answer = api_connection_service.init_tournament(&json!({
        "mod_type": mod_type,
        "name": name,
        "server_id": server_id,
        "channel_id": channel_id as i64,
        "first_message_id": first_message_id as i64,
        "last_message_id": last_message_id as i64,
    })).await?;

    context.say(answer).await?;

    Ok(())
}

/// This command checks is requested tournament registered and if so starts process of its parsing.
#[poise::command(slash_command)]
pub async fn parse_results(
    context: crate::Context<'_>,
    #[description = "Id of tournament to parse results"]
    tournament_id: String
) -> Result<(), crate::Error> {

    let api_connection_service = &context.data().api_connection_service;
    let parser_service = &context.data().parser_service;

    let tournament = api_connection_service.get_tournament(tournament_id).await?;

    let channel = ChannelId::new(tournament.channel_id as u64);
    let messages_builder = GetMessages::new()
        .after(MessageId::new(tournament.first_message_id as u64))
        .before(MessageId::new(tournament.last_message_id as u64));

    let messages = channel.messages(context, messages_builder).await.unwrap();
    let cleaned_messages = messages.iter()
        .filter(|m| m.id.get() >= tournament.first_message_id as u64)
        .collect::<Vec<&Message>>();

    for message in &cleaned_messages {
        tracing::info!("{:?}", message.content);
    }

    let mod_type = ModType::from_repr(tournament.mod_type).unwrap();
    let races = api_connection_service.load_races().await?;
    let heroes = api_connection_service.load_heroes(mod_type).await?;
    let data_model =  ParsingDataModel { races: races, heroes: heroes};

    match mod_type {
        ModType::Universe => {
            //process_messages(service, &messages, UniverseParser {}, &data_model);
        },
        ModType::Hrta => {
            tracing::info!("Processing hrta data");
            for message in &cleaned_messages {
                let mut parsed_data = parser_service.parse_match_structure(&message.content, &HrtaParser{}, &data_model);
                api_connection_service.send_match(&mut parsed_data, tournament.id, message.id.get() as i64).await?;
            }
        }
    }
    
    context.say("Success").await?;

    Ok(())
}