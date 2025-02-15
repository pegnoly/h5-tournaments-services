use poise::serenity_prelude::*;

pub async fn build_base_interface(
    context: &Context,
    interaction: &ComponentInteraction
) -> Result<(), crate::Error> {
    let response_message = CreateInteractionResponseMessage::new()
        .components(vec![
            CreateActionRow::Buttons(vec![
                CreateButton::new("setup_tournament_name_button").label("Указать имя турнира").style(ButtonStyle::Success),
                CreateButton::new("setup_tournament_channels_button").label("Указать связанные с турниром каналы").style(ButtonStyle::Secondary),
                CreateButton::new("setup_tournament_reports_button").label("Указать параметры отчетов турнира").style(ButtonStyle::Secondary)
            ])
        ])
        .ephemeral(true)
        .content("**Настроить параметры турнира для бота.**\n_Эти параметры **НЕ СВЯЗАНЫ** с Challonge.com турниром, они определяют только взаимодействие участников турнира с ботом._");

    interaction.create_response(context, CreateInteractionResponse::Message(response_message)).await?;
    Ok(())
}

pub async fn build_channels_selection_interface(
    context: &Context,
    interaction: &ComponentInteraction
) -> Result<(), crate::Error> {
    let response_message = CreateInteractionResponseMessage::new()
        .content("**Укажите необходимые каналы и роли для турнира**")
        .select_menu(CreateSelectMenu::new("registration_channel_selector", 
            CreateSelectMenuKind::Channel {channel_types: Some(vec![ChannelType::Text]), default_channels: None })
            .placeholder("Укажите канал для регистрации юзеров")
        )
        .select_menu(CreateSelectMenu::new("reports_channel_selector",
            CreateSelectMenuKind::Channel { channel_types: Some(vec![ChannelType::Text]), default_channels: None })
            .placeholder("Укажите канал для заполнения отчетов")
        )
        .select_menu(CreateSelectMenu::new("tournament_role_selector", 
            CreateSelectMenuKind::Role { default_roles: None })
            .placeholder("Укажите роль для участника турнира")
        );
    
    interaction.create_response(context, CreateInteractionResponse::UpdateMessage(response_message)).await?;
    Ok(())
}

pub async fn build_reports_data_selection_interface(
    context: &Context,
    interaction: &ComponentInteraction
) -> Result<(), crate::Error> {
    let response_message = CreateInteractionResponseMessage::new()
    .content("**Укажите параметры отчетов турнира**")
    .select_menu(CreateSelectMenu::new("tournament_bargains_usage_selector", 
        CreateSelectMenuKind::String { options: vec![
            CreateSelectMenuOption::new("Использовать торги", "UseBargains"),
            CreateSelectMenuOption::new("Не спользовать торги", "DontUseBargains")
        ]})
        .placeholder("Укажите статус использования торгов")
    )
    .select_menu(CreateSelectMenu::new("tournament_bargains_color_usage_selector",
        CreateSelectMenuKind::String { options: vec![
            CreateSelectMenuOption::new("Указывать цвет торга", "UseBargainsColor"),
            CreateSelectMenuOption::new("Не указывать цвет торга", "DontUseBargainsColor")
        ]})
        .placeholder("Укажите статус использования цвета торгов")
    )
    .select_menu(CreateSelectMenu::new("tournament_foreign_heroes_usage_selector",
        CreateSelectMenuKind::String { options: vec![
            CreateSelectMenuOption::new("Можно использовать неродных героев", "UseForeignHeroes"),
            CreateSelectMenuOption::new("Нельзя использовать неродных героев", "DontUseForeignHeroes")
        ]})
        .placeholder("Укажите статус использования неродных героев")
    );

    interaction.create_response(context, CreateInteractionResponse::UpdateMessage(response_message)).await?;
    Ok(())
}