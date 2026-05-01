import { useEffect, useState } from "react";

import {
  type TimerStatus,
  fmtSeconds,
  onStateChanged,
  onTick,
  timerClient,
} from "../../lib/timer-client";

export function DebugPanel() {
  const [status, setStatus] = useState<TimerStatus | null>(null);

  useEffect(() => {
    timerClient.getStatus().then(setStatus).catch(console.error);

    const unsubs = [
      onTick(setStatus),
      onStateChanged(setStatus),
    ];
    return () => {
      unsubs.forEach((p) => p.then((fn) => fn()));
    };
  }, []);

  const btn =
    "rounded bg-neutral-800 px-3 py-1.5 text-xs font-medium text-white hover:bg-neutral-700 active:bg-neutral-900";

  return (
    <section className="mt-8 rounded-xl border border-amber-200 bg-amber-50 p-5">
      <h2 className="mb-3 text-xs font-semibold uppercase tracking-widest text-amber-600">
        M2 Debug panel — remove in M6
      </h2>

      {status ? (
        <div className="space-y-3">
          <div className="grid grid-cols-2 gap-2 text-xs">
            <Stat label="State" value={status.state} />
            <Stat label="Pause reason" value={status.pause_reason ?? "—"} />
            <Stat
              label="Until warning"
              value={fmtSeconds(status.seconds_until_warning)}
            />
            <Stat
              label="Until break"
              value={fmtSeconds(status.seconds_until_break)}
            />
            <Stat
              label="Break remaining"
              value={fmtSeconds(status.seconds_remaining_in_break)}
            />
            <Stat label="Postponed" value={String(status.postponed_count)} />
          </div>

          <div className="flex flex-wrap gap-2 pt-1">
            <button className={btn} onClick={() => timerClient.start()}>
              Start
            </button>
            <button className={btn} onClick={() => timerClient.pause()}>
              Pause
            </button>
            <button className={btn} onClick={() => timerClient.resume()}>
              Resume
            </button>
            <button className={btn} onClick={() => timerClient.takeBreakNow()}>
              Break now
            </button>
            <button className={btn} onClick={() => timerClient.skipNextBreak()}>
              Skip next
            </button>
            <button className={btn} onClick={() => timerClient.postponeBreak()}>
              +5 min
            </button>
          </div>
        </div>
      ) : (
        <p className="text-xs text-amber-700">Loading…</p>
      )}
    </section>
  );
}

function Stat({ label, value }: { label: string; value: string }) {
  return (
    <div>
      <span className="text-neutral-500">{label}: </span>
      <span className="font-mono font-semibold text-neutral-900">{value}</span>
    </div>
  );
}
