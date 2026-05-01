use tauri::AppHandle;
use tauri_plugin_global_shortcut::{Code, GlobalShortcutExt, Shortcut, ShortcutState};
use tokio::sync::mpsc;

use crate::timer::TimerCommand;

fn esc_shortcut() -> Shortcut {
    Shortcut::new(None, Code::Escape)
}

/// Register ESC system-wide. Pressing it sends `TimerCommand::EscPressed`.
/// Safe to call when already registered (no-op).
pub fn register_esc(app: &AppHandle, timer_tx: mpsc::Sender<TimerCommand>) {
    let gs = app.global_shortcut();
    let esc = esc_shortcut();

    if gs.is_registered(esc) {
        return;
    }

    gs.on_shortcut(esc, move |_app, _shortcut, event| {
        if event.state() == ShortcutState::Pressed {
            let _ = timer_tx.try_send(TimerCommand::EscPressed);
        }
    })
    .ok();
}

/// Unregister the ESC global shortcut. Safe to call when not registered.
pub fn unregister_esc(app: &AppHandle) {
    let _ = app.global_shortcut().unregister(esc_shortcut());
}
