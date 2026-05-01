# Installing opti-break

These builds are **unsigned**. Your OS will warn you the first time — follow the steps below to get past it.

---

## macOS

1. Download `opti-break_x.x.x_universal.dmg` from the [latest release](https://github.com/rafi-aman3/opti-break/releases/latest).
2. Open the DMG and drag **opti-break** into Applications.
3. Try to open it — macOS will block it with a "cannot be opened because the developer cannot be verified" message.
4. Open **System Settings → Privacy & Security** and scroll down until you see a message about opti-break being blocked. Click **Open Anyway**.
5. (Alternative) In Terminal, remove the quarantine flag and open directly:
   ```
   xattr -d com.apple.quarantine /Applications/opti-break.app
   open /Applications/opti-break.app
   ```
6. The app lives in the menu bar — no Dock icon by design.

---

## Windows

1. Download `opti-break_x.x.x_x64-setup.exe` from the [latest release](https://github.com/rafi-aman3/opti-break/releases/latest).
2. Run the installer. Windows SmartScreen will show "Windows protected your PC".
3. Click **More info**, then **Run anyway**.
4. Follow the installer prompts. opti-break will appear in the system tray after install.

> **Note**: SmartScreen warnings are shown for any software that hasn't accumulated enough install history with Microsoft. The app contains no malware — you can inspect the source code in this repo.

---

## What to expect

- **First launch**: a short setup screen lets you pick your break interval and whether to start at login.
- **Tray icon**: the primary interface. Left-click or right-click for the menu.
- **Every 20 minutes** (or your chosen interval): a 10-second warning toast appears, followed by a full-screen dim overlay for 20 seconds.
- **ESC** ends a break early. **+5 min** postpones. **Pause** options are in the tray menu.
- **Preferences** and **Stats** open via the tray menu.

---

## Known limitations

- Builds are unsigned. Gatekeeper/SmartScreen warnings are expected.
- Auto-updater is not included in this version.
- A break overlay will not cover a true fullscreen app on macOS (Keynote presentation mode, full-screen games via GPU exclusive mode). This is an OS restriction.
- Linux is not an officially supported target in this release.
