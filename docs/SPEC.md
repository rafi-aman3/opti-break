# 20-20-20 eye care app — build spec

## Overview

A cross-platform desktop utility that enforces the 20-20-20 rule: every 20 minutes, the user is reminded to look at something 20 feet away for 20 seconds. The app sits in the system tray, runs in the background, and dims the screen during breaks to make the pause feel real. It targets Windows, macOS, and Linux from a single codebase, with a companion browser extension covering Chrome, Edge, Brave, and Firefox.

The product is scoped narrowly to eye care. It is not a productivity tracker, screen time monitor, or general wellness app. Per-app tracking, cloud sync, and profile-based configuration were considered and explicitly cut from v1.

## Tech stack

The desktop app is built on Tauri 2.x. Tauri was chosen over Electron for its dramatically lower resource footprint — a Tauri binary runs at roughly 5–10 MB with 50–100 MB of RAM, versus 100+ MB and 150–300 MB for an equivalent Electron app. Since this app runs continuously throughout the user's working day, that difference matters.

The frontend uses TypeScript with Svelte (React is an acceptable alternative if preferred). Tailwind handles styling. The Rust backend handles the timer, OS-level integrations, and persistence. Four Tauri plugins do most of the heavy lifting: `tauri-plugin-autostart` for boot-time launch, `tauri-plugin-notification` for fallback notifications, `tauri-plugin-global-shortcut` for ESC during break overlays, and `tauri-plugin-sql` for the SQLite store. The `user-idle` Rust crate handles cross-platform idle detection.

The browser extension uses TypeScript with Vite (`@crxjs/vite-plugin`) and ships as Manifest V3. The same Manifest works for Chrome, Edge, and Brave; Firefox needs a small manifest tweak. Background timing uses `chrome.alarms` rather than `setInterval`, because MV3 service workers are killed after roughly 30 seconds of idle and `setInterval` callbacks won't fire reliably.

A small shared TypeScript package holds the timer logic, settings schema, and break overlay UI, imported by both desktop and extension targets.

## Core loop

Every 20 minutes, the timer fires a break event. At T-10 seconds, a small warning toast appears in the bottom-right corner of the user's screen, counting down. At T-0, the toast fades into a fullscreen dim overlay across all monitors. The overlay shows a 20-second countdown and the prompt "Look at something 20 feet away." After 20 seconds, the overlay fades out and the timer restarts.

The user can interrupt at any time. Hitting ESC during the warning skips the upcoming break; hitting ESC during the overlay ends it immediately. Both states feed into analytics with status `skipped`. The toast also exposes a "+5 min" button that postpones the break by five minutes.

Default timer values are a 20-minute interval and 20-second break duration. Both are configurable in preferences; interval can range 5–60 minutes and duration 10–60 seconds.

## Break experience

The dim overlay is the app's signature moment. It's implemented as one transparent, borderless, always-on-top Tauri window per connected monitor, created on break start and destroyed on break end. The overlay paints an `rgba(0, 0, 0, 0.78)` fill — 78% opacity — over the user's desktop, leaving a faint impression of their work behind it. This is gentler than full black, which feels like a system crash, while still being dark enough to discourage continuing to read. The opacity is configurable in settings between 50% and 95%.

Both the fade-in (400ms) and fade-out (250ms) are CSS transitions on the overlay's background-color. Instant overlays are startling; the fade gives the user a moment to register what's happening.

The break UI sits centered on the dim layer: a small "Eye break" label, a large countdown timer in tabular-nums style, the prompt "Look at something 20 feet away," a thin progress bar, and a small "Press esc to skip" hint. All white-on-dark.

ESC dismissal uses `tauri-plugin-global-shortcut` to register the key system-wide for the duration of the break, then unregisters when the break ends. Listening for ESC only inside the overlay window is unreliable on Windows and some Linux window managers because the overlay doesn't always grab focus cleanly.

Multi-monitor handling is automatic: the app enumerates monitors via `app.available_monitors()` on break start and creates one overlay window per monitor. There is no fullscreen-app detection in v1 — if the user is mid-presentation, the overlay still fires. The escape valve is the tray menu's "Pause for 30 min / 1 hour / until tomorrow" options, which let users opt out of breaks before sitting down to a meeting.

## Reminders

The pre-break warning is a small custom Tauri window — not an OS notification — pinned to the bottom-right corner of the primary display. It appears 10 seconds before the break, counts down to zero, and fades into the dim overlay. The toast is roughly 230 pixels wide and contains a status dot, the countdown text, the "Look at something far away" subtext, and two inline buttons: "Skip" and "+5 min."

Sound is off by default. Users can enable it in settings; v1 ships with one gentle chime. Sound respects system mute and DND.

A custom toast window was chosen over the OS notification API for two reasons. System notifications can be silently muted by the user's global notification settings, leading to "the app doesn't work" support tickets. They also don't reliably show a live countdown — they're fire-and-forget. A custom window guarantees consistent appearance and behavior across all three operating systems.

## Smart triggers

The app pauses when the user has been idle for three minutes (no mouse or keyboard input). This is detected via the `user-idle` crate, which abstracts `GetLastInputInfo` on Windows, `CGEventSourceSecondsSinceLastEventType` on macOS, and the appropriate X11/Wayland equivalents on Linux. On all three OSes, locked screens and sleeping displays register as idle through the same APIs, so no separate detection code is needed.

When activity returns, the timer resets to zero rather than resuming from where it paused. The reasoning is conservative: if the user was away for three or more minutes, their eyes already had a break.

The app launches on system boot by default, configurable in settings. This uses `tauri-plugin-autostart`, which handles the platform-specific registration (Login Items on macOS, Registry Run keys on Windows, autostart `.desktop` files on Linux).

## Analytics

The data model is intentionally minimal — one row per break event:

```sql
CREATE TABLE break_events (
  id INTEGER PRIMARY KEY AUTOINCREMENT,
  timestamp TEXT NOT NULL,
  status TEXT NOT NULL CHECK(status IN ('completed', 'skipped', 'postponed')),
  duration_actual INTEGER NOT NULL,
  monitor_count INTEGER NOT NULL
);
CREATE INDEX idx_timestamp ON break_events(timestamp);
```

`duration_actual` records the seconds the user actually spent in the break (20 for completed, less if they hit ESC early, 0 for skipped). `monitor_count` is informational. No personally identifying data, no app usage data, no window titles. Storage is local-only via `tauri-plugin-sql`; there is no cloud sync in v1.

Streaks are computed at read time, not stored. A "streak day" is a day where the user took at least 50% of scheduled breaks during a period of at least 30 minutes of active screen time. The 30-minute floor prevents days the laptop was barely used from counting against the streak. The streak ends when a day fails to qualify; there is no notification when this happens, no shame state, no recovery prompt. Users can disable streak tracking entirely via a settings toggle.

The stats screen shows three top metric cards (today's compliance, current streak, this week's total breaks), a 7-day bar chart of compliance, and an "all time" footer with total breaks taken, longest streak, and total "looking far away hours" — a small engagement metric framing the data positively.

## Customisation

Settings are stored as a single flat object, no profile nesting:

```json
{
  "timer": { "interval_minutes": 20, "break_seconds": 20 },
  "break": { "dim_opacity": 0.78, "monitors": "all", "fade_in_ms": 400 },
  "reminders": { "warning_seconds": 10, "sound_enabled": false, "sound_id": "chime_soft" },
  "schedule": { "active_hours_enabled": false, "active_hours": null, "idle_threshold_minutes": 3 },
  "general": { "autostart": true, "theme": "system", "streaks_enabled": true }
}
```

User-configurable fields cover the timer interval (5–60 min), break duration (10–60 sec), dim intensity, monitor selection, pre-break warning duration, sound settings, active hours window, idle threshold, autostart, theme (light/dark/system), and streak visibility. Everything else is hardcoded — fade durations, warning toast position, break overlay UI, ESC behavior, storage backend. Exposing these creates support burden without meaningful user value.

Active hours is off by default. When enabled, the user picks a daily start and end time and active days; outside those hours the timer doesn't run. Defaulting it on with a 9-to-6 preset would frustrate users who work irregular schedules.

## Tray menu

The system tray menu is the primary UI surface — most users interact with it dozens of times per day and never open the preferences window. It contains four sections separated by dividers. The status section shows the app name and time-until-next-break. The pause section offers "Pause for 30 min," "Pause for 1 hour," and "Pause until tomorrow." The actions section offers "Take a break now" and "Skip next break." The bottom section links to "Stats…", "Preferences…", and "Quit."

The tray icon should reflect state: a normal icon when the timer is running, a paused variant when paused, and a subtle indicator (for example, a small dot) during the last minute before a break. Clicking the icon opens the menu rather than the preferences window — preferences open from the menu item.

## Browser extension

The extension is a companion product, not a replacement. It covers cases where users primarily work in the browser and don't want to install a desktop app. The UX mirrors the desktop app: same break interval, same warning toast styling, same dim overlay aesthetic.

Two limitations apply. First, "dim" can only mean "inject a CSS overlay into the active tab" — the extension cannot dim the OS or other applications. Some users will find this acceptable; others will find it pointless. Communicate this clearly in onboarding. Second, idle detection is limited to `chrome.idle.queryState`, which is coarser than the native `user-idle` crate but adequate.

The Manifest V3 timer must use `chrome.alarms.create('eyebreak', { periodInMinutes: 20 })`, never `setInterval`. The service worker is killed after about 30 seconds of idle; alarms wake it on schedule. This is a common beginner mistake.

## Out of scope for v1

Profiles (Work / Evening / Weekend configurations) were considered and cut. Single global config plus active hours covers the realistic use cases without the storage and UI complexity of profile switching. The decision can be revisited in v2 once user feedback is in.

Per-app tracking and screen time analytics were considered and cut. They would shift the product positioning from eye care utility to productivity tracker, requiring scary permission prompts on macOS and roughly two weeks of additional development. Better as a v2 opt-in feature once the core product proves itself.

Cloud sync, data export, hourly heatmaps, typing-detection postponement, automatic fullscreen-app detection, and goal-setting features are all deferred to later releases. None are technically blocking; all add complexity without core value.

## Open questions for the build phase

A name for the app — "Eye care" is a placeholder throughout this document. Pick something memorable before any user-facing work ships.

Application icon and tray icon design — both required for distribution. The tray icon needs at minimum two states (active and paused) and ideally a third for the last-minute warning.

Code signing certificates — Apple Developer ($99/year) and a Windows code-signing certificate (~$200/year) for production releases. Skippable for early development; users will see "unidentified developer" warnings until signed.

Distribution strategy — direct download from a website, plus optional store listings (Mac App Store, Microsoft Store, Snapcraft, Flathub). Each store has its own review process and requirements.

Pricing model — free, donationware, or paid. The decision affects telemetry needs (do you measure conversion?), licensing infrastructure (do you need license-key validation?), and update mechanism (self-hosted vs store-managed).

Update mechanism — Tauri supports an updater plugin that fetches updates from a JSON manifest. Set this up before the first public release to avoid users being stuck on old versions.