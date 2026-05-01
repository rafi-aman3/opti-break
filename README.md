# opti-break

A lightweight desktop app that enforces the **20-20-20 rule**: every 20 minutes, look 20 feet away for 20 seconds.

Built with [Tauri 2](https://tauri.app) (Rust backend, WebView frontend). Runs on macOS and Windows. Lives in the menu bar / system tray — no Dock icon.

---

## Install

See **[docs/INSTALL.md](docs/INSTALL.md)** for step-by-step instructions, including how to get past Gatekeeper (macOS) and SmartScreen (Windows).

Download the latest build from the [Releases page](https://github.com/rafi-aman3/opti-break/releases/latest).

| Platform | File |
|---|---|
| macOS (Apple Silicon + Intel) | `opti-break_x.x.x_universal.dmg` |
| Windows 64-bit | `opti-break_x.x.x_x64-setup.exe` |

---

## What to expect

**First launch** shows a short setup screen — pick your break interval and whether opti-break should start when you log in.

After that, opti-break runs silently in the background:

- A **10-second warning toast** slides in from the bottom-right before each break.
- A **full-screen dim overlay** appears for 20 seconds (or your chosen duration).
- Press **ESC** at any time to skip the current break.
- The **tray/menu-bar icon** is the primary control surface — right-click for the full menu.

**Tray menu options**

| Item | What it does |
|---|---|
| Pause for 30 min / 1 hour | Suspends the timer; auto-resumes |
| Pause until tomorrow | Suspends until midnight |
| Resume | Cancels an active pause |
| Take a break now | Triggers an immediate break |
| Skip next break | Skips one upcoming break without pausing |
| Stats… | Opens the analytics view |
| Preferences… | Opens settings |
| Quit | Exits the app |

---

## Preferences

| Setting | Default | Notes |
|---|---|---|
| Break interval | 20 min | 5–60 min |
| Break duration | 20 sec | 10–60 sec |
| Pre-break warning | 10 sec | 5–30 sec |
| Dim opacity | 78 % | 50–95 % |
| Cover monitors | All | All or primary only |
| Sound | On | Short chime before break |
| Idle detection | On | Pauses timer after 3 min idle |
| Active hours | Off | Set a window + days to limit reminders |
| Start at login | Off | |
| Theme | System | Light / Dark / System |
| Streaks | On | Shown in Stats |

---

## Stats

The Stats screen shows:

- **Today's compliance** — breaks completed vs. attempted
- **This week** — total break time in minutes
- **Current streak** — consecutive days meeting the ≥ 50 % completion goal
- **7-day bar chart** — completed (blue) vs. skipped/postponed (grey)
- **All-time totals** — completed, skipped, total eye time, longest streak

All data is stored locally in SQLite (`~/Library/Application Support/com.rafiaman.opti-break/` on macOS). Nothing is sent anywhere.

---

## Building from source

```bash
# Prerequisites: Node 22+, pnpm 9+, Rust stable, Xcode CLT (macOS) or MSVC (Windows)

git clone https://github.com/rafi-aman3/opti-break.git
cd opti-break
pnpm install

# Development (hot-reload)
pnpm tauri dev

# Production build
pnpm tauri build
```

---

## Known limitations

- **Unsigned binaries** — Gatekeeper and SmartScreen will warn on first launch. See [docs/INSTALL.md](docs/INSTALL.md).
- **Full-screen apps** — the overlay will not cover a true GPU-exclusive full-screen app on macOS (Keynote presentation mode, full-screen games). This is an OS restriction.
- **No auto-updater** — update by downloading a new release.
- **Linux** — not an officially supported target in this release.

---

## License

MIT
