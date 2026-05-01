import { invoke } from "@tauri-apps/api/core";
import { listen, type UnlistenFn } from "@tauri-apps/api/event";

export type StateKind = "running" | "warning" | "on_break" | "paused";
export type PauseReason = "manual" | "idle" | "outside_hours";

export interface TimerStatus {
  state: StateKind;
  pause_reason: PauseReason | null;
  seconds_until_warning: number | null;
  seconds_until_break: number | null;
  seconds_remaining_in_break: number | null;
  postponed_count: number;
}

export const timerClient = {
  getStatus: () => invoke<TimerStatus>("get_timer_status"),
  start: () => invoke<void>("timer_start"),
  pause: () => invoke<void>("timer_pause"),
  resume: () => invoke<void>("timer_resume"),
  takeBreakNow: () => invoke<void>("take_break_now"),
  skipNextBreak: () => invoke<void>("skip_next_break"),
  postponeBreak: () => invoke<void>("postpone_break"),
};

export function onTick(cb: (s: TimerStatus) => void): Promise<UnlistenFn> {
  return listen<TimerStatus>("timer:tick", (e) => cb(e.payload));
}

export function onStateChanged(cb: (s: TimerStatus) => void): Promise<UnlistenFn> {
  return listen<TimerStatus>("timer:state_changed", (e) => cb(e.payload));
}

export function onWarningStarted(cb: (s: TimerStatus) => void): Promise<UnlistenFn> {
  return listen<TimerStatus>("timer:warning_started", (e) => cb(e.payload));
}

export function onBreakStarted(cb: (s: TimerStatus) => void): Promise<UnlistenFn> {
  return listen<TimerStatus>("timer:break_started", (e) => cb(e.payload));
}

export function onBreakEnded(cb: () => void): Promise<UnlistenFn> {
  return listen<void>("timer:break_ended", () => cb());
}

export function fmtSeconds(secs: number | null): string {
  if (secs === null) return "--:--";
  const s = Math.max(0, Math.round(secs));
  const m = Math.floor(s / 60);
  return `${m}:${String(s % 60).padStart(2, "0")}`;
}
