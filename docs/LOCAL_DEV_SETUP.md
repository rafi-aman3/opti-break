# Local Development Setup

Everything you need to run and build opti-break on your machine.

---

## Prerequisites

### All platforms

| Tool | Version | Install |
|---|---|---|
| Node.js | 22+ | [nodejs.org](https://nodejs.org) or `nvm` |
| pnpm | 9+ | `npm i -g pnpm` |
| Rust | stable | [rustup.rs](https://rustup.rs) |

### macOS

- **Xcode Command Line Tools** — `xcode-select --install`
- No extra Rust targets needed for dev; for a universal release build add:
  ```bash
  rustup target add aarch64-apple-darwin x86_64-apple-darwin
  ```

### Windows

- **Visual Studio Build Tools 2022** with the "Desktop development with C++" workload
- Or a full Visual Studio 2022 installation

### Linux (not officially supported)

Tauri on Linux requires `webkit2gtk`, `libappindicator`, and a few other system packages. See the [Tauri Linux prerequisites](https://tauri.app/start/prerequisites/#linux) page.

---

## First-time setup

```bash
git clone https://github.com/rafi-aman3/opti-break.git
cd opti-break
pnpm install
```

---

## Running in development

```bash
pnpm tauri dev
```

This starts two processes in parallel:
- **Vite** dev server on `http://localhost:1420` (hot-reload for frontend changes)
- **Rust/Tauri** process that opens the native window and hosts the WebView

Frontend changes reflect instantly. Rust changes trigger a recompile (takes 5–30s depending on what changed).

---

## Project structure

```
opti-break/
├── src/                        # React + TypeScript frontend
│   ├── features/
│   │   ├── break/              # Full-screen break overlay
│   │   ├── warning/            # Pre-break warning toast
│   │   ├── onboarding/         # First-launch setup screen
│   │   ├── preferences/        # Settings UI (all sections)
│   │   └── stats/              # Analytics screen
│   ├── shared/                 # Settings schema (Zod), shared components
│   └── lib/                    # IPC clients (timer, stats)
├── src-tauri/
│   ├── src/
│   │   ├── lib.rs              # App setup, IPC commands
│   │   ├── timer.rs            # Timer state machine
│   │   ├── tray.rs             # System tray setup and icon updates
│   │   ├── db.rs               # SQLite analytics layer
│   │   ├── idle.rs             # Idle detection (pauses timer)
│   │   ├── schedule.rs         # Active hours logic
│   │   ├── settings.rs         # Settings struct + load/save
│   │   ├── state.rs            # Shared AppState
│   │   └── windows.rs          # Overlay window management
│   ├── icons/                  # All app + tray icon sizes
│   ├── Cargo.toml
│   └── tauri.conf.json
├── scripts/
│   └── gen-tray-icons.mjs      # Regenerates tray PNGs from Lucide SVGs
├── docs/
│   ├── SPEC.md                 # Full product specification
│   ├── LOCAL_DEV_SETUP.md      # This file
│   └── INSTALL.md              # End-user install instructions
└── .github/workflows/
    └── release.yml             # CI: auto-version + build on push to main
```

---

## Key commands

```bash
# Development
pnpm tauri dev              # Start full app (frontend + Rust)

# Frontend only (no native window — useful for UI work)
pnpm build                  # TypeScript check + Vite bundle

# Rust only
cd src-tauri && cargo check # Fast type check, no link step
cd src-tauri && cargo build # Full Rust build

# Icons
pnpm tauri icon <file.png>  # Regenerate all icon sizes from a 1024x1024 PNG
pnpm gen-tray-icons         # Regenerate tray PNGs from Lucide SVGs

# Production build (local, unsigned)
pnpm tauri build
```

---

## IPC — adding a new command

1. Define in `src-tauri/src/lib.rs` with `#[tauri::command]`
2. Register it in `.invoke_handler(tauri::generate_handler![...])` in the same file
3. Call from the frontend with `invoke("command_name", { arg })` via `@tauri-apps/api`

---

## Settings

Settings are stored as JSON at:

| Platform | Path |
|---|---|
| macOS | `~/Library/Application Support/com.rafiaman.opti-break/settings.json` |
| Windows | `%APPDATA%\com.rafiaman.opti-break\settings.json` |

Delete the file to reset to defaults. The Rust struct (`src-tauri/src/settings.rs`) and the TypeScript Zod schema (`src/shared/settings.ts`) must stay in sync.

---

## Analytics database

SQLite database lives alongside settings:

| Platform | Path |
|---|---|
| macOS | `~/Library/Application Support/com.rafiaman.opti-break/analytics.db` |
| Windows | `%APPDATA%\com.rafiaman.opti-break\analytics.db` |

Schema: `break_events(id, timestamp, status, duration_actual, monitor_count)`.

---

## Contributing

See [`.claude/git-workflow.md`](../.claude/git-workflow.md) for branch naming, commit message format, and the release process.

Pull requests are welcome. Keep PRs small and focused — one feature or fix per PR.
