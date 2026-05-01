# opti-break — 7-milestone build plan

## Context

The repo is currently a bare Tauri 2.x + React 19 scaffold (default `greet` command, default Vite/React UI). The product is fully specced in `docs/SPEC.md`: a 20-20-20 eye-care desktop app — tray-resident, multi-monitor dim overlays, custom warning toast, idle pause, autostart, SQLite analytics, streaks, preferences UI. The goal is to ship a friends-testable build (unsigned `.dmg` + Windows `.msi`) that delivers v1 end-to-end, sliced into exactly 7 milestones where each one is independently demoable.

Browser extension, code signing, auto-updater, profiles, and per-app tracking are explicitly cut from this plan and deferred to v2 (rationale: scope, value-per-week of work, and user-facing positioning — see spec §Out of scope). The browser extension will live in a separate repo when v2 begins.

## Decisions to lock in before M1

- **App name**: `opti-break` (matches repo + bundle id `com.rafiaman.opti-break`); spec's "Eye care" was a placeholder.
- **Frontend stack**: keep React (spec accepts either Svelte or React). Add Tailwind for styling.
- **Theme**: light/dark/system supported from M6 onward.
- **Distribution (M7)**: GitHub Releases, unsigned builds, document Gatekeeper / SmartScreen workarounds for friends.
- **Updater**: deferred to v2 — do NOT wire `tauri-plugin-updater` (would prompt warnings on unsigned builds).
- **Browser extension**: deferred to v2, separate repo.

---

## M1 — Foundation, settings store, IPC plumbing

**Headline**: Replace the scaffold with a real shell — plugins wired, settings persisted, dev window cleaned.

**Scope**:
- Strip greet command + scaffold UI; introduce a single React entry routed by `?route=preferences|warning|overlay`.
- Add Rust deps: `tauri-plugin-autostart`, `tauri-plugin-notification`, `tauri-plugin-global-shortcut`, `tauri-plugin-sql` (sqlite feature), `user-idle`, `tokio` (rt + macros), `chrono`.
- Add JS deps: matching `@tauri-apps/plugin-*` packages, `tailwindcss`, `zod`, `zustand`.
- Implement settings module (Rust source of truth): persist flat JSON per spec §Customisation to `app_data_dir()/settings.json`. Commands: `get_settings`, `update_settings(patch)`, `reset_settings`. Mirror the schema with Zod on the frontend.
- Tauri managed state (`AppState`) holding settings + future timer handle.
- Update `tauri.conf.json`: keep main window visible during M1–M2 for dev (will hide in M3 once tray exists). Add `LSUIElement: true` (mac) bundle config for agent-mode behavior at M3+.
- Update `capabilities/default.json` to grant defaults for the new plugins.

**Demo**: `pnpm tauri dev` opens an empty Preferences window; `invoke('get_settings')` returns defaults; round-trip persistence works after restart.

**Key files (new/changed)**:
- `src-tauri/Cargo.toml`
- `src-tauri/src/lib.rs`
- `src-tauri/src/settings.rs` (new)
- `src-tauri/src/state.rs` (new)
- `src-tauri/tauri.conf.json`
- `src-tauri/capabilities/default.json`
- `package.json`
- `tailwind.config.ts`, `postcss.config.js` (new)
- `src/shared/settings.ts` (new — typed schema)
- `src/main.tsx` (route by query param)

**Risks**: tauri-plugin capability quirks; on macOS, `global-shortcut` registration prompt only fires later when used.

---

## M2 — Headless timer engine + debug panel

**Headline**: The 20-min/20-sec loop runs in Rust, emits events; a temporary debug panel proves it.

**Scope**:
- State machine in `src-tauri/src/timer.rs`: `Running { next_break_at }`, `Paused { reason }`, `Warning { ends_at }`, `OnBreak { ends_at, postponed_count }`.
- Single `tokio::task` driven by absolute deadlines (so settings changes mid-cycle re-evaluate cleanly).
- Commands: `timer_start`, `timer_pause(reason)`, `timer_resume`, `take_break_now`, `skip_next_break`, `postpone_break`, `get_timer_status`.
- Events: `timer:tick`, `timer:warning_started`, `timer:break_started`, `timer:break_ended`, `timer:state_changed`.
- Honor `interval_minutes`, `break_seconds`, `warning_seconds` from settings live.
- Throwaway "Debug" panel in Preferences with current state, next-break ETA, and one button per command. Stays through M4, removed in M6.

**Demo**: with interval 1 min / break 10 s / warning 5 s, debug panel ticks down through warning → break → resume; postpone shifts ETA by 5 min; pause/resume works.

**Key files**:
- `src-tauri/src/timer.rs` (new)
- `src-tauri/src/lib.rs` (register commands, spawn task)
- `src/features/debug/DebugPanel.tsx` (new, throwaway)
- `src/lib/timer-client.ts` (new — typed invoke + event wrappers)

**Risks**: only Rust mutates timer state; React must stay event-driven (no parallel JS countdown). Use `tokio::time::Instant` deadlines, not `sleep(duration)`.

---

## M3 — Tray menu + warning toast window

**Headline**: App feels real — tray drives everything, warning toast appears 10 s before each break.

**Scope**:
- System tray via Tauri 2 core API: icon + tooltip with time-to-next-break, full menu per spec §Tray menu (status / pause submenu / actions / footer). Stats and Preferences items just show + focus the prefs window for now.
- Tray icon swaps state: running / paused / last-minute. Three monochrome PNGs (template images on macOS).
- Warning toast: separate Tauri window `?route=warning`, borderless, transparent, always-on-top, no-focus, ~260×120, positioned bottom-right of primary monitor. Created on `timer:warning_started`, destroyed on warning end / break start. UI: status dot, countdown, subtext, "Skip" → `skip_next_break`, "+5 min" → `postpone_break`. CSS fade-in 200 ms.
- Pause submenu wires to `timer_pause` with reasons `Manual30`, `Manual60`, `UntilTomorrow` (computed against local midnight).
- Quit calls `app.exit(0)`. CloseRequested on prefs window → hide instead of quit. Main window now hidden by default; only opened from tray.
- New capability file `capabilities/warning.json`.

**Demo**: tray menu visible on macOS + Windows; live "Next break in 0:42"; warning toast slides in 5 s before break; "+5 min" postpones; "Pause for 30 min" works and tray icon flips to paused; Quit fully exits.

**Key files**:
- `src-tauri/src/tray.rs` (new)
- `src-tauri/src/windows.rs` (new — helpers for spawning warning + overlay windows)
- `src-tauri/tauri.conf.json` (tray + window definitions)
- `src-tauri/capabilities/warning.json` (new)
- `src-tauri/icons/tray-{running,paused,warning}.png` (placeholders)
- `src/features/warning/WarningToast.tsx` (new)

**Risks**: macOS template-icon tinting (use 22×22 monochrome with `@2x`); Windows left-click vs right-click menu semantics (`show_menu_on_left_click(true)`); toast must not steal focus.

**Decision needed by M3**: tray icon art. Placeholder OK if final art lands by M7.

---

## M4 — Multi-monitor break overlay + global ESC

**Headline**: The signature dim-overlay break experience works across every connected monitor.

**Scope**:
- On `timer:break_started`: enumerate `app.available_monitors()`, spawn one overlay window per monitor sized to that monitor's bounds. Config: decorations off, transparent, always-on-top, skip-taskbar, non-resizable, shadowless. Focus only the primary monitor's overlay.
- React `/overlay` route: `rgba(0,0,0,opacity)` backdrop with 400 ms CSS fade-in / 250 ms fade-out. Centered: "Eye break" label, large `tabular-nums` countdown driven by `timer:tick`, "Look at something 20 feet away.", thin progress bar, "Press esc to skip" hint.
- Global ESC via `tauri-plugin-global-shortcut`: register on warning-started AND break-started, unregister on break-ended. Press ends overlay early → Rust records partial duration (M5) and triggers fade-out.
- Honor `dim_opacity` (50–95%) and `monitors` ("all" | "primary").
- New capability `capabilities/overlay.json`.

**Demo**: with short interval, warning fades into overlay across every monitor at 78% dim; ESC ends it; setting `monitors: "primary"` only spawns one; disconnect/reconnect a display between breaks → next break enumerates correctly.

**Key files**:
- `src-tauri/src/windows.rs` (overlay spawning + teardown)
- `src-tauri/src/shortcuts.rs` (new — ESC register/unregister bound to break lifecycle)
- `src-tauri/capabilities/overlay.json` (new)
- `src/features/overlay/BreakOverlay.tsx` (new)

**Risks**:
- macOS transparency requires `rgba(0,0,0,0)` body background and careful titlebar config; test under both light and dark system theme.
- macOS Spaces: a non-fullscreen always-on-top window does NOT cover an active true-fullscreen app (Keynote present mode). Spec accepts this; document it.
- Linux/Wayland multi-monitor positioning is unreliable — best-effort.
- Always destroy overlay windows after fade-out; never pool. Run a 10-cycle stress test to confirm no leaked windows.
- Deregister ESC the moment break ends so it doesn't get swallowed in the user's editor.

---

## M5 — SQLite analytics + idle pause + autostart + sound

**Headline**: Every break is recorded, the timer pauses when you walk away, the app launches at boot.

**Scope**:
- `tauri-plugin-sql` with a single migration creating `break_events` per spec §Analytics (plus the timestamp index).
- `src-tauri/src/db.rs`: `record_break_event(status, duration_actual, monitor_count)` called from timer transitions:
  - natural break end → `completed`, duration = `break_seconds`
  - ESC during break → `skipped`, duration = elapsed seconds
  - ESC during warning OR `skip_next_break` → `skipped`, duration = 0
  - `+5 min` → `postponed`, duration = 0 (one event per postpone press)
- Idle detection: `user-idle` polled every 5 s in a `tokio::task`. Cross `idle_threshold_minutes` while `Running` → `Paused { Idle }`. On return → fresh full interval (per spec: "timer resets to zero"). Detect sleep/wake via large elapsed jumps and treat as idle reset.
- Autostart: `tauri-plugin-autostart` enabled by default; toggle reflects setting changes; respect existing OS state on first run.
- Sound: bundle `assets/chime_soft.mp3`; play via Web Audio in the warning window when `sound_enabled = true`. System mute is respected automatically by audio output; document that DND respect is OS-driven.
- Verify zero network code in the build.

**Demo**:
- Walk away 3 min → tray shows paused; return → countdown restarts from a full interval.
- Run several short cycles; SQLite browser shows correct rows for completed / skipped / postponed.
- Reboot → app launches automatically; tray appears.
- Toggle sound on → next warning plays the chime.

**Key files**:
- `src-tauri/src/db.rs` (new)
- `src-tauri/src/idle.rs` (new)
- `src-tauri/src/timer.rs` (record events at transitions)
- `src-tauri/src/lib.rs` (sql plugin init w/ migrations, autostart init)
- `src-tauri/resources/chime_soft.mp3` (new)
- `src/features/warning/WarningToast.tsx` (audio playback)

**Risks**: pin migration `version: 1` and never edit it once shared (data wipe risk). Windows `GetLastInputInfo` misreports in RDP sessions — known limitation. macOS LSUIElement removes dock icon — ensure prefs accessible only via tray.

---

## M6 — Stats screen + Preferences UI + active hours

**Headline**: The visible app is real — full preferences, a stats dashboard, and active-hours scheduling.

**Scope**:
- Two routes inside the prefs window: `/preferences` and `/stats`. Tray "Stats…" / "Preferences…" navigate + show + focus. CloseRequested → hide.
- **Preferences UI** (Tailwind, no component lib): grouped sections matching settings schema.
  - Timer: interval slider 5–60, duration 10–60.
  - Break: dim opacity 50–95 %, monitors radio (all / primary).
  - Reminders: warning seconds 5–30, sound toggle, sound preview button.
  - Schedule: idle threshold 1–10 min, active-hours toggle + day picker + start/end time picker.
  - General: autostart, theme (light/dark/system), streaks toggle.
  - Footer: "Reset to defaults" (does NOT clear analytics).
  - Each change → `update_settings(patch)`. Theme + active-hours apply immediately.
- **Active hours engine** in `src-tauri/src/schedule.rs`: when enabled, timer enters `Paused { OutsideHours }` outside the window; resumes at next active block start. Re-evaluated on every tick + on settings change. Local time, `chrono::Local`.
- **Stats screen**: pure read-time computation.
  - Today's compliance: `completed / scheduled` (scheduled estimated from idle-gap heuristic).
  - Current streak: walk back day-by-day applying ≥50 % completed AND ≥30 min active screen rule. Cap 365 days.
  - This week's total: count of `completed` in last 7 days.
  - 7-day bar chart: hand-rolled SVG, one `<rect>` per day (no chart lib).
  - All-time footer: total breaks, longest streak, total looking-far-away hours = `Σ duration_actual where status=completed / 3600`.
  - Hide streak card when `streaks_enabled = false`.
- Theme: Tailwind class strategy + `prefers-color-scheme` listener, broadcast `settings:updated` event so all open windows re-pick.
- Remove the M2 debug panel.

**Demo**: change every setting and see effects (opacity 90 % → next overlay darker; active-hours window starting in 30 s → timer pauses on close); after running short cycles for ~30 min, stats page shows today's compliance, populated bar chart, hidden streak card when toggled off.

**Key files**:
- `src/features/preferences/PreferencesPage.tsx` (new)
- `src/features/preferences/sections/{Timer,Break,Reminders,Schedule,General}.tsx` (new)
- `src/features/stats/StatsPage.tsx` (new)
- `src/features/stats/streak.ts` (new — pure read-time logic)
- `src-tauri/src/schedule.rs` (new)
- `src-tauri/src/db.rs` (add `query_events_range`, `query_aggregates`)
- `src-tauri/src/timer.rs` (consult schedule on tick + on settings change)

**Risks**: time zones / DST → use `chrono::Local` consistently for both active hours and streak day boundaries. "Reset to defaults" must NOT touch the analytics DB.

---

## M7 — Polish, packaging, friends-testable installers

**Headline**: Ship `.dmg` and `.msi` installers a friend can double-click.

**Scope**:
- Polish pass: copy review, animation smoothness, microcopy, empty-states ("No data yet — your first break is in 18:42").
- Crash safety: panic hook → `app_data_dir/logs/`. Timer/idle/sql tasks log + restart, never panic the process.
- Final tray icons (running / paused / last-minute) — replace M3 placeholders.
- Final app icon — regenerate `.icns` and `.ico` via `pnpm tauri icon path/to/source.png`.
- First-run onboarding dialog: explain the app, ask for autostart consent, let user pick interval. Persist `onboarded: true`.
- Build configuration:
  - macOS: `bundle.macOS` with `LSUIElement: true`, category `public.app-category.healthcare-fitness`. DMG target.
  - Windows: NSIS installer, single-language English, embedded WebView2 bootstrapper (`webview2InstallMode: "downloadBootstrapper"`).
  - Both unsigned. Document Gatekeeper (`xattr -d com.apple.quarantine`) and SmartScreen ("More info → Run anyway") in `docs/INSTALL.md`.
- GitHub Actions release workflow: matrix `macos-latest` (universal binary) + `windows-latest` (x64), uploads draft release artifacts.
- README rewrite: install instructions, what to expect, known limitations (unsigned, no Linux focus, no fullscreen-app detection).

**Demo**: a friend on macOS downloads the DMG, drags to Applications, opens (clicks past Gatekeeper), gets the tray, accepts autostart, sees their first break 20 min later. Same flow on Windows with the MSI.

**Key files**:
- `src-tauri/tauri.conf.json` (final bundle config)
- `src-tauri/icons/*` (regenerated)
- `src/features/onboarding/Onboarding.tsx` (new)
- `docs/INSTALL.md` (new)
- `.github/workflows/release.yml` (new)
- `README.md` (rewritten)

**Risks**: building Windows from a Mac requires CI or a VM (CI is cleaner). Universal mac binary needs both `aarch64-apple-darwin` and `x86_64-apple-darwin` toolchains. Do NOT enable updater plugin (unsigned builds will warn).

---

## Critical files (architectural backbone)

- `src-tauri/src/lib.rs` — wires plugins, registers commands, spawns timer/idle tasks, builds tray
- `src-tauri/src/timer.rs` — single source of truth for timer state (M2)
- `src-tauri/src/windows.rs` — warning + overlay window spawn/teardown (M3, M4)
- `src-tauri/src/db.rs` — analytics persistence + queries (M5, M6)
- `src-tauri/tauri.conf.json` — bundle, capabilities glue, window templates
- `src-tauri/Cargo.toml` — single place where all backend deps land in M1

## Deferred to v2 (explicitly excluded)

Profiles · per-app tracking / screen-time analytics · cloud sync · data export · hourly heatmaps · typing-detection postponement · automatic fullscreen-app detection · goal-setting · browser extension (separate repo) · code signing · app store listings · pricing/licensing · auto-updater · full Linux support.

## Verification approach

No test harness; validation is manual checkpoints per milestone.

1. **Per-milestone smoke checklist** — the demo criterion above is the gate. Runs in `pnpm tauri dev`.
2. **Short-interval mode** (interval 1 min / break 10 s / warning 5 s) for fast cycling — keep available via debug panel through M5.
3. **Settings round-trip** — change one setting, restart, confirm persistence + applied behavior.
4. **Multi-monitor probe** at M4 and M7.
5. **Idle probe** at M5: lock screen 3+ min, verify paused; unlock, verify reset (not resume).
6. **Production-build probe** at M7: every M1–M6 demo criterion must pass against the bundled `.app` / installed `.exe`, not just `pnpm tauri dev`.
7. **Friends pilot** post-M7: 2–4 testers for one week; capture feedback in a single doc; batch fixes for v1.1.
