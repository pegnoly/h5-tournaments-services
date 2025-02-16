use poise::serenity_prelude::*;
use crate::services::h5_tournaments::{payloads::{CreateOrganizerPayload, GetOrganizerPayload}, service::H5TournamentsService};

pub async fn start_admin_registration(
    context: &Context,
    interaction: &ComponentInteraction,
    service: &H5TournamentsService
) -> Result<(), crate::Error> {
    if let Some(_existing_organizer) = 
        service.get_organizer(GetOrganizerPayload::default().with_discord_id(interaction.user.id.get() as i64)).await? {
        
        interaction.create_response(context, CreateInteractionResponse::Message(
            CreateInteractionResponseMessage::new()
                .ephemeral(true)
                .content("Вы уже зарегистрированы как организатор турниров")
        )).await?;
    } else {
        interaction.create_response(context, CreateInteractionResponse::Modal(
            CreateModal::new("tournament_admin_challonge_key_modal", "Укажите свой Challonge.com ключ API")
                .components(vec![
                    CreateActionRow::InputText(
                        CreateInputText::new(InputTextStyle::Short, "Введите свой ключ API", "tournament_admin_challonge_key_input")
                    )
                ])
        )).await?;
    }
    Ok(())
}

pub async fn process_admin_registration_modal(
    context: &Context,
    interaction: &ModalInteraction,
    service: &H5TournamentsService
) -> Result<(), crate::Error> {
    let user = interaction.user.id.get();
    for row in &interaction.data.components {
        for component in &row.components {
            match component {
                ActionRowComponent::InputText(text) => {
                    if text.custom_id.as_str() == "tournament_admin_challonge_key_input" {
                        service.create_organizer(CreateOrganizerPayload::new(user, text.value.as_ref().unwrap().to_owned())).await?;
                        break;
                    }
                },
                _=> {}
            }
        }
    }
    interaction.create_response(context, CreateInteractionResponse::Message(
        CreateInteractionResponseMessage::new()
            .ephemeral(true)
            .content("Вы успешно зарегистрированы, как организатор турниров.")
    )).await?;
    Ok(())
}