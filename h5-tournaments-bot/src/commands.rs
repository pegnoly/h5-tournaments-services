use std::{str::FromStr, vec};

use h5_tournaments_api::prelude::ModType;
use poise::serenity_prelude::*;
use serde::{Deserialize, Serialize};
use serde_json::json;
use uuid::Uuid;

use crate::{
    graphql::queries::update_users_bulk,
    parser::{types::HrtaParser, utils::ParsingDataModel}, types::payloads::GetTournament,
};

/// This command collects user input and if everything is correct sends tournament creating request
// Correctness check isn't implemented yet and i think won't be cause in new project this won't be used.
#[poise::command(slash_command)]
pub async fn init_tournament(
    context: crate::Context<'_>,
    #[description = "Modification type: Universe(0) or Hrta(1)"] mod_type: i32,
    #[description = "Name of tournament"] name: String,
    #[description = "Id of channel with tournament reports"] channel_id: String,
    #[description = "Id of first message with tournament's data"] first_message_id: String,
    #[description = "Id of last message with tournament's data"] last_message_id: String,
) -> Result<(), crate::Error> {
    let server_id = context.guild_id().unwrap().get() as i64;
    let channel_id = u64::from_str_radix(&channel_id, 10).unwrap();
    let first_message_id = u64::from_str_radix(&first_message_id, 10).unwrap();
    let last_message_id = u64::from_str_radix(&last_message_id, 10).unwrap();

    let h5_tournament_service = &context.data().h5_tournament_service;
    let answer = h5_tournament_service
        .init_tournament(&json!({
            "mod_type": mod_type,
            "name": name,
            "server_id": server_id,
            "channel_id": channel_id as i64,
            "first_message_id": first_message_id as i64,
            "last_message_id": last_message_id as i64,
        }))
        .await?;

    context.say(answer).await?;

    Ok(())
}

#[derive(Debug, Serialize, Deserialize)]
pub struct TempMessageModel {
    pub message_id: u64,
    pub message_text: String,
    pub tournament_id: Uuid
}

#[poise::command(slash_command)]
pub async fn deprecated_get_messages(
    context: crate::Context<'_>,
    tournament_id: String,
    channel_id: String,
    first_message: String,
    last_message: String 
) -> Result<(), crate::Error> {
    let h5_tournament_service = &context.data().h5_tournament_service;
    let tournament_id = Uuid::from_str(&tournament_id)?;
    
    let channel = ChannelId::new(u64::from_str_radix(&channel_id, 10)?);
    let messages = channel
        .messages(
            context,
            GetMessages::new()
                .after(u64::from_str_radix(&first_message, 10)?)
                .limit(100),
        )
        .await
        .unwrap();

    let last_message_id = u64::from_str_radix(&last_message, 10)?;
    let messages_filtered = messages
        .into_iter()
        .filter(|message| message.id.get() <= last_message_id)
        .map(|message| {
            TempMessageModel {
                message_id: message.id.get(),
                message_text: message.content,
                tournament_id: tournament_id
            }
        })
        .collect::<Vec<TempMessageModel>>();
    h5_tournament_service.load_messages(messages_filtered).await?;
    Ok(())
}

/// This command checks is requested tournament registered and if so starts process of its parsing.
// #[poise::command(slash_command)]
// pub async fn parse_results(
//     context: crate::Context<'_>,
//     #[description = "Id of tournament to parse results"] tournament_id: String,
// ) -> Result<(), crate::Error> {
//     let h5_tournament_service = &context.data().h5_tournament_service;
//     let parser_service = &context.data().parser_service;

//     let tournament = h5_tournament_service
//         .get_tournament_data(GetTournament::default().with_id(id))

//     let channel = ChannelId::new(tournament.channel_id as u64);
//     let messages = channel
//         .messages(
//             context,
//             GetMessages::new()
//                 .after(tournament.first_message_id as u64)
//                 .limit(100),
//         )
//         .await
//         .unwrap();

//     let messages_filtered = messages
//         .into_iter()
//         .filter(|message| message.id.get() <= tournament.last_message_id as u64)
//         .collect::<Vec<Message>>();

//     for message in &messages_filtered {
//         tracing::info!("{:?}", message.content);
//     }

//     let mod_type = ModType::from_repr(tournament.mod_type).unwrap();
//     let races = h5_tournament_service.load_races().await?;
//     let heroes = h5_tournament_service.load_heroes(mod_type).await?;
//     let data_model = ParsingDataModel {
//         races: races,
//         heroes: heroes,
//     };

//     match mod_type {
//         ModType::Universe => {
//             //process_messages(service, &messages, UniverseParser {}, &data_model);
//         }
//         ModType::Hrta => {
//             tracing::info!("Processing hrta data");
//             for message in &messages_filtered {
//                 let mut parsed_data = parser_service.parse_match_structure(
//                     &message.content,
//                     &HrtaParser {},
//                     &data_model,
//                 );
//                 h5_tournament_service
//                     .send_match(&mut parsed_data, tournament.id, message.id.get() as i64)
//                     .await?;
//             }
//         }
//     }

//     context.say("Success").await?;

//     Ok(())
// }

// // #[poise::command(slash_command)]
// // pub async fn create_user(
// //     context: crate::Context<'_>,
// //     #[description = "User's nickname for tournaments system"]
// //     nickname: String,
// //     #[description = "User's discord id"]
// //     id: String
// // ) -> Result<(), crate::Error> {
// //     // let h5_tournament_service = &context.data().h5_tournament_service;
// //     // let res = h5_tournament_service.create_user(nickname, id, false).await;
// //     // match res {
// //     //     Ok(res) => {
// //     //         context.say(res).await.unwrap();
// //     //         Ok(())
// //     //     },
// //     //     Err(error) => {
// //     //         Err(crate::Error::from(error))
// //     //     }
// //     // }
// // }

// #[poise::command(slash_command)]
// pub async fn init_services(context: crate::Context<'_>) -> Result<(), crate::Error> {
//     let guild = context.guild_id().unwrap();
//     let bot_category = guild
//         .create_channel(
//             context,
//             CreateChannel::new("Tournaments actions")
//                 .kind(ChannelType::Category)
//                 //.kind(ChannelType::Private)
//                 .permissions(vec![PermissionOverwrite {
//                     allow: Permissions::VIEW_CHANNEL | Permissions::SEND_MESSAGES,
//                     kind: PermissionOverwriteType::Member(UserId::new(436937919308234762)),
//                     deny: Permissions::ADMINISTRATOR,
//                 }]),
//         )
//         .await
//         .unwrap();
//     let channel = guild
//         .create_channel(
//             context,
//             CreateChannel::new("tournament-actions")
//                 .category(bot_category.id)
//                 .kind(ChannelType::Text),
//         )
//         .await?;

//     let create_message_button =
//         CreateButton::new("create_tournament_button").label("Create tournament");
//     let message = CreateMessage::new().button(create_message_button);
//     channel.send_message(context, message).await?;

//     while let Some(interaction) = ComponentInteractionCollector::new(context)
//         .channel_id(channel.id)
//         .next()
//         .await
//     {
//         match interaction.data.kind {
//             ComponentInteractionDataKind::Button => {
//                 if interaction.data.custom_id == "create_tournament_button".to_string() {
//                     println!("Create tournament pressed")
//                 }
//             }
//             _ => {}
//         }
//     }

//     Ok(())
// }

#[poise::command(slash_command)]
pub async fn setup_tournament(
    context: crate::Context<'_>,
    #[description = "Name of tournament"] name: String,
    #[description = "Tournament operator's id"] operator_id: Uuid,
    #[description = "Id of reports channel of this tournament"] reports_channel_id: String,
    #[description = "Id of registration channel of this tournament"] register_channel_id: String,
    #[description = "Will this tournament use bargains"] use_bargains: bool,
    #[description = "Do players suppose to use foreign heroes in games"] use_foreign_heroes: bool,
    #[description = "Unique role for participants of this tournament"] role: String,
    create_objects: bool,
) -> Result<(), crate::Error> {
    let h5_tournament_service = &context.data().h5_tournament_service;
    //let section_id = h5_tournament_service.get_operator_section(operator_id).await?;
    let reports_channel = ChannelId::from(u64::from_str_radix(&reports_channel_id, 10)?);
    let register_channel = ChannelId::from(u64::from_str_radix(&register_channel_id, 10)?);

    if create_objects {
        let reports_message = CreateMessage::new().button(
            CreateButton::new("create_report_button")
                .label("Написать отчет")
                .disabled(false),
        );
        reports_channel
            .send_message(context, reports_message)
            .await?;

        let register_message =
            CreateMessage::new().components(vec![CreateActionRow::Buttons(vec![
                CreateButton::new("register_user_button")
                    .label("Зарегистрироваться в турнире")
                    .style(ButtonStyle::Success),
                CreateButton::new("unregister_user_button")
                    .label("Отменить регистрацию")
                    .style(ButtonStyle::Danger),
                CreateButton::new("update_user_data_button")
                    .label("Редактировать данные")
                    .style(ButtonStyle::Secondary),
            ])]);

        register_channel
            .send_message(context, register_message)
            .await?;
    }

    // let create_tournament_res = h5_tournament_service.create_tournament(
    //     name,
    //     operator_id,
    //     reports_channel_id,
    //     register_channel_id,
    //     use_bargains,
    //     use_foreign_heroes,
    //     role
    // ).await?;

    // context.say(create_tournament_res).await?;

    Ok(())
}

#[poise::command(slash_command)]
pub async fn delete_unused(
    context: crate::Context<'_>,
    channel: String,
    message_id: String,
) -> Result<(), crate::Error> {
    let id = u64::from_str_radix(&message_id, 10).unwrap();
    //let guild = context.guild_id().unwrap();
    let channel_id = u64::from_str_radix(&channel, 10).unwrap();
    let channel = ChannelId::from(channel_id);
    channel.delete_message(context, id).await.unwrap();
    Ok(())
}

// #[poise::command(slash_command)]
// pub async fn register_in_tournament(
//     context: crate::Context<'_>,
//     tournament: String,
//     user: String,
//     group: i64
// ) -> Result<(), crate::Error> {
//     let tournament_id = Uuid::from_str(&tournament).unwrap();
//     let user_id = Uuid::from_str(&user).unwrap();
//     let api = &context.data().h5_tournament_service;
//     let res = api.create_participant(tournament_id, user_id, group).await?;
//     context.say(res.to_string()).await?;
//     Ok(())
// }

#[poise::command(slash_command)]
pub async fn get_tournaments(context: crate::Context<'_>) -> Result<(), crate::Error> {
    let service = &context.data().challonge_service;
    //service.get_tournaments().await?;
    Ok(())
}

#[poise::command(slash_command)]
pub async fn test_challonge_participant_add(
    context: crate::Context<'_>,
    tournament_id: String,
    participant_id: String,
    participant_name: String,
) -> Result<(), crate::Error> {
    let service = &context.data().challonge_service;
    //service.add_participant(tournament_id, participant_id, participant_name).await?;
    Ok(())
}

#[poise::command(slash_command)]
pub async fn build_administration_panel(
    context: crate::Context<'_>,
    channel_id: String,
) -> Result<(), crate::Error> {
    let message_builder = CreateMessage::new().components(vec![CreateActionRow::Buttons(vec![
        CreateButton::new("admin_registration_button")
            .label("Привязать свой Challonge.com ключ API")
            .style(ButtonStyle::Primary),
        CreateButton::new("tournament_creation_button")
            .label("Зарегистрировать турнир в базе бота")
            .style(ButtonStyle::Secondary),
        CreateButton::new("tournament_sync_button")
            .label("Синхронизировать турниры с Challonge.com")
            .style(ButtonStyle::Secondary),
        CreateButton::new("administrate_tournament_button")
            .label("Настроить турнир")
            .style(ButtonStyle::Secondary),
    ])]);

    let channel = ChannelId::from(u64::from_str_radix(&channel_id, 10)?);
    channel.send_message(context, message_builder).await?;
    context
        .say(format!(
            "Administration interface built successfully in channel {}",
            &channel
        ))
        .await?;
    Ok(())
}

#[poise::command(slash_command)]
pub async fn sync_users_nicknames(context: crate::Context<'_>) -> Result<(), crate::Error> {
    let tournaments_service = &context.data().h5_tournament_service;
    let users = tournaments_service.get_users().await?.unwrap();
    let mut payload = vec![];
    for u in users {
        let user_id = UserId::from(u.discord_id as u64);
        let discord_user = user_id.to_user(context).await?;
        payload.push(update_users_bulk::UserBulkUpdatePayload {
            id: u.id,
            discord_nick: Some(discord_user.name),
        });
    }
    tournaments_service.update_users_bulk(payload).await?;
    context.say("Done").await?;
    Ok(())
}
