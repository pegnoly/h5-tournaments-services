use super::source::{create_tournament, parse_reports_messages, try_get_tournament_by_channel, try_get_tournament_by_id};

// if tournament already exists and has reports channel id
#[poise::command(slash_command)]
pub async fn init_existing_tournament(
    context: crate::Context<'_>,
    name: String,
    reports_channel_id: String // discord doesn't allow to enter so big numbers even if i'm trying to use 128-bit ints
) -> Result<(), crate::Error> {

    let converted_channel_id = u64::from_str_radix(&reports_channel_id, 10).unwrap();
    let client = context.data().client.read().await;

    let res = try_get_tournament_by_channel(&client, converted_channel_id).await;
    match res {
        Ok(possible_tournament) => {
            if let Some(_t) = possible_tournament {
                context.reply("There is already tournament that uses this channel for reports").await.unwrap();
                Ok(())
            }
            else {
                if let Ok(message) = create_tournament(&client, context.guild_id().unwrap().get(), converted_channel_id, name).await {
                    context.reply(message).await.unwrap();
                    Ok(())
                }
                else {
                    Err(crate::Error::from("Failed to send tournament creation request"))
                }
            }
        },
        Err(e) => {
            Err(crate::Error::from(e))
        }
    }
}

#[poise::command(slash_command)]
pub async fn parse_results(
    context: crate::Context<'_>,
    tournament_id: String
) -> Result<(), crate::Error> {
    // get channel
    let res = try_get_tournament_by_id(&context.data().client.read().await, &tournament_id).await;
    match res {
        Ok(possible_tournament) => {
            match possible_tournament {
                Some(tournament) => {
                    context.reply(format!("Got existing tournament with channel id {}", tournament.channel_id)).await.unwrap();
                    parse_reports_messages(&context, &tournament).await.unwrap();
                    Ok(())
                },
                None => {
                    context.reply(format!("No existing tournament with id {}", &tournament_id)).await.unwrap();
                    Ok(())
                }
            }
        },
        Err(e) => {
            Err(crate::Error::from(e))
        }
    }
}