import { useEffect, useRef, useState } from "react";
import { getCurrentWindow } from "@tauri-apps/api/window";
import { onTick, onBreakEnded, fmtSeconds } from "../../lib/timer-client";

function urlParam(key: string): string | null {
  return new URLSearchParams(window.location.search).get(key);
}

export function BreakOverlay() {
  const totalSeconds = parseInt(urlParam("duration") ?? "20", 10);
  const opacity = parseFloat(urlParam("opacity") ?? "0.78");

  const [visible, setVisible] = useState(false);
  const [secsRemaining, setSecsRemaining] = useState<number | null>(null);
  const closingRef = useRef(false);

  useEffect(() => {
    // Trigger fade-in after first paint.
    requestAnimationFrame(() => setVisible(true));

    const unTick = onTick((s) => {
      if (s.seconds_remaining_in_break !== null) {
        setSecsRemaining(s.seconds_remaining_in_break);
      }
    });

    const unEnded = onBreakEnded(() => {
      if (closingRef.current) return;
      closingRef.current = true;
      setVisible(false);
      setTimeout(() => getCurrentWindow().close(), 260);
    });

    return () => {
      unTick.then((fn) => fn());
      unEnded.then((fn) => fn());
    };
  }, []);

  // Fraction of break elapsed (0 → 1).
  const progress =
    secsRemaining !== null ? 1 - Math.max(0, secsRemaining) / totalSeconds : 0;

  return (
    <div
      style={{
        backgroundColor: `rgba(0,0,0,${opacity})`,
        opacity: visible ? 1 : 0,
        transition: `opacity ${visible ? 400 : 250}ms ease`,
      }}
      className="fixed inset-0 flex items-center justify-center text-white select-none"
    >
      <div className="flex flex-col items-center gap-4">
        <p className="text-sm tracking-[0.25em] text-white/60 uppercase">
          Eye Break
        </p>

        <p className="text-8xl font-extralight tabular-nums">
          {fmtSeconds(secsRemaining)}
        </p>

        <p className="text-lg text-white/80">
          Look at something 20 feet away.
        </p>

        <div className="mt-1 w-48 h-1 rounded-full bg-white/20 overflow-hidden">
          <div
            className="h-full rounded-full bg-white/60 transition-all duration-1000 ease-linear"
            style={{ width: `${Math.min(100, progress * 100)}%` }}
          />
        </div>

        <p className="mt-1 text-xs text-white/40">Press esc to skip</p>
      </div>
    </div>
  );
}
