import { useState } from "react";
import { updateSettings } from "../../lib/settings-client";

interface Props {
  onComplete: () => void;
}

export function Onboarding({ onComplete }: Props) {
  const [intervalMinutes, setIntervalMinutes] = useState(20);
  const [autostart, setAutostart] = useState(true);
  const [saving, setSaving] = useState(false);

  async function handleStart() {
    setSaving(true);
    try {
      await updateSettings({
        timer: { interval_minutes: intervalMinutes },
        general: { autostart, onboarded: true },
      });
      onComplete();
    } catch (e) {
      console.error(e);
      setSaving(false);
    }
  }

  return (
    <div className="flex flex-col items-center justify-center h-full px-10 bg-white dark:bg-neutral-900 select-none">
      {/* Icon placeholder */}
      <div className="w-16 h-16 rounded-2xl bg-blue-500 flex items-center justify-center mb-6 shadow-lg">
        <svg viewBox="0 0 32 32" className="w-9 h-9 text-white fill-current" aria-hidden="true">
          <circle cx="16" cy="16" r="7" />
          <path d="M16 2a1.5 1.5 0 0 1 1.5 1.5v3a1.5 1.5 0 0 1-3 0v-3A1.5 1.5 0 0 1 16 2Zm0 24a1.5 1.5 0 0 1 1.5 1.5v3a1.5 1.5 0 0 1-3 0v-3A1.5 1.5 0 0 1 16 26ZM2 16a1.5 1.5 0 0 1 1.5-1.5h3a1.5 1.5 0 0 1 0 3h-3A1.5 1.5 0 0 1 2 16Zm24 0a1.5 1.5 0 0 1 1.5-1.5h3a1.5 1.5 0 0 1 0 3h-3A1.5 1.5 0 0 1 26 16Z" />
        </svg>
      </div>

      <h1 className="text-2xl font-bold text-neutral-900 dark:text-neutral-100 mb-1">
        Welcome to opti-break
      </h1>
      <p className="text-sm text-neutral-500 dark:text-neutral-400 text-center mb-10 max-w-xs">
        Every 20 minutes, take 20 seconds to look at something 20 feet away.
        Your eyes will thank you.
      </p>

      {/* Interval picker */}
      <div className="w-full max-w-sm space-y-2 mb-6">
        <div className="flex items-baseline justify-between text-sm">
          <span className="font-medium text-neutral-800 dark:text-neutral-200">
            Remind me every
          </span>
          <span className="tabular-nums font-semibold text-blue-500">
            {intervalMinutes} min
          </span>
        </div>
        <input
          type="range"
          min={5}
          max={60}
          value={intervalMinutes}
          onChange={(e) => setIntervalMinutes(parseInt(e.target.value, 10))}
          className="w-full accent-blue-500"
        />
        <div className="flex justify-between text-[10px] text-neutral-300 dark:text-neutral-600">
          <span>5 min</span>
          <span>60 min</span>
        </div>
      </div>

      {/* Autostart toggle */}
      <div className="w-full max-w-sm flex items-center justify-between mb-10">
        <div>
          <p className="text-sm font-medium text-neutral-800 dark:text-neutral-200">
            Launch at login
          </p>
          <p className="text-xs text-neutral-400 dark:text-neutral-500">
            opti-break runs quietly in the menu bar
          </p>
        </div>
        <button
          type="button"
          role="switch"
          aria-checked={autostart}
          onClick={() => setAutostart((v) => !v)}
          className={`relative inline-flex h-5 w-9 shrink-0 cursor-pointer items-center rounded-full transition-colors ${
            autostart ? "bg-blue-500" : "bg-neutral-300 dark:bg-neutral-600"
          }`}
        >
          <span
            className={`inline-block h-4 w-4 transform rounded-full bg-white shadow transition-transform ${
              autostart ? "translate-x-4" : "translate-x-0.5"
            }`}
          />
        </button>
      </div>

      {/* CTA */}
      <button
        type="button"
        onClick={handleStart}
        disabled={saving}
        className="w-full max-w-sm rounded-xl bg-blue-500 px-6 py-3 text-sm font-semibold text-white hover:bg-blue-600 active:bg-blue-700 disabled:opacity-60 transition-colors shadow"
      >
        {saving ? "Setting up…" : "Get started"}
      </button>
    </div>
  );
}
