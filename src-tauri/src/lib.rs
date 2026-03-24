mod commands;
mod paths;

pub fn run() {
    tauri::Builder::default()
        .plugin(tauri_plugin_shell::init())
        .invoke_handler(tauri::generate_handler![
            commands::settings::read_settings,
            commands::settings::write_settings,
            commands::stats::get_stats,
            commands::projects::list_projects,
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
