mod settings;
mod state;

use serde_json::Value;
use tauri::{Manager, State};
use tauri_plugin_autostart::MacosLauncher;

use crate::settings::Settings;
use crate::state::AppState;

#[tauri::command]
fn get_settings(state: State<'_, AppState>) -> Settings {
    state.settings.read().unwrap().clone()
}

#[tauri::command]
fn update_settings(state: State<'_, AppState>, patch: Value) -> Result<Settings, String> {
    let mut guard = state.settings.write().unwrap();
    let merged = settings::merge_patch(&guard, patch)?;
    settings::save(&state.settings_path, &merged).map_err(|e| format!("save failed: {e}"))?;
    *guard = merged.clone();
    Ok(merged)
}

#[tauri::command]
fn reset_settings(state: State<'_, AppState>) -> Result<Settings, String> {
    let defaults = Settings::default();
    settings::save(&state.settings_path, &defaults).map_err(|e| format!("save failed: {e}"))?;
    *state.settings.write().unwrap() = defaults.clone();
    Ok(defaults)
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .plugin(tauri_plugin_opener::init())
        .plugin(tauri_plugin_autostart::init(
            MacosLauncher::LaunchAgent,
            None,
        ))
        .plugin(tauri_plugin_notification::init())
        .plugin(tauri_plugin_global_shortcut::Builder::new().build())
        .plugin(tauri_plugin_sql::Builder::new().build())
        .setup(|app| {
            let app_data_dir = app
                .path()
                .app_data_dir()
                .expect("failed to resolve app_data_dir");
            std::fs::create_dir_all(&app_data_dir).ok();

            let settings_path = settings::settings_path(&app_data_dir);
            let loaded = settings::load(&settings_path);

            app.manage(AppState::new(loaded, settings_path));
            Ok(())
        })
        .invoke_handler(tauri::generate_handler![
            get_settings,
            update_settings,
            reset_settings
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
