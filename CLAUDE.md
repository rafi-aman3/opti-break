# CLAUDE.md

This file provides guidance to Claude Code (claude.ai/code) when working with code in this repository.

## What this is

A cross-platform desktop app enforcing the 20-20-20 eye care rule: every 20 minutes the user looks 20 feet away for 20 seconds. Full product spec is in `docs/SPEC.md`. The codebase is currently the bare Tauri scaffold — all app logic (timer, overlays, tray menu, analytics) is yet to be built.

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

### Adding a Tauri command (IPC)

1. Define in `src-tauri/src/lib.rs` with `#[tauri::command]`
2. Register in the `.invoke_handler(tauri::generate_handler![...])` call in the same file
3. Call from frontend: `invoke("command_name", { arg })` via `@tauri-apps/api`

### Adding a Tauri plugin

1. Add to `src-tauri/Cargo.toml` dependencies
2. Register with `.plugin(plugin::init())` in the builder chain in `lib.rs`
3. Add the required permissions to `src-tauri/capabilities/default.json`

### Planned plugins (per spec)

| Plugin | Purpose |
|---|---|
| `tauri-plugin-autostart` | Launch on system boot |
| `tauri-plugin-notification` | Fallback OS notifications |
| `tauri-plugin-global-shortcut` | ESC key during break overlays |
| `tauri-plugin-sql` | SQLite analytics store |
| `user-idle` (Rust crate) | Cross-platform idle detection |

### Key implementation details from spec

- **Break overlay**: one borderless always-on-top Tauri window *per monitor*, created on break start and destroyed on break end. Opacity 78% (`rgba(0,0,0,0.78)`), configurable 50–95%.
- **Pre-break warning**: custom Tauri window (not OS notification) pinned bottom-right, appears 10 s before break.
- **ESC handling**: `tauri-plugin-global-shortcut` registers ESC system-wide for the break duration — do not rely on focus inside the overlay window.
- **Timer pause**: idle after 3 min pauses the timer; on return, timer resets to zero (not resumed).
- **Analytics schema**: `break_events(id, timestamp, status, duration_actual, monitor_count)` — local SQLite only, no cloud sync.
- **Tray menu**: primary UI surface — status, pause options, actions, stats/prefs/quit.
- **Settings**: flat JSON object (no profiles); see spec §Customisation for full schema.
- **Browser extension**: companion product using `@crxjs/vite-plugin`, MV3 — timer must use `chrome.alarms`, not `setInterval`.

### Window and app config

Window size, app identifier, and bundle icons are configured in `src-tauri/tauri.conf.json`.
