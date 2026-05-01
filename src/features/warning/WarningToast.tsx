import { useEffect, useState } from "react";

import { getSettings } from "../../lib/settings-client";
import {
  fmtSeconds,
  onStateChanged,
  onTick,
  timerClient,
  type TimerStatus,
} from "../../lib/timer-client";

function playChime() {
  try {
    const ctx = new AudioContext();
    const osc = ctx.createOscillator();
    const gain = ctx.createGain();
    osc.connect(gain);
    gain.connect(ctx.destination);
    osc.type = "sine";
    osc.frequency.value = 528;
    gain.gain.setValueAtTime(0.3, ctx.currentTime);
    gain.gain.exponentialRampToValueAtTime(0.001, ctx.currentTime + 1.5);
    osc.start();
    osc.stop(ctx.currentTime + 1.5);
    osc.onended = () => ctx.close();
  } catch {
    // Audio not available — silently ignore.
  }
}

export function WarningToast() {
  const [status, setStatus] = useState<TimerStatus | null>(null);

  useEffect(() => {
    timerClient.getStatus().then(setStatus).catch(console.error);
    const unsubs = [onTick(setStatus), onStateChanged(setStatus)];

    // Play chime once on mount if sound is enabled.
    getSettings()
      .then((s) => {
        if (s.reminders.sound_enabled) playChime();
      })
      .catch(console.error);

    return () => {
      unsubs.forEach((p) => p.then((fn) => fn()));
    };
  }, []);

  const secondsLeft = status?.seconds_until_break ?? null;

  return (
    <div
      className="flex h-full items-end justify-end"
      style={{ background: "transparent" }}
    >
      <div
        className="mb-0 mr-0 w-full overflow-hidden rounded-xl bg-neutral-900/95 shadow-2xl backdrop-blur-sm"
        style={{ animation: "fadeIn 200ms ease-out" }}
      >
        {/* Top bar */}
        <div className="flex items-center gap-2 px-4 pt-3.5 pb-1">
          <span className="h-2 w-2 rounded-full bg-amber-400 shadow-[0_0_6px_2px_rgba(251,191,36,0.5)]" />
          <span className="text-xs font-semibold tracking-wide text-neutral-200">
            Eye break in{" "}
            <span className="font-mono tabular-nums text-white">
              {fmtSeconds(secondsLeft)}
            </span>
          </span>
        </div>

        {/* Subtext */}
        <p className="px-4 pb-3 text-[11px] text-neutral-400">
          Look at something 20 feet away
        </p>

        {/* Actions */}
        <div className="flex border-t border-neutral-700/60">
          <button
            type="button"
            onClick={() => timerClient.skipNextBreak()}
            className="flex-1 py-2.5 text-xs font-medium text-neutral-400 transition-colors hover:bg-neutral-800 hover:text-neutral-200"
          >
            Skip
          </button>
          <div className="w-px bg-neutral-700/60" />
          <button
            type="button"
            onClick={() => timerClient.postponeBreak()}
            className="flex-1 py-2.5 text-xs font-medium text-neutral-400 transition-colors hover:bg-neutral-800 hover:text-neutral-200"
          >
            +5 min
          </button>
        </div>
      </div>

      <style>{`
        @keyframes fadeIn {
          from { opacity: 0; transform: translateY(8px); }
          to   { opacity: 1; transform: translateY(0); }
        }
      `}</style>
    </div>
  );
}
