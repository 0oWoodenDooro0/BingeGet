// Prevents additional console window on Windows in release, DO NOT REMOVE!!
#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

use bingeget::commands::*;

fn main() {
    let client = reqwest::Client::builder()
        .pool_max_idle_per_host(10)
        .build()
        .expect("Failed to build HTTP client");

    tauri::Builder::default()
        .plugin(tauri_plugin_dialog::init())
        .manage(client)
        .invoke_handler(tauri::generate_handler![
            get_system_environment,
            parse_series_metadata,
            fetch_episode_m3u8,
            search_series,
            probe_episode_stream,
            start_download_queue,
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
