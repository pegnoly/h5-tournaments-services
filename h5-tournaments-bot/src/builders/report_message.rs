use std::fmt::format;

use poise::serenity_prelude::{ComponentInteraction, Context, CreateButton, CreateInteractionResponseMessage, CreateSelectMenu, CreateSelectMenuOption};
use uuid::Uuid;
use crate::{api_connector::service::ApiConnectionService, graphql::queries::get_users_query::GetUsersQueryUsers};

pub async fn initial_build(
    context: &Context,
    api: &ApiConnectionService,
    interaction: &ComponentInteraction,
    id: &String,
    channel: u64,
    user: u64
) -> Result<(), crate::Error> {

    let tournament_data = api.get_tournament_data(None, Some(channel.to_string())).await?;
    let operator_data = api.get_operator_data(tournament_data.as_ref().unwrap().operator).await?;
    let user_data = api.get_user(None, Some(user.to_string())).await?.unwrap();
    let users = api.get_users().await?.unwrap();
    tracing::info!("Match build started by interaction {}", interaction.id.get());

    let match_creation_response = api.create_match(tournament_data.as_ref().unwrap().id, user_data.id, interaction.id.get()).await?;

    let message_builder = CreateInteractionResponseMessage::new()
        .content(format!("Отчет для турнира {} турнирного оператора {} от игрока {}", tournament_data.as_ref().unwrap().name, operator_data, user_data.nickname))
        .select_menu(create_opponent_selector(users, None))
        .select_menu(create_games_count_selector(5, None))
        .button(CreateButton::new("start_report").label("Начать заполнение отчета"))
        .ephemeral(true);

    interaction.create_response(context, poise::serenity_prelude::CreateInteractionResponse::Message(message_builder)).await?;
    Ok(())
}

pub async fn rebuild_initial(match_id: Uuid, api: &ApiConnectionService) -> Result<CreateInteractionResponseMessage, crate::Error> {
    let match_data = api.get_match(Some(match_id), None, None).await?;
    if let Some(existing_match) = match_data {
        let users = api.get_users().await?.unwrap();;
        tracing::info!("Match data: {:?}", &existing_match);
        let tournament_data = api.get_tournament_data(Some(existing_match.tournament), None).await?;
        tracing::info!("Tournament data: {:?}", &tournament_data);
        let operator_data = api.get_operator_data(tournament_data.as_ref().unwrap().operator).await?;
        tracing::info!("Operator data: {:?}", &operator_data);
        let user_data = api.get_user(Some(existing_match.first_player), None).await?.unwrap();
        tracing::info!("User data: {:?}", &user_data);
        let message_builder = CreateInteractionResponseMessage::new()
            .content(format!("Отчет для турнира {} турнирного оператора {} от игрока {}", tournament_data.as_ref().unwrap().name, operator_data, user_data.nickname))
            .select_menu(create_opponent_selector(users, existing_match.second_player))
            .select_menu(create_games_count_selector(5, existing_match.games_count))
            .button(CreateButton::new("start_report").label("Начать заполнение отчета"))
            .ephemeral(true);
        Ok(message_builder)
    }
    else {
        Err(crate::Error::from("Failed to rebuild message"))
    }
}

fn create_opponent_selector(users: Vec<GetUsersQueryUsers>, current_selected_user: Option<Uuid>) -> CreateSelectMenu {
    let options = users.iter()
        .map(|user| {
            CreateSelectMenuOption::new(user.nickname.clone(), user.id.to_string())
                .default_selection(
            if current_selected_user.is_some() && current_selected_user.unwrap() == user.id {
                        true
                    }
                    else {
                        false
                    }) 
        })
        .collect::<Vec<CreateSelectMenuOption>>();

    CreateSelectMenu::new("opponent_selector", poise::serenity_prelude::CreateSelectMenuKind::String { options: options })
        .placeholder("Укажите оппонента")
}

fn create_games_count_selector(games_count: i32, selected_value: Option<i64>) -> CreateSelectMenu {
    let options = (1..games_count + 1)
        .map(|number| {
            CreateSelectMenuOption::new(number.to_string(), number.to_string()).default_selection(
                if selected_value.is_some() && selected_value.unwrap() as i32 == number {
                    true
                }
                else {
                    false
                }
            )
        })
        .collect::<Vec<CreateSelectMenuOption>>();

    CreateSelectMenu::new("games_count_selector", poise::serenity_prelude::CreateSelectMenuKind::String { options: options })
        .placeholder("Укажите число игр")
}

pub async fn build_games_message(
    context: &Context,
    api: &ApiConnectionService,
    initial_message: u64
) -> Result<CreateInteractionResponseMessage, crate::Error> {
    let match_data = api.get_match(None, None, Some(initial_message.to_string())).await?;
    if let Some(existing_match) = match_data {
        let games_count = existing_match.games_count.unwrap();
        Ok(CreateInteractionResponseMessage::new().content("Игра 1").select_menu(build_race_selector(1)).select_menu(build_hero_selector(1)))
    }
    else {
        Err(crate::Error::from("Failed to build game message"))   
    }
}

fn build_race_selector(game_number: i64) -> CreateSelectMenu {
    CreateSelectMenu::new(
        format!("race_selector_{}", game_number),
        poise::serenity_prelude::CreateSelectMenuKind::String { options: vec![
            CreateSelectMenuOption::new("Орден порядка", "1"),
            CreateSelectMenuOption::new("Инферно", "2")
        ]})
        .placeholder(format!("Укажите свою расу в игре №{}", game_number))
}

fn build_hero_selector(game_number: i64) -> CreateSelectMenu {
    CreateSelectMenu::new(
        format!("hero_selector_{}", game_number),
        poise::serenity_prelude::CreateSelectMenuKind::String { options: vec![
            CreateSelectMenuOption::new("Орден порядка", "1"),
            CreateSelectMenuOption::new("Инферно", "2")
        ]})
        .placeholder(format!("Укажите своего героя в игре №{}", game_number))
}