import { invoke } from "@tauri-apps/api/core";

export const SOUND_IDS = [
  "Glass",
  "Tink",
  "Pop",
  "Hero",
  "Ping",
  "Submarine",
  "Funk",
  "Bottle",
] as const;

export type SoundId = (typeof SOUND_IDS)[number];

export function isSoundId(v: unknown): v is SoundId {
  return typeof v === "string" && (SOUND_IDS as readonly string[]).includes(v);
}

export function playSound(id: string, volume: number): Promise<void> {
  return invoke<void>("play_sound", { id, volume });
}
