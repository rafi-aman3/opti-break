mod settings;
mod state;
mod timer;

use std::sync::{Arc, RwLock};

use serde_json::Value;
use tauri::{Manager, State};
use tauri_plugin_autostart::MacosLauncher;

use crate::settings::Settings;
use crate::state::AppState;
use crate::timer::{PauseReason, TimerCommand, TimerStatus};

// ── Settings commands ─────────────────────────────────────────────────────────

#[tauri::command]
fn get_settings(state: State<'_, AppState>) -> Settings {
    state.settings.read().unwrap().clone()
}

#[tauri::command]
fn update_settings(state: State<'_, AppState>, patch: Value) -> Result<Settings, String> {
    let mut guard = state.settings.write().unwrap();
    let merged = settings::merge_patch(&guard, patch)?;
    settings::save(&state.settings_path, &merged).map_err(|e| format!("save failed: {e}"))?;
    // Notify timer of new settings.
    let _ = state
        .timer_tx
        .try_send(TimerCommand::SettingsUpdated(Box::new(merged.clone())));
    *guard = merged.clone();
    Ok(merged)
}

#[tauri::command]
fn reset_settings(state: State<'_, AppState>) -> Result<Settings, String> {
    let defaults = Settings::default();
    settings::save(&state.settings_path, &defaults).map_err(|e| format!("save failed: {e}"))?;
    let _ = state
        .timer_tx
        .try_send(TimerCommand::SettingsUpdated(Box::new(defaults.clone())));
    *state.settings.write().unwrap() = defaults.clone();
    Ok(defaults)
}

// ── Timer commands ─────────────────────────────────────────────────────────────

#[tauri::command]
fn get_timer_status(state: State<'_, AppState>) -> TimerStatus {
    state.status_snapshot()
}

#[tauri::command]
fn timer_start(state: State<'_, AppState>) {
    let _ = state.timer_tx.try_send(TimerCommand::Start);
}

#[tauri::command]
fn timer_pause(state: State<'_, AppState>) {
    let _ = state
        .timer_tx
        .try_send(TimerCommand::Pause(PauseReason::Manual));
}

#[tauri::command]
fn timer_resume(state: State<'_, AppState>) {
    let _ = state.timer_tx.try_send(TimerCommand::Resume);
}

#[tauri::command]
fn take_break_now(state: State<'_, AppState>) {
    let _ = state.timer_tx.try_send(TimerCommand::TakeBreakNow);
}

#[tauri::command]
fn skip_next_break(state: State<'_, AppState>) {
    let _ = state.timer_tx.try_send(TimerCommand::SkipNextBreak);
}

#[tauri::command]
fn postpone_break(state: State<'_, AppState>) {
    let _ = state.timer_tx.try_send(TimerCommand::PostponeBreak);
}

// ── App setup ─────────────────────────────────────────────────────────────────

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

            let timer_status = Arc::new(RwLock::new(TimerStatus::default()));
            let timer_tx = timer::spawn(app.handle().clone(), loaded.clone(), timer_status.clone());

            app.manage(AppState::new(loaded, settings_path, timer_tx, timer_status));
            Ok(())
        })
        .invoke_handler(tauri::generate_handler![
            // settings
            get_settings,
            update_settings,
            reset_settings,
            // timer
            get_timer_status,
            timer_start,
            timer_pause,
            timer_resume,
            take_break_now,
            skip_next_break,
            postpone_break,
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
