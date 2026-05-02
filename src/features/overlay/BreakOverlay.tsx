import { useEffect, useRef, useState } from "react";
import { getCurrentWindow } from "@tauri-apps/api/window";
import { onBreakEnded, fmtSeconds } from "../../lib/timer-client";

function urlParam(key: string): string | null {
  return new URLSearchParams(window.location.search).get(key);
}

export function BreakOverlay() {
  const totalSeconds = parseInt(urlParam("duration") ?? "20", 10);
  const opacity = parseFloat(urlParam("opacity") ?? "0.78");

  const [visible, setVisible] = useState(false);
  const [secsRemaining, setSecsRemaining] = useState(totalSeconds);
  const closingRef = useRef(false);

  useEffect(() => {
    requestAnimationFrame(() => setVisible(true));

    const id = setInterval(() => {
      setSecsRemaining((prev) => Math.max(0, prev - 1));
    }, 1000);

    const unEnded = onBreakEnded(() => {
      if (closingRef.current) return;
      closingRef.current = true;
      clearInterval(id);
      setVisible(false);
      setTimeout(() => getCurrentWindow().close(), 260);
    });

    return () => {
      clearInterval(id);
      unEnded.then((fn) => fn());
    };
  }, []);

  const progress = 1 - Math.max(0, secsRemaining) / totalSeconds;

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
