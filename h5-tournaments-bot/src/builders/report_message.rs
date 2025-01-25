use poise::serenity_prelude::{ComponentInteraction, CreateActionRow, CreateButton, CreateInteractionResponseMessage, CreateMessage, CreateSelectMenu, CreateSelectMenuOption};

pub async fn initial_build(
    context: &crate::Context<'_>,
    interaction: &ComponentInteraction
) -> Result<(), crate::Error> {
    let message_builder = CreateInteractionResponseMessage::new()
        .content("Базовые данные")
        .select_menu(CreateSelectMenu::new(
            "opponent_select_menu",
             poise::serenity_prelude::CreateSelectMenuKind::String { 
                options: vec![
                    CreateSelectMenuOption::new("opponent_1", "Test opponent 1"),
                    CreateSelectMenuOption::new("opponent_2", "Test opponent 2"),
                    CreateSelectMenuOption::new("opponent_3", "Test opponent 3")
                ] 
            })
            .placeholder("Укажите ник оппонента")
        )
        .select_menu(CreateSelectMenu::new(
            "games_count_menu", 
            poise::serenity_prelude::CreateSelectMenuKind::String { 
                options: vec![
                    CreateSelectMenuOption::new("games_count_1", "1"),
                    CreateSelectMenuOption::new("games_count_2", "2"),
                    CreateSelectMenuOption::new("games_count_3", "3"),
                    CreateSelectMenuOption::new("games_count_4", "4"),
                    CreateSelectMenuOption::new("games_count_5", "5")
                ]  
            })
            .placeholder("Укажите число игр")
        )
        .button(CreateButton::new("start_report").label("Начать заполнение отчета"))
        .ephemeral(true);

    interaction.create_response(context, poise::serenity_prelude::CreateInteractionResponse::Message(message_builder)).await?;
    Ok(())
}

pub async fn build(message_id: u64) {

}