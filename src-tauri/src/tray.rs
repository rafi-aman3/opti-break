use std::time::Duration;

use tauri::{
    menu::{MenuBuilder, MenuItemBuilder, PredefinedMenuItem, SubmenuBuilder},
    tray::TrayIconBuilder,
    AppHandle, Manager,
};

use crate::{
    state::AppState,
    timer::{SharedStatus, TimerCommand, StateKind},
};

pub fn setup(app: &AppHandle, timer_status: SharedStatus) -> tauri::Result<()> {
    // ── Menu items ────────────────────────────────────────────────────────────
    let status_item = MenuItemBuilder::with_id("status", "opti-break")
        .enabled(false)
        .build(app)?;

    let sep = || PredefinedMenuItem::separator(app);

    let pause_30 = MenuItemBuilder::with_id("pause30", "Pause for 30 min").build(app)?;
    let pause_60 = MenuItemBuilder::with_id("pause60", "Pause for 1 hour").build(app)?;
    let pause_tomorrow =
        MenuItemBuilder::with_id("pause_tomorrow", "Pause until tomorrow").build(app)?;
    let resume_item = MenuItemBuilder::with_id("resume", "Resume").build(app)?;

    let pause_sub = SubmenuBuilder::new(app, "Pause")
        .item(&pause_30)
        .item(&pause_60)
        .item(&pause_tomorrow)
        .build()?;

    let take_break = MenuItemBuilder::with_id("take_break", "Take a break now").build(app)?;
    let skip_break = MenuItemBuilder::with_id("skip_break", "Skip next break").build(app)?;

    let stats_item = MenuItemBuilder::with_id("stats", "Stats…").build(app)?;
    let prefs_item = MenuItemBuilder::with_id("preferences", "Preferences…").build(app)?;
    let quit_item = MenuItemBuilder::with_id("quit", "Quit").build(app)?;

    let menu = MenuBuilder::new(app)
        .item(&status_item)
        .item(&sep()?)
        .item(&pause_sub)
        .item(&resume_item)
        .item(&sep()?)
        .item(&take_break)
        .item(&skip_break)
        .item(&sep()?)
        .item(&stats_item)
        .item(&prefs_item)
        .item(&sep()?)
        .item(&quit_item)
        .build()?;

    // ── Tray icon ─────────────────────────────────────────────────────────────
    let icon = app
        .default_window_icon()
        .cloned()
        .expect("no default window icon configured");

    TrayIconBuilder::with_id("main")
        .icon(icon)
        .tooltip("opti-break")
        .menu(&menu)
        .show_menu_on_left_click(true)
        .on_menu_event(|app, event| handle_menu(app, event.id.as_ref()))
        .build(app)?;

    // ── Background tooltip + icon updater ─────────────────────────────────────
    let app_handle = app.clone();
    let status_item_clone = status_item.clone();
    let mut last_icon_state: Option<TrayIconState> = None;
    tokio::spawn(async move {
        loop {
            tokio::time::sleep(Duration::from_secs(10)).await;
            let snapshot = timer_status.read().unwrap().clone();
            let text = format_status_text(&snapshot);
            let _ = status_item_clone.set_text(&text);
            if let Some(tray) = app_handle.tray_by_id("main") {
                let _ = tray.set_tooltip(Some(text));
                let icon_state = tray_icon_state(&snapshot);
                if Some(icon_state) != last_icon_state {
                    if let Some(icon) = load_tray_icon(icon_state) {
                        let _ = tray.set_icon(Some(icon));
                    }
                    last_icon_state = Some(icon_state);
                }
            }
        }
    });

    Ok(())
}

fn handle_menu(app: &AppHandle, id: &str) {
    let state = app.state::<AppState>();

    match id {
        "pause30" => {
            let _ = state
                .timer_tx
                .try_send(TimerCommand::PauseFor(Some(Duration::from_secs(30 * 60))));
        }
        "pause60" => {
            let _ = state
                .timer_tx
                .try_send(TimerCommand::PauseFor(Some(Duration::from_secs(60 * 60))));
        }
        "pause_tomorrow" => {
            let mins = mins_until_midnight();
            let _ = state
                .timer_tx
                .try_send(TimerCommand::PauseFor(Some(Duration::from_secs(
                    mins * 60,
                ))));
        }
        "resume" => {
            let _ = state.timer_tx.try_send(TimerCommand::Resume);
        }
        "take_break" => {
            let _ = state.timer_tx.try_send(TimerCommand::TakeBreakNow);
        }
        "skip_break" => {
            let _ = state.timer_tx.try_send(TimerCommand::SkipNextBreak);
        }
        "stats" | "preferences" => {
            if let Some(win) = app.get_webview_window("main") {
                let route = if id == "stats" { "stats" } else { "preferences" };
                let url = format!("index.html?route={route}");
                win.navigate(url.parse().unwrap()).ok();
                win.show().ok();
                win.set_focus().ok();
            }
        }
        "quit" => {
            app.exit(0);
        }
        _ => {}
    }
}

fn format_status_text(status: &crate::timer::TimerStatus) -> String {
    match status.state {
        StateKind::Running => {
            if let Some(secs) = status.seconds_until_break {
                let m = secs.max(0) / 60;
                let s = secs.max(0) % 60;
                format!("opti-break · next break in {m}:{s:02}")
            } else {
                "opti-break".to_string()
            }
        }
        StateKind::Warning => "opti-break · break starting soon".to_string(),
        StateKind::OnBreak => "opti-break · eye break".to_string(),
        StateKind::Paused => "opti-break · paused".to_string(),
    }
}

// ── Tray icon state ───────────────────────────────────────────────────────────

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum TrayIconState {
    Running,
    Paused,
    Warning,
}

fn tray_icon_state(status: &crate::timer::TimerStatus) -> TrayIconState {
    match status.state {
        StateKind::Warning | StateKind::OnBreak => TrayIconState::Warning,
        StateKind::Paused => TrayIconState::Paused,
        StateKind::Running => {
            // Last-minute alert: ≤60 seconds to break
            if status.seconds_until_break.map(|s| s <= 60).unwrap_or(false) {
                TrayIconState::Warning
            } else {
                TrayIconState::Running
            }
        }
    }
}

fn load_tray_icon(state: TrayIconState) -> Option<tauri::image::Image<'static>> {
    let bytes: &[u8] = match state {
        TrayIconState::Running => include_bytes!("../icons/tray-running.png"),
        TrayIconState::Paused => include_bytes!("../icons/tray-paused.png"),
        TrayIconState::Warning => include_bytes!("../icons/tray-warning.png"),
    };
    let img = image::load_from_memory(bytes).ok()?;
    let rgba = img.to_rgba8();
    let (w, h) = rgba.dimensions();
    Some(tauri::image::Image::new_owned(rgba.into_raw(), w, h))
}

fn mins_until_midnight() -> u64 {
    use chrono::{Local, Timelike};
    let now = Local::now();
    let secs_since_midnight =
        now.hour() as u64 * 3600 + now.minute() as u64 * 60 + now.second() as u64;
    let secs_in_day = 24 * 3600u64;
    let remaining_secs = secs_in_day.saturating_sub(secs_since_midnight);
    // at least 1 min
    (remaining_secs / 60).max(1)
}
