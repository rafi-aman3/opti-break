import { invoke } from "@tauri-apps/api/core";

import { type Settings, SettingsSchema } from "../shared/settings";

export async function getSettings(): Promise<Settings> {
  const raw = await invoke<unknown>("get_settings");
  return SettingsSchema.parse(raw);
}

export async function updateSettings(patch: DeepPartial<Settings>): Promise<Settings> {
  const raw = await invoke<unknown>("update_settings", { patch });
  return SettingsSchema.parse(raw);
}

export async function resetSettings(): Promise<Settings> {
  const raw = await invoke<unknown>("reset_settings");
  return SettingsSchema.parse(raw);
}

type DeepPartial<T> = T extends object ? { [K in keyof T]?: DeepPartial<T[K]> } : T;
