import React, { useEffect, useState } from "react";
import ReactDOM from "react-dom/client";
import { listen } from "@tauri-apps/api/event";

import { PreferencesPage } from "./features/preferences/PreferencesPage";
import { StatsPage } from "./features/stats/StatsPage";
import { WarningToast } from "./features/warning/WarningToast";
import { BreakOverlay } from "./features/overlay/BreakOverlay";
import { getSettings } from "./lib/settings-client";
import type { Settings, Theme } from "./shared/settings";
import "./index.css";

type WindowRoute = "preferences" | "stats" | "warning" | "overlay";
type MainTab = "preferences" | "stats";

function resolveRoute(): WindowRoute {
  const param = new URLSearchParams(window.location.search).get("route");
  switch (param) {
    case "warning": return "warning";
    case "overlay": return "overlay";
    case "stats": return "stats";
    default: return "preferences";
  }
}

// Transparent body for frameless windows.
const initialRoute = resolveRoute();
if (initialRoute === "warning" || initialRoute === "overlay") {
  document.documentElement.style.background = "transparent";
  document.body.style.background = "transparent";
}

// ── Theme helpers ─────────────────────────────────────────────────────────────

function applyTheme(theme: Theme) {
  const prefersDark = window.matchMedia("(prefers-color-scheme: dark)").matches;
  const dark = theme === "dark" || (theme === "system" && prefersDark);
  document.documentElement.classList.toggle("dark", dark);
}

// ── Main window (preferences + stats) ────────────────────────────────────────

function MainWindow({ initialTab }: { initialTab: MainTab }) {
  const [tab, setTab] = useState<MainTab>(initialTab);

  // Theme: load on mount, react to settings:updated and media changes.
  useEffect(() => {
    getSettings().then((s) => applyTheme(s.general.theme)).catch(console.error);

    const mq = window.matchMedia("(prefers-color-scheme: dark)");
    const mqListener = () =>
      getSettings().then((s) => {
        if (s.general.theme === "system") applyTheme("system");
      }).catch(console.error);
    mq.addEventListener("change", mqListener);

    const unsub = listen<Settings>("settings:updated", (e) =>
      applyTheme(e.payload.general.theme)
    );

    return () => {
      mq.removeEventListener("change", mqListener);
      unsub.then((fn) => fn());
    };
  }, []);

  return (
    <div className="flex flex-col h-full bg-white dark:bg-neutral-900">
      {/* Tab bar */}
      <header className="shrink-0 flex items-center gap-1 px-6 pt-5 pb-0">
        <h1 className="text-sm font-semibold text-neutral-900 dark:text-neutral-100 mr-4">
          opti-break
        </h1>
        <TabButton active={tab === "preferences"} onClick={() => setTab("preferences")}>
          Preferences
        </TabButton>
        <TabButton active={tab === "stats"} onClick={() => setTab("stats")}>
          Stats
        </TabButton>
      </header>

      <div className="mx-6 mt-3 mb-0 border-t border-neutral-100 dark:border-neutral-800" />

      {/* Content */}
      <div className="flex-1 min-h-0">
        {tab === "preferences" ? <PreferencesPage /> : <StatsPage />}
      </div>
    </div>
  );
}

function TabButton({
  active,
  onClick,
  children,
}: {
  active: boolean;
  onClick: () => void;
  children: React.ReactNode;
}) {
  return (
    <button
      type="button"
      onClick={onClick}
      className={`px-3 py-1.5 text-sm rounded-md font-medium transition-colors ${
        active
          ? "bg-neutral-100 dark:bg-neutral-800 text-neutral-900 dark:text-neutral-100"
          : "text-neutral-500 dark:text-neutral-400 hover:text-neutral-800 dark:hover:text-neutral-200"
      }`}
    >
      {children}
    </button>
  );
}

// ── Root ──────────────────────────────────────────────────────────────────────

function Root() {
  switch (initialRoute) {
    case "warning":
      return <WarningToast />;
    case "overlay":
      return <BreakOverlay />;
    default:
      return <MainWindow initialTab={initialRoute === "stats" ? "stats" : "preferences"} />;
  }
}

ReactDOM.createRoot(document.getElementById("root") as HTMLElement).render(
  <React.StrictMode>
    <Root />
  </React.StrictMode>,
);
