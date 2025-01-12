// use std::str::FromStr;

// use human_regex::{digit, one_or_more, or, whitespace, zero_or_more, exactly};
// use poise::serenity_prelude::{futures::StreamExt, ChannelId, GuildId, Message};
// use reqwest::{Client, Error, Response, StatusCode};
// use tokio::sync::RwLockReadGuard;
// use uuid::Uuid;

// use crate::parser::utils::GameResult;

// use super::utils::{BargainsColor, BargainsData, BargainsType, Game, GameSideData, Hero, HeroType, Match, ParsingDataModel, Race, RaceType, Tournament};

// pub async fn create_tournament(
//     client: &RwLockReadGuard<'_, Client>,
//     server_id: u64,
//     channel_id: u64,
//     name: String 
// ) -> Result<String, ()> {
//     let tournament_creation_response = client
//         .post(format!("https://h5-tournaments-api.shuttleapp.rs/tournament?server_id={}&channel_id={}&name={}", server_id as i64, channel_id as i64, name))
//         .send()
//         .await;
//     match tournament_creation_response {
//         Ok(success) => {
//             let response_text = success.text().await.unwrap();
//             Ok(response_text)
//         },
//         Err(_failure) => {
//             Err(())
//         }
//     }
// }

// pub async fn try_get_tournament_by_channel(
//     client: &RwLockReadGuard<'_, Client>,
//     channel_id: u64,
// ) -> Result<Option<Tournament>, String> {
//     let tournament_response = client.get(format!("https://h5-tournaments-api.shuttleapp.rs/tournament?channel_id={}", channel_id))
//         .send()
//         .await;
//     let res = process_tournament_response(tournament_response).await;
//     match res {
//         Ok(possible_tournament) => {
//             Ok(possible_tournament)
//         },
//         Err(e) => {
//             Err(e)
//         }
//     }
// }

// pub async fn try_get_tournament_by_id(
//     client: &RwLockReadGuard<'_, Client>,
//     id: &String
// ) -> Result<Option<Tournament>, String> {
//     let tournament_response = client.get(format!("https://h5-tournaments-api.shuttleapp.rs/tournament?id={}", Uuid::from_str(id).unwrap()))
//         .send()
//         .await;
//     let res = process_tournament_response(tournament_response).await;
//     match res {
//         Ok(possible_tournament) => {
//             Ok(possible_tournament)
//         },
//         Err(e) => {
//             Err(e)
//         }
//     }
// }

// async fn process_tournament_response(
//     response: Result<Response, Error> 
// ) -> Result<Option<Tournament>, String> {
//     match response {
//         Ok(success) => {
//             match success.status() {
//                 StatusCode::OK => {
//                     let json: Result<Tournament, reqwest::Error> = success.json().await;
//                     match json {
//                         Ok(tournament) => {
//                             Ok(Some(tournament))
//                         },
//                         Err(e) => {
//                             Err("Failed to parse tournament model json".to_string())
//                         }
//                     }
//                 },
//                 StatusCode::NO_CONTENT => {
//                     Ok(None)
//                 },
//                 _=> {
//                     Err("Failed to fetch existing tournament".to_string())
//                 }
//             }
//         },
//         Err(failure) => {
//             Err(format!("Failed to request existing tournament: {}", failure.to_string()))
//         }
//     }
// }

// pub async fn parse_reports_messages(
//     context: &crate::Context<'_>,
//     tournament: &Tournament
// ) -> Result<(), ()> {
//     let client = context.data().client.read().await;

//     let parsing_data = get_data(&client).await;
//     match parsing_data {
//         Ok(data) => {
//             //tracing::info!("Ready to parse messages...");
//             let channel = ChannelId::new(tournament.channel_id as u64);
//             // let guild_channels = Guild::get(context, GuildId::new(tournament.server_id as u64)).await.unwrap().channels(context).await.unwrap();
//             // let guild_channel = guild_channels.get(&channel).unwrap();
//             // tracing::info!("Guild channel: {:?}", guild_channel);
//             // tracing::info!("Last message: {:?}", &message.unwrap().content);
//             // let messages = guild_channel.messages(context, GetMessages::new().after(MessageId::new(1254042077012955249))).await.unwrap();
//             // tracing::info!("Got messages: {:?}", &messages);
//             let mut messages = channel.messages_iter(context).boxed();
//             // tracing::info!("Messages: {:?}", &messages);
//             while let Some(possible_message) = messages.next().await {
//                 //tracing::info!("possible message: {:?}", &possible_message);
//                 match possible_message {
//                     Ok(message) => {
//                         tracing::info!("Got message: {}", &message.content);
//                         let match_structure = define_match_structure(&message.content).await;
//                         tracing::info!("Match structure got from it: {:?}", &match_structure);
//                         process_match_structure(&context, match_structure, &message, tournament, &data).await;
//                     },
//                     Err(e) => {
//                         tracing::info!("Error? {}", e.to_string())
//                     }
//                 }
//             }
//             Ok(())
//         },
//         Err(e) => {
//             context.reply(e).await.unwrap();
//             Err(())
//         }
//     }
// }



// async fn get_data(
//     client: &RwLockReadGuard<'_, Client>
// ) -> Result<ParsingDataModel, String> {
//     let races_res = get_races(client).await;
//     let heroes_res = get_heroes(client).await;

//     if races_res.is_ok() && heroes_res.is_ok() {
//         Ok(ParsingDataModel { 
//             races: races_res.unwrap(), 
//             heroes: heroes_res.unwrap()
//         })
//     }
//     else {
//         let mut error = String::from("Errors occured while fetching data for parsing: ");
//         if let Some(err) = races_res.err() {
//             error += &err;
//         }
//         if let Some(err) = heroes_res.err() {
//             error += &err
//         }
//         Err(error)
//     }
// }

// async fn get_races(
//     client: &RwLockReadGuard<'_, Client>
// ) -> Result<Vec<Race>, String> {
//     let response = client.get("https://h5-tournaments-api.shuttleapp.rs/races")
//         .send()
//         .await;
//     match response {
//         Ok(success) => {
//             let json: Result<Vec<Race>, Error> = success.json().await;
//             match json {
//                 Ok(races) => {
//                     tracing::info!("Got races: {:?}", &races);
//                     Ok(races)
//                 },
//                 Err(json_error) => {
//                     Err(format!("Failed to parse races json: {}", json_error.to_string()))
//                 }
//             }
//         },
//         Err(failure) => {
//             Err(format!("Failed to send request to get races: {}", failure.to_string()))
//         }
//     }
// }

// async fn get_heroes(
//     client: &RwLockReadGuard<'_, Client>
// ) -> Result<Vec<Hero>, String> {
//     let response = client.get("https://h5-tournaments-api.shuttleapp.rs/heroes")
//         .send()
//         .await;
//     match response {
//         Ok(success) => {
//             let json: Result<Vec<Hero>, Error> = success.json().await;
//             match json {
//                 Ok(heroes) => {
//                     Ok(heroes)
//                 },
//                 Err(json_error) => {
//                     Err(format!("Failed to parse heroes json: {}", json_error.to_string()))
//                 }
//             }
//         },
//         Err(failure) => {
//             Err(format!("Failed to send request to get heroes: {}", failure.to_string()))
//         }
//     }
// }

// #[derive(Debug, Default)]
// struct MatchStructure {
//     pub players_string: String,
//     pub games_strings: Vec<String>,
//     pub bargains_string: Vec<String>
// }

// async fn define_match_structure(
//     message: &String,
// ) -> MatchStructure {
//     let mut parts_count = 0;
//     let mut match_base_data = MatchStructure::default();
//     message.split("\n")
//         .filter(|s| {
//             s.len() > 0
//         })
//         .for_each(|s| {
//             parts_count += 1;
//             if parts_count == 1 {
//                 match_base_data.players_string = s.to_string();
//             }
//             else {
//                 if parts_count % 2 == 0 {
//                     match_base_data.games_strings.push(s.to_string());
//                 }
//                 else {
//                     match_base_data.bargains_string.push(s.to_string());
//                 }
//             }
//         });
//     match_base_data
// }

// async fn process_match_structure(
//     context: &crate::Context<'_>,
//     match_structure: MatchStructure,
//     message: &Message,
//     tournament: &Tournament,
//     data: &ParsingDataModel
// ) {
//     let mut new_match = Match::default();
//     let players: Vec<&str> = match_structure.players_string.split("vs")
//         .map(|s| s.trim())
//         .collect();
//     let client = context.data().client.read().await;
//     // let mut nickname = String::new();
//     // if let Some(nick) = message.author.nick_in(context, GuildId::new(tournament.server_id as u64)).await {
//     //     nickname = nick;
//     // }
//     // else {
//     //     nickname = message.author.name.clone();
//     // }
//     let match_creation_response = client
//         .post(format!("https://h5-tournaments-api.shuttleapp.rs/match?message={}&tournament={}&first_player={}&second_player={}", 
//             message.id.get(),
//             tournament.id, 
//             players[0],
//             players[1]))
//         .send()
//         .await;
//     match match_creation_response {
//         Ok(success) => {
//             let json: Result<Match, Error> = success.json().await;
//             match json {
//                 Ok(created_match) => {
//                     tracing::info!("Match created: {:?}", &created_match);
//                     new_match = created_match;
//                 },
//                 Err(json_error) => {
//                     tracing::info!("Error parsing match json: {}", json_error.to_string())
//                 }
//             }
//         },
//         Err(failure) => {}
//     }

//     let games_count = match_structure.games_strings.len();
//     for game_number in 0..games_count {
//         if match_structure.games_strings.get(game_number).is_some() && match_structure.bargains_string.get(game_number).is_some() {
//             if let Some(game) = process_game_info(
//                 &match_structure.games_strings[game_number], 
//                 &match_structure.bargains_string[game_number].to_lowercase(), 
//                 &new_match.id,
//                 data
//             ).await {
//                 tracing::info!("Game detected: {:?}", &game);
//                 create_game(&client, &game).await.unwrap();
//             }
//             else {
//                 break;
//             }
//         }
//         else {
//             break;
//         }
//     }

// }


// async fn process_game_info(
//     game_string: &String,
//     bargains_string: &String,
//     match_id: &Uuid,
//     data: &ParsingDataModel
// ) -> Option<Game> {
//     let sides_data: Vec<&str> = game_string.split_inclusive(|c| c == '>' || c == '<')
//         .map(|s| s.trim())
//         .collect();
//     //tracing::info!("sides data: {:?}", &sides_data);
//     if sides_data.len() == 1 {
//         //tracing::info!("No game detected in string {}", game_string);
//         return None;
//     }
//     let first_player_game_data = process_side_info(sides_data[0], data);
//     let second_player_game_data = process_side_info(sides_data[1], data);
//     let bargains_data = process_bargains_info(bargains_string);

//     let mut game_result = GameResult::NotDetected;

//     if let Some(_) = sides_data.iter().find(|d| d.contains(">")) {
//         game_result = GameResult::FirstPlayerWon;
//     }
//     else {
//         game_result = GameResult::SecondPlayerWon;
//     }

//     Some(Game {
//         id: uuid::Uuid::new_v4(),
//         match_id: *match_id,
//         first_player_race: first_player_game_data.race,
//         first_player_hero: first_player_game_data.hero,
//         second_player_hero: second_player_game_data.hero,
//         second_player_race: second_player_game_data.race,
//         bargains_color: bargains_data.color,
//         bargains_amount: bargains_data.amount,
//         result: game_result
//     })
// }

// fn process_side_info(
//     side_string: &str,
//     data: &ParsingDataModel
// ) -> GameSideData {
//     let mut side_data = GameSideData {
//         race: RaceType::NotDetected,
//         hero: HeroType::NotDetected
//     };

//     let side_info: Vec<&str> = side_string.split(|c| c == ')' || c == '(')
//         .map(|s| s.trim())
//         .collect();

//     if side_info.len() == 1 {
//         return side_data;
//     }

//     let race_info = side_info[0].to_lowercase();
//     let hero_info = side_info[1].to_lowercase();

//     if let Some(race) = data.races.iter()
//         .find(|r| r.name_variants.variants.iter().any(|v| v.contains(&race_info))) {
//         side_data.race = race.id;
//     } 

//     if let Some(hero) =  data.heroes.iter()
//         .find(|h| h.name_variants.variants.iter().any(|v| v.contains(&hero_info))) {
//         side_data.hero = hero.id;
//     }

//     side_data
// }


// fn process_bargains_info(
//     bargains_string: &String
// ) -> BargainsData {
//     let bargains_types = vec![
//         BargainsType {
//             color: BargainsColor::ColorRed,
//             actual_name: String::from("Красный"),
//             variants: vec![String::from("красный")]
//         },
//         BargainsType {
//             color: BargainsColor::ColorBlue,
//             actual_name: String::from("Синий"),
//             variants: vec![String::from("синий")]
//         }
//     ];

//     let mut bargains_data = BargainsData {
//         color: BargainsColor::NotDetected,
//         amount: 0
//     };

//     // life saver
//     let readable_regex = exactly(1, or(&['+', '-'])) + zero_or_more(whitespace()) + one_or_more(digit());
//     let amount_regex = readable_regex.to_regex();
//     if let Some(capture) = amount_regex.find(bargains_string) {
//         if let Ok(amount) = capture.as_str().replace(" ", "").parse::<i32>() {
//             bargains_data.amount = amount;
//         }
//     }

//     if let Some(bargains_type) = bargains_types.iter()
//         .find(|t| {
//                 t.variants.iter()
//                     .any(|v| {
//                         bargains_string.contains(v)
//                     })
//             }) {
//         bargains_data.color = bargains_type.color;
//     }

//     bargains_data

// }

// async fn create_game(
//     client: &RwLockReadGuard<'_, Client>,
//     game: &Game
// ) -> Result<(), ()> {
//     let response = client.post("https://h5-tournaments-api.shuttleapp.rs/game")
//         .json(game)
//         .send()
//         .await;
//     match response {
//         Ok(success) => {
//             //tracing::info!("Game created successfully");
//             Ok(())
//         },
//         Err(failure) => {
//             tracing::error!("Failed to send game creation response: {}", failure.to_string());
//             Err(())
//         }
//     }
// }