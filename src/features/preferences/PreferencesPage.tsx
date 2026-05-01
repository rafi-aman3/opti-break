import { useEffect, useState } from "react";

import { type Settings } from "../../shared/settings";
import { getSettings, resetSettings } from "../../lib/settings-client";

export function PreferencesPage() {
  const [settings, setSettings] = useState<Settings | null>(null);
  const [error, setError] = useState<string | null>(null);

  useEffect(() => {
    getSettings().then(setSettings).catch((err) => setError(String(err)));
  }, []);

  return (
    <main className="mx-auto max-w-2xl px-8 py-10">
      <header className="mb-8">
        <h1 className="text-2xl font-semibold tracking-tight">opti-break</h1>
        <p className="mt-1 text-sm text-neutral-500">
          20-20-20 eye care · settings preview
        </p>
      </header>

      {error && (
        <div className="rounded-lg border border-red-300 bg-red-50 p-4 text-sm text-red-800">
          {error}
        </div>
      )}

      {settings && (
        <section className="space-y-4">
          <p className="text-sm text-neutral-600">
            Settings load from Rust over IPC. UI controls land in M6.
          </p>
          <pre className="overflow-auto rounded-lg border border-neutral-200 bg-neutral-50 p-4 text-xs">
            {JSON.stringify(settings, null, 2)}
          </pre>
          <button
            type="button"
            onClick={() => resetSettings().then(setSettings).catch((err) => setError(String(err)))}
            className="rounded-md bg-neutral-900 px-4 py-2 text-sm font-medium text-white hover:bg-neutral-800"
          >
            Reset to defaults
          </button>
        </section>
      )}
    </main>
  );
}
