mod db;
mod idle;
mod schedule;
mod settings;
mod shortcuts;
#[cfg(target_os = "macos")]
mod spaces;
mod state;
mod timer;
mod tray;
mod windows;

use std::sync::{Arc, RwLock};
use std::time::Duration;

use serde_json::Value;
use tauri::{AppHandle, Emitter, Manager, State, WindowEvent};
use tauri_plugin_autostart::MacosLauncher;

use crate::settings::Settings;
use crate::state::AppState;
use crate::timer::{TimerCommand, TimerStatus};

// ── Settings commands ─────────────────────────────────────────────────────────

#[tauri::command]
fn get_settings(state: State<'_, AppState>) -> Settings {
    state.settings.read().unwrap().clone()
}

#[tauri::command]
fn update_settings(
    app: AppHandle,
    state: State<'_, AppState>,
    patch: Value,
) -> Result<Settings, String> {
    let mut guard = state.settings.write().unwrap();
    let merged = settings::merge_patch(&guard, patch)?;
    settings::save(&state.settings_path, &merged).map_err(|e| format!("save failed: {e}"))?;
    let _ = state
        .timer_tx
        .try_send(TimerCommand::SettingsUpdated(Box::new(merged.clone())));
    sync_autostart(&app, merged.general.autostart);
    let _ = app.emit("settings:updated", &merged);
    *guard = merged.clone();
    Ok(merged)
}

#[tauri::command]
fn reset_settings(app: AppHandle, state: State<'_, AppState>) -> Result<Settings, String> {
    let defaults = Settings::default();
    settings::save(&state.settings_path, &defaults).map_err(|e| format!("save failed: {e}"))?;
    let _ = state
        .timer_tx
        .try_send(TimerCommand::SettingsUpdated(Box::new(defaults.clone())));
    sync_autostart(&app, defaults.general.autostart);
    let _ = app.emit("settings:updated", &defaults);
    *state.settings.write().unwrap() = defaults.clone();
    Ok(defaults)
}

fn sync_autostart(app: &AppHandle, enabled: bool) {
    use tauri_plugin_autostart::ManagerExt;
    let al = app.autolaunch();
    if enabled {
        let _ = al.enable();
    } else {
        let _ = al.disable();
    }
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
    let _ = state.timer_tx.try_send(TimerCommand::PauseFor(None));
}

#[tauri::command]
fn timer_pause_for(state: State<'_, AppState>, minutes: u32) {
    let _ = state
        .timer_tx
        .try_send(TimerCommand::PauseFor(Some(Duration::from_secs(
            minutes as u64 * 60,
        ))));
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

// ── Analytics commands ─────────────────────────────────────────────────────────

#[tauri::command]
fn get_day_stats(state: State<'_, AppState>, days: u32) -> Result<Vec<db::DayStats>, String> {
    let Some(ref db) = state.db else {
        return Ok(vec![]);
    };
    db::query_day_stats(db, days).map_err(|e| format!("db error: {e}"))
}

#[tauri::command]
fn get_aggregates(state: State<'_, AppState>) -> Result<db::Aggregates, String> {
    let Some(ref db) = state.db else {
        return Ok(db::Aggregates::default());
    };
    db::query_aggregates(db).map_err(|e| format!("db error: {e}"))
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

            // Panic hook — writes to logs/panic.log so crashes are diagnosable.
            let logs_dir = app_data_dir.join("logs");
            std::fs::create_dir_all(&logs_dir).ok();
            let panic_log = logs_dir.join("panic.log");
            std::panic::set_hook(Box::new(move |info| {
                use std::io::Write;
                let ts = chrono::Local::now().to_rfc3339();
                let msg = format!("[{ts}] PANIC: {info}\n");
                if let Ok(mut f) = std::fs::OpenOptions::new()
                    .create(true)
                    .append(true)
                    .open(&panic_log)
                {
                    let _ = f.write_all(msg.as_bytes());
                }
                eprintln!("PANIC: {info}");
            }));

            let db = db::open(&app_data_dir)
                .map_err(|e| tracing::warn!("db: open failed: {e}"))
                .ok();

            let settings_path = settings::settings_path(&app_data_dir);
            let loaded = settings::load(&settings_path);

            {
                use tauri_plugin_autostart::ManagerExt;
                let al = app.handle().autolaunch();
                if loaded.general.autostart {
                    let _ = al.enable();
                } else {
                    let _ = al.disable();
                }
            }

            let timer_status = Arc::new(RwLock::new(TimerStatus::default()));
            let timer_tx = timer::spawn(
                app.handle().clone(),
                loaded.clone(),
                timer_status.clone(),
                db.clone(),
            );

            let onboarded = loaded.general.onboarded;

            app.manage(AppState::new(
                loaded,
                settings_path,
                timer_tx.clone(),
                timer_status.clone(),
                db,
            ));

            let settings_arc = app.state::<AppState>().settings_arc();
            idle::spawn(timer_tx, settings_arc);

            tray::setup(app.handle(), timer_status)?;

            let main_win = app.get_webview_window("main").expect("no main window");
            let win_clone = main_win.clone();
            main_win.on_window_event(move |event| {
                if let WindowEvent::CloseRequested { api, .. } = event {
                    api.prevent_close();
                    win_clone.hide().ok();
                }
            });

            // Show onboarding on first launch; otherwise start hidden.
            // Frontend (MainWindowGate) checks onboarded from settings and renders accordingly.
            if onboarded {
                app.get_webview_window("main").map(|w| w.hide().ok());
            } else {
                if let Some(win) = app.get_webview_window("main") {
                    win.show().ok();
                    win.center().ok();
                }
            }

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
            timer_pause_for,
            timer_resume,
            take_break_now,
            skip_next_break,
            postpone_break,
            // analytics
            get_day_stats,
            get_aggregates,
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
