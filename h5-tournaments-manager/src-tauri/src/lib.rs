use tournament::commands::LocalAppManager;

pub mod tournament;

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .manage(LocalAppManager::new())
        .plugin(tauri_plugin_shell::init())
        .invoke_handler(tauri::generate_handler![
            tournament::commands::load_tournaments,
            tournament::commands::load_races,
            tournament::commands::load_heroes,
            tournament::commands::load_matches,
            tournament::commands::load_games,
            tournament::commands::create_game,
            tournament::commands::update_game,
            tournament::commands::update_match,
            tournament::commands::load_games_for_stats
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}