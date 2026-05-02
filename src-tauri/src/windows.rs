use tauri::{AppHandle, Manager, WebviewUrl, WebviewWindowBuilder};

use crate::settings::{MonitorSelection, Settings};

// ── Warning toast ─────────────────────────────────────────────────────────────

const WARNING_LABEL: &str = "warning";
const WARNING_W: f64 = 280.0;
const WARNING_H: f64 = 132.0;
const MARGIN: f64 = 20.0;

#[cfg(target_os = "windows")]
const BOTTOM_EXTRA: f64 = 48.0;
#[cfg(not(target_os = "windows"))]
const BOTTOM_EXTRA: f64 = 0.0;

pub fn show_warning(app: &AppHandle) -> tauri::Result<()> {
    if let Some(win) = app.get_webview_window(WARNING_LABEL) {
        win.show()?;
        return Ok(());
    }

    let (x, y) = bottom_right_pos(app, WARNING_W, WARNING_H);

    WebviewWindowBuilder::new(
        app,
        WARNING_LABEL,
        WebviewUrl::App("index.html?route=warning".into()),
    )
    .decorations(false)
    .transparent(true)
    .always_on_top(true)
    .focused(false)
    .skip_taskbar(true)
    .resizable(false)
    .shadow(false)
    .inner_size(WARNING_W, WARNING_H)
    .position(x, y)
    .build()?;

    Ok(())
}

pub fn close_warning(app: &AppHandle) {
    if let Some(win) = app.get_webview_window(WARNING_LABEL) {
        win.close().ok();
    }
}

// ── Break overlay (multi-monitor) ─────────────────────────────────────────────

const OVERLAY_PREFIX: &str = "overlay_";

pub fn show_overlay(app: &AppHandle, settings: &Settings) -> tauri::Result<()> {
    // If any display is on a fullscreen Space, switch to its desktop Space
    // first — our plain-NSWindow overlay does not render on fullscreen Spaces.
    // Scheduled on the main thread before window creation so the Space switch
    // is enqueued ahead of the window-create tasks wry posts internally.
    #[cfg(target_os = "macos")]
    {
        let _ = app.run_on_main_thread(|| crate::spaces::leave_fullscreen_spaces());
    }

    let opacity = settings.break_.dim_opacity;

    let monitors: Vec<_> = match settings.break_.monitors {
        MonitorSelection::Primary => app
            .primary_monitor()?
            .map(|m| vec![m])
            .unwrap_or_default(),
        MonitorSelection::All => app.available_monitors()?,
    };

    // Determine which monitor is primary (for focus assignment).
    let primary_pos = app
        .primary_monitor()
        .ok()
        .flatten()
        .map(|m| *m.position());

    for (i, monitor) in monitors.iter().enumerate() {
        let label = format!("{OVERLAY_PREFIX}{i}");

        // Skip if this overlay is already open.
        if app.get_webview_window(&label).is_some() {
            continue;
        }

        let scale = monitor.scale_factor();
        let phys_pos = monitor.position();
        let phys_size = monitor.size();

        let lx = phys_pos.x as f64 / scale;
        let ly = phys_pos.y as f64 / scale;
        let lw = phys_size.width as f64 / scale;
        let lh = phys_size.height as f64 / scale;

        let is_primary = primary_pos
            .map(|p| p == *phys_pos)
            .unwrap_or(i == 0);

        let url = format!(
            "index.html?route=overlay&opacity={opacity}&duration={}",
            settings.timer.break_seconds
        );

        // Build hidden so we can set NSWindowCollectionBehavior before the window
        // is assigned to any Space. Showing it first and configuring later causes
        // macOS to anchor the window to the current Space only.
        let win = WebviewWindowBuilder::new(app, &label, WebviewUrl::App(url.into()))
            .decorations(false)
            .transparent(true)
            .always_on_top(true)
            .focused(false)
            .skip_taskbar(true)
            .resizable(false)
            .shadow(false)
            .visible(false)
            .position(lx, ly)
            .inner_size(lw, lh)
            .build()?;
        configure_overlay_window(&win, is_primary);
    }

    Ok(())
}

/// Force-close all overlay windows (used as a safety net after fade-out).
pub fn close_overlay(app: &AppHandle) {
    for (label, win) in app.webview_windows() {
        if label.starts_with(OVERLAY_PREFIX) {
            win.close().ok();
        }
    }
}

// ── Overlay window configuration ─────────────────────────────────────────────

fn configure_overlay_window(win: &tauri::WebviewWindow, focus: bool) {
    #[cfg(target_os = "macos")]
    {
        let win = win.clone();
        win.clone().run_on_main_thread(move || {
            use objc2::runtime::AnyObject;
            use objc2::msg_send;
            use raw_window_handle::{HasWindowHandle, RawWindowHandle};

            let Ok(handle) = win.window_handle() else { return };
            let RawWindowHandle::AppKit(h) = handle.as_raw() else { return };

            let ns_view = h.ns_view.as_ptr() as *mut AnyObject;
            unsafe {
                let ns_window: *mut AnyObject = msg_send![ns_view, window];
                if ns_window.is_null() {
                    return;
                }

                // canJoinAllSpaces (1) | stationary (1 << 4) | fullScreenAuxiliary (1 << 8)
                let _: () = msg_send![ns_window, setCollectionBehavior:
                    (1usize | (1usize << 4) | (1usize << 8))];

                // NSScreenSaverWindowLevel = 1000 — paints above full-screen app content.
                let _: () = msg_send![ns_window, setLevel: 1000i64];
            }

            win.show().ok();
            if focus {
                win.set_focus().ok();
            }
        })
        .ok();
    }
    #[cfg(not(target_os = "macos"))]
    {
        win.set_visible_on_all_workspaces(true).ok();
        win.show().ok();
        if focus {
            win.set_focus().ok();
        }
    }
}

// ── Helpers ───────────────────────────────────────────────────────────────────

fn bottom_right_pos(app: &AppHandle, win_w: f64, win_h: f64) -> (f64, f64) {
    let fallback = (
        1600.0 - win_w - MARGIN,
        900.0 - win_h - MARGIN - BOTTOM_EXTRA,
    );

    let Ok(Some(monitor)) = app.primary_monitor() else {
        return fallback;
    };

    let scale = monitor.scale_factor();
    let phys = monitor.size();
    let pos = monitor.position();

    let lw = phys.width as f64 / scale;
    let lh = phys.height as f64 / scale;
    let lx = pos.x as f64 / scale;
    let ly = pos.y as f64 / scale;

    (
        lx + lw - win_w - MARGIN,
        ly + lh - win_h - MARGIN - BOTTOM_EXTRA,
    )
}
