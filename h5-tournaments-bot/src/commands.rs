use std::vec;

use futures_executor::block_on_stream;
use h5_tournaments_api::prelude::ModType;
use poise::serenity_prelude::{futures::StreamExt, ChannelId, ChannelType, ComponentInteractionCollector, ComponentInteractionDataKind, CreateButton, CreateChannel, CreateMessage, GetMessages, Message, MessageId, PermissionOverwrite, PermissionOverwriteType, Permissions, UserId};
use serde_json::json;
use uuid::Uuid;

use crate::{builders, graphql::queries::create_user_mutation::Variables, parser::{types::HrtaParser, utils::ParsingDataModel}};

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
    let messages = channel.messages(context, GetMessages::new().after(tournament.first_message_id as u64).limit(100)).await.unwrap();

    let messages_filtered = messages.into_iter().filter(|message| {
        message.id.get() <= tournament.last_message_id as u64
    }).collect::<Vec<Message>>();

    for message in &messages_filtered {
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
            for message in &messages_filtered {
                let mut parsed_data = parser_service.parse_match_structure(&message.content, &HrtaParser{}, &data_model);
                api_connection_service.send_match(&mut parsed_data, tournament.id, message.id.get() as i64).await?;
            }
        }
    }
    
    context.say("Success").await?;

    Ok(())
}

#[poise::command(slash_command)]
pub async fn create_user(
    context: crate::Context<'_>,
    #[description = "User's nickname for tournaments system"]
    nickname: String,
    #[description = "User's discord id"]
    id: String
) -> Result<(), crate::Error> {
    let api_connection_service = &context.data().api_connection_service;
    let res = api_connection_service.create_user(nickname, id).await;
    match res {
        Ok(res) => {
            context.say(res).await.unwrap();
            Ok(())
        },
        Err(error) => {
            Err(crate::Error::from(error))
        }
    }
}

#[poise::command(slash_command)]
pub async fn init_services(
    context: crate::Context<'_>
) -> Result<(), crate::Error> {
    let guild = context.guild_id().unwrap();
    let bot_category = guild.create_channel(context, 
        CreateChannel::new("Tournaments actions")
            .kind(ChannelType::Category)
            //.kind(ChannelType::Private)
            .permissions(vec![PermissionOverwrite {
                allow: Permissions::VIEW_CHANNEL | Permissions::SEND_MESSAGES,
                kind: PermissionOverwriteType::Member(UserId::new(436937919308234762)),
                deny: Permissions::ADMINISTRATOR
            }])
    ).await.unwrap();
    let channel = guild.create_channel(context, 
        CreateChannel::new("tournament-actions")
            .category(bot_category.id)
            .kind(ChannelType::Text)
        ).await?;
    
    let create_message_button = CreateButton::new("create_tournament_button")
        .label("Create tournament");
    let message = CreateMessage::new()
        .button(create_message_button);
    channel.send_message(context, message).await?;

    while let Some(interaction) = ComponentInteractionCollector::new(context).channel_id(channel.id).next().await {
        match interaction.data.kind {
            ComponentInteractionDataKind::Button => {
                if interaction.data.custom_id == "create_tournament_button".to_string()  {
                    println!("Create tournament pressed")
                }
            },
            _=> {}
        }
    }

    Ok(())
}

#[poise::command(slash_command)]
pub async fn setup_tournament(
    context: crate::Context<'_>,
    name: String,
    operator_id: Uuid,
    reports_channel: String
) -> Result<(), crate::Error> {

    // let api_connection_service = &context.data().api_connection_service;
    // let section_id = api_connection_service.get_operator(operator_id).await?;
    // let guild = context.guild_id().unwrap();
    // let permissions = guild.roles(context).await?.iter()
    //     .filter_map(|(id, role)| {
    //         if role.has_permission(Permissions::ADMINISTRATOR) {
    //             None
    //         }
    //         else {
    //             match role.name.as_str() {
    //                 "homm5-tournaments-bot" | "Статистика" => {
    //                     // tracing::info!("Changing permissions for role {}", role.name);
    //                     // Some(PermissionOverwrite { 
    //                     //     allow: Permissions::VIEW_CHANNEL | Permissions::SEND_MESSAGES | Permissions::USE_APPLICATION_COMMANDS | Permissions::USE_EXTERNAL_APPS, 
    //                     //     deny: Permissions::MANAGE_GUILD | Permissions::MANAGE_CHANNELS, 
    //                     //     kind: PermissionOverwriteType::Role(*id)
    //                     // })
    //                     None
    //                 },
    //                 _=> {
    //                     Some(PermissionOverwrite { 
    //                         allow: role.permissions, 
    //                         deny: Permissions::empty(), 
    //                         kind: PermissionOverwriteType::Role(*id) 
    //                     })
    //                 }
    //             }
    //         }
    //     }).collect::<Vec<PermissionOverwrite>>();

    // let channel = guild.create_channel(context, 
    //     CreateChannel::new(format!("отчеты-{}", &name))
    //         .kind(ChannelType::Text)
    //         //.category(ChannelId::from(section_id as u64))
    //         .permissions(permissions)
    //     ).await?;
    
    // tracing::info!("Channel created: {}", &channel.id.get());
    // let button = CreateButton::new("create_report_button").label("Написать отчет").disabled(false);
    // let message = CreateMessage::new().button(button);
    // channel.send_message(context, message).await?;

    // let create_tournament_res = api_connection_service.create_tournament(name, operator_id, channel.id.get() as i64).await?;
    // context.say(create_tournament_res).await?;

    let api_connection_service = &context.data().api_connection_service;
    let section_id = api_connection_service.get_operator_section(operator_id).await?;
    let channel = ChannelId::from(u64::from_str_radix(&reports_channel, 10)?);

    let button = CreateButton::new("create_report_button").label("Написать отчет").disabled(false);
    let message = CreateMessage::new().button(button);
    channel.send_message(context, message).await?;

    let create_tournament_res = api_connection_service.create_tournament(name, operator_id, channel.get() as i64).await?;
    context.say(create_tournament_res).await?;

    Ok(())
}

#[poise::command(slash_command)]
pub async fn delete_unused(
    context: crate::Context<'_>,
    channel: String,
    message_id: String
) -> Result<(), crate::Error> {
    let id = u64::from_str_radix(&message_id, 10).unwrap();
    let guild = context.guild_id().unwrap();
    let channel_id = u64::from_str_radix(&channel, 10).unwrap();
    let channel = ChannelId::from(channel_id);
    channel.delete_message(context, id).await.unwrap();
    Ok(())
}