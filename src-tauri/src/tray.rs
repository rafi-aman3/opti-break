use std::time::{Duration, Instant};

use tauri::{
    menu::{MenuBuilder, MenuItemBuilder, PredefinedMenuItem, SubmenuBuilder},
    tray::TrayIconBuilder,
    AppHandle, Emitter, Manager,
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
        .icon_as_template(true)
        .tooltip("opti-break")
        .menu(&menu)
        .show_menu_on_left_click(true)
        .on_menu_event(|app, event| handle_menu(app, event.id.as_ref()))
        .build(app)?;

    // ── Background tooltip + icon updater ─────────────────────────────────────
    // The timer only writes TimerStatus when it wakes (e.g. at the warning
    // deadline, ~19 min away). To show a live countdown we anchor the last
    // published seconds value and subtract wall-clock elapsed time ourselves.
    let app_handle = app.clone();
    let status_item_clone = status_item.clone();
    let mut last_icon_state: Option<TrayIconState> = None;

    // Anchor: the timer-published values and the instant we first saw them.
    let mut anchor_state: Option<StateKind> = None;
    let mut anchor_until_break: Option<i64> = None;
    let mut anchor_remaining: Option<i64> = None;
    let mut anchored_at = Instant::now();

    tauri::async_runtime::spawn(async move {
        loop {
            let snapshot = timer_status.read().unwrap().clone();

            // Re-anchor whenever the timer publishes genuinely new values.
            if Some(snapshot.state) != anchor_state
                || snapshot.seconds_until_break != anchor_until_break
                || snapshot.seconds_remaining_in_break != anchor_remaining
            {
                anchored_at = Instant::now();
                anchor_state = Some(snapshot.state);
                anchor_until_break = snapshot.seconds_until_break;
                anchor_remaining = snapshot.seconds_remaining_in_break;
            }

            // Derive live seconds by subtracting wall-clock elapsed since anchor.
            let elapsed = anchored_at.elapsed().as_secs() as i64;
            let live_until_break = anchor_until_break.map(|a| (a - elapsed).max(0));
            let live_remaining = anchor_remaining.map(|a| (a - elapsed).max(0));

            let text = format_status_text(snapshot.state, live_until_break);
            let _ = status_item_clone.set_text(&text);
            if let Some(tray) = app_handle.tray_by_id("main") {
                let _ = tray.set_tooltip(Some(text));
                let _ = tray.set_title(Some(
                    format_tray_title(snapshot.state, live_until_break, live_remaining).as_str(),
                ));
                let icon_state = tray_icon_state(snapshot.state, live_until_break);
                if Some(icon_state) != last_icon_state {
                    if let Some(icon) = load_tray_icon(icon_state) {
                        let _ = tray.set_icon(Some(icon));
                        let _ = tray.set_icon_as_template(true);
                    }
                    last_icon_state = Some(icon_state);
                }
            }
            tokio::time::sleep(Duration::from_secs(1)).await;
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
                win.emit("navigate", route).ok();
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

fn format_status_text(state: StateKind, live_until_break: Option<i64>) -> String {
    match state {
        StateKind::Running => {
            if let Some(secs) = live_until_break {
                let m = secs / 60;
                let s = secs % 60;
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

fn format_tray_title(
    state: StateKind,
    live_until_break: Option<i64>,
    live_remaining: Option<i64>,
) -> String {
    match state {
        StateKind::Running | StateKind::Warning => {
            if let Some(secs) = live_until_break {
                format!("{}:{:02}", secs / 60, secs % 60)
            } else {
                String::new()
            }
        }
        StateKind::OnBreak => {
            if let Some(secs) = live_remaining {
                format!("{}s", secs)
            } else {
                String::new()
            }
        }
        StateKind::Paused => String::new(),
    }
}

// ── Tray icon state ───────────────────────────────────────────────────────────

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum TrayIconState {
    Running,
    Paused,
    Warning,
}

fn tray_icon_state(state: StateKind, live_until_break: Option<i64>) -> TrayIconState {
    match state {
        StateKind::Warning | StateKind::OnBreak => TrayIconState::Warning,
        StateKind::Paused => TrayIconState::Paused,
        StateKind::Running => {
            if live_until_break.map(|s| s <= 60).unwrap_or(false) {
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
    (remaining_secs / 60).max(1)
}
