import { useEffect, useState } from "react";

import { type Settings } from "../../shared/settings";
import { getSettings, resetSettings, updateSettings } from "../../lib/settings-client";
import { TimerSection } from "./sections/Timer";
import { BreakSection } from "./sections/Break";
import { RemindersSection } from "./sections/Reminders";
import { ScheduleSection } from "./sections/Schedule";
import { GeneralSection } from "./sections/General";

type DeepPartial<T> = T extends object ? { [K in keyof T]?: DeepPartial<T[K]> } : T;

export function PreferencesPage() {
  const [settings, setSettings] = useState<Settings | null>(null);
  const [error, setError] = useState<string | null>(null);
  const [resetting, setResetting] = useState(false);

  useEffect(() => {
    getSettings().then(setSettings).catch((e) => setError(String(e)));
  }, []);

  async function handleUpdate(patch: DeepPartial<Settings>) {
    try {
      const updated = await updateSettings(patch);
      setSettings(updated);
      setError(null);
    } catch (e) {
      setError(String(e));
    }
  }

  async function handleReset() {
    setResetting(true);
    try {
      const defaults = await resetSettings();
      setSettings(defaults);
      setError(null);
    } catch (e) {
      setError(String(e));
    } finally {
      setResetting(false);
    }
  }

  if (!settings) {
    return (
      <div className="flex items-center justify-center h-full text-neutral-400 dark:text-neutral-500 text-sm">
        {error ? (
          <p className="text-red-500 px-8 text-center">{error}</p>
        ) : (
          "Loading…"
        )}
      </div>
    );
  }

  return (
    <div className="flex flex-col h-full">
      <div className="flex-1 overflow-y-auto px-8 py-6 space-y-8">
        {error && (
          <p className="text-xs text-red-500 dark:text-red-400 bg-red-50 dark:bg-red-950/30 rounded-lg px-3 py-2">
            {error}
          </p>
        )}

        <TimerSection settings={settings} onUpdate={handleUpdate} />

        <div className="border-t border-neutral-100 dark:border-neutral-800" />
        <BreakSection settings={settings} onUpdate={handleUpdate} />

        <div className="border-t border-neutral-100 dark:border-neutral-800" />
        <RemindersSection settings={settings} onUpdate={handleUpdate} />

        <div className="border-t border-neutral-100 dark:border-neutral-800" />
        <ScheduleSection settings={settings} onUpdate={handleUpdate} />

        <div className="border-t border-neutral-100 dark:border-neutral-800" />
        <GeneralSection settings={settings} onUpdate={handleUpdate} />
      </div>

      {/* Footer */}
      <div className="shrink-0 border-t border-neutral-100 dark:border-neutral-800 px-8 py-4 flex items-center justify-between">
        <p className="text-xs text-neutral-400 dark:text-neutral-500">
          Analytics data is not cleared on reset.
        </p>
        <button
          type="button"
          onClick={handleReset}
          disabled={resetting}
          className="rounded-md border border-neutral-200 dark:border-neutral-700 px-3 py-1.5 text-xs font-medium text-neutral-600 dark:text-neutral-400 hover:bg-neutral-50 dark:hover:bg-neutral-800 disabled:opacity-50 transition-colors"
        >
          {resetting ? "Resetting…" : "Reset to defaults"}
        </button>
      </div>
    </div>
  );
}
