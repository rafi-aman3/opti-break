# CLAUDE.md

This file provides guidance to Claude Code (claude.ai/code) when working with code in this repository.

## What this is

A cross-platform desktop app enforcing the 20-20-20 eye care rule: every 20 minutes the user looks 20 feet away for 20 seconds. Full product spec is in `docs/SPEC.md`.

The core is built: timer state machine, multi-monitor break overlays, pre-break warning, ESC-to-skip, idle/sleep detection, active-hours scheduling, SQLite analytics, system tray with live countdown, autostart, and first-run onboarding.

## Commands

```bash
# Run full app in development (Vite dev server + Rust/native window)
pnpm tauri dev

# Type-check + Vite bundle (frontend only, no native window)
pnpm build

# Build production distributable
pnpm tauri build
```

No test suite or linter is configured. `pnpm build` runs `tsc -noEmit` as the primary static check.

## Architecture

Tauri 2.x: a Rust process hosts native OS windows containing a WebView. The frontend (React 19 + TypeScript, bundled by Vite on port 1420) communicates with the Rust backend via IPC.

### Backend modules (`src-tauri/src/`)

| Module | Role |
|---|---|
| `lib.rs` | Plugin/command registration, app setup, panic-log hook |
| `state.rs` | `AppState` — settings, settings path, timer channel, shared status snapshot, optional DB handle |
| `timer.rs` | Single tokio task owning the state machine (`Running` / `Warning` / `OnBreak` / `Paused`); driven by a `tokio::select!` over a deadline and a command channel |
| `windows.rs` | Builds the warning toast and per-monitor overlay windows; macOS NSPanel configuration |
| `tray.rs` | Tray menu, live countdown updater, icon-state switching |
| `shortcuts.rs` | Registers/unregisters ESC global shortcut for the duration of a warning/break |
| `idle.rs` | Polls `user-idle`; emits `IdlePause`/`IdleResume`. Also detects machine sleep via wall-clock skew |
| `schedule.rs` | Pure function: `is_within_active_hours(settings)` — handles overnight windows |
| `settings.rs` | Schema, JSON load/save, deep-merge patch |
| `db.rs` | rusqlite-backed `break_events` table; queries for day stats and aggregates |

### Frontend (`src/`)

A single Vite bundle serves multiple Tauri windows. Window role is selected by URL query string:

```
index.html?route=preferences | stats | warning | overlay | onboarding
```

`src/main.tsx` reads `?route=` and renders the matching component. The main window also listens for a `navigate` event from the tray to switch tabs in-place.

| Path | Role |
|---|---|
| `features/preferences/` | Preferences tabs (Timer, Break, Reminders, Schedule, General) |
| `features/stats/` | Stats page + `streak.ts` derivation |
| `features/warning/WarningToast.tsx` | Pre-break toast (bottom-right) |
| `features/overlay/BreakOverlay.tsx` | Full-screen dim overlay with countdown |
| `features/onboarding/Onboarding.tsx` | First-run flow |
| `lib/{settings,timer,stats}-client.ts` | Typed IPC wrappers + event listeners |
| `shared/settings.ts` | Zod schema + `Settings` type — mirrors the Rust `Settings` struct |

### IPC surface

Commands registered in `lib.rs`'s `invoke_handler!`:
- Settings: `get_settings`, `update_settings(patch)`, `reset_settings`
- Timer: `get_timer_status`, `timer_start`, `timer_pause`, `timer_pause_for(minutes)`, `timer_resume`, `take_break_now`, `skip_next_break`, `postpone_break`
- Analytics: `get_day_stats(days)`, `get_aggregates`

Events emitted by the backend: `timer:tick`, `timer:state_changed`, `timer:warning_started`, `timer:break_started`, `timer:break_ended`, `settings:updated`, `navigate`.

### Adding a Tauri command

1. Define in `src-tauri/src/lib.rs` with `#[tauri::command]`
2. Register in the `.invoke_handler(tauri::generate_handler![...])` call
3. Add a typed wrapper to the matching `src/lib/*-client.ts`

### Adding a Tauri plugin

1. Add to `src-tauri/Cargo.toml` dependencies
2. Register with `.plugin(plugin::init())` in the builder chain in `lib.rs`
3. Add the required permissions to `src-tauri/capabilities/default.json`

Currently registered: `opener`, `autostart`, `notification`, `global-shortcut`, `sql`. Note: `tauri-plugin-sql` is registered but the analytics layer uses `rusqlite` directly from Rust (`db.rs`) — the plugin is available for future frontend-side queries but is not in use today.

## Implementation notes (easy-to-miss details)

- **Timer is event-driven, not tick-driven.** The task sleeps until the next deadline (`earliest_deadline`) or the next command. UI countdowns must interpolate from the last published `TimerStatus` against wall-clock time — see `tray.rs` for the canonical pattern. Do not assume `timer:tick` fires every second.
- **macOS overlay = NSPanel, not NSWindow.** `configure_overlay_window` in `windows.rs` swaps the live class via `object_setClass`, then sets `NSWindowStyleMaskNonactivatingPanel`, `canJoinAllSpaces | fullScreenAuxiliary`, and `NSScreenSaverWindowLevel (1000)`. Without this, the overlay disappears in full-screen Spaces.
- **NSPanel `releasedWhenClosed` must be set to NO.** Both the frontend (`BreakOverlay`) and backend (`close_overlay`) call `close()` on break-end; without disabling the auto-release, the second call hits a freed pointer and SIGABRTs.
- **Build overlay hidden, configure, then show.** Setting `NSWindowCollectionBehavior` after the window is already on a Space anchors it to that Space only.
- **`macOSPrivateApi: true`** in `tauri.conf.json` is required for the NSPanel work and for `transparent: true` overlays.
- **ESC is a global shortcut**, not a window key handler. `shortcuts::register_esc` runs only while a warning or break is active and is unregistered on exit — do not rely on focus inside the overlay window.
- **Idle pause resets the timer** on resume (`make_running_fresh`), it does not resume the elapsed count. Also: if `actual_elapsed` between idle polls vastly exceeds `POLL`, the machine likely slept — `idle.rs` treats that as an immediate `IdlePause`.
- **Active hours are re-evaluated every tick** (`check_schedule` in the timer loop) and the `Running`/`Paused(OutsideHours)` transition is automatic. When active hours are enabled, the timer also wakes at least every 60 s to detect window boundaries.
- **Settings updates preserve in-flight Running deadlines** — see `SettingsUpdated` in `on_command` (it recomputes `next_warning_at` from the remaining time, doesn't restart the cycle). Other states are left as-is until they next transition.
- **Main window hides on close, never quits.** `WindowEvent::CloseRequested` calls `prevent_close()` + `hide()`. The only quit path is the tray "Quit" item.
- **First-run flow:** if `settings.general.onboarded` is false, the main window is shown on launch and `MainWindowGate` renders `<Onboarding>`. Otherwise the app starts hidden in the tray.
- **Analytics schema:** `break_events(id, timestamp TEXT (RFC3339 UTC), status, duration_actual, monitor_count)`. `status` ∈ {`completed`, `skipped`, `postponed`}. Day grouping happens in `query_day_stats` against local time so DST is handled correctly.
- **Panic log:** `set_hook` writes to `<app_data_dir>/logs/panic.log`. Useful when debugging silent crashes (especially the macOS NSPanel path).

## Window and app config

Window size, app identifier, bundle icons, and `macOSPrivateApi` are configured in `src-tauri/tauri.conf.json`. Tray icons (`tray-running.png`, `tray-paused.png`, `tray-warning.png` + @2x variants) live in `src-tauri/icons/` and are embedded via `include_bytes!`.
