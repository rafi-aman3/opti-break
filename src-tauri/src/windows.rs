use tauri::{AppHandle, Manager, WebviewUrl, WebviewWindowBuilder};

const WARNING_LABEL: &str = "warning";
const WARNING_W: f64 = 280.0;
const WARNING_H: f64 = 132.0;
const MARGIN: f64 = 20.0;

// Extra clearance from the bottom to avoid the Windows taskbar / macOS dock.
#[cfg(target_os = "windows")]
const BOTTOM_EXTRA: f64 = 48.0;
#[cfg(not(target_os = "windows"))]
const BOTTOM_EXTRA: f64 = 0.0;

pub fn show_warning(app: &AppHandle) -> tauri::Result<()> {
    // Reuse existing window if already open.
    if let Some(win) = app.get_webview_window(WARNING_LABEL) {
        win.show()?;
        return Ok(());
    }

    let (x, y) = bottom_right_pos(app, WARNING_W, WARNING_H);

    WebviewWindowBuilder::new(
        app,
        WARNING_LABEL,
        WebviewUrl::App(format!("index.html?route=warning").into()),
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

// ── Overlay stubs (implemented in M4) ────────────────────────────────────────

pub fn show_overlay(app: &AppHandle) {
    tracing::debug!("show_overlay: stub — implemented in M4");
    let _ = app;
}

pub fn close_overlay(app: &AppHandle) {
    tracing::debug!("close_overlay: stub — implemented in M4");
    let _ = app;
}

// ── Helpers ───────────────────────────────────────────────────────────────────

fn bottom_right_pos(app: &AppHandle, win_w: f64, win_h: f64) -> (f64, f64) {
    let fallback = (1600.0 - win_w - MARGIN, 900.0 - win_h - MARGIN - BOTTOM_EXTRA);

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
