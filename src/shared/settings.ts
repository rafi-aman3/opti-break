import { z } from "zod";

export const ThemeSchema = z.enum(["light", "dark", "system"]);
export type Theme = z.infer<typeof ThemeSchema>;

export const MonitorSelectionSchema = z.enum(["all", "primary"]);
export type MonitorSelection = z.infer<typeof MonitorSelectionSchema>;

export const ActiveHoursSchema = z.object({
  start: z.string().regex(/^\d{2}:\d{2}$/),
  end: z.string().regex(/^\d{2}:\d{2}$/),
  days: z.array(z.number().int().min(0).max(6)),
});
export type ActiveHours = z.infer<typeof ActiveHoursSchema>;

export const SettingsSchema = z.object({
  timer: z.object({
    interval_minutes: z.number().int().min(5).max(60),
    break_seconds: z.number().int().min(10).max(60),
  }),
  break: z.object({
    dim_opacity: z.number().min(0.5).max(0.95),
    monitors: MonitorSelectionSchema,
    fade_in_ms: z.number().int().min(0).max(2000),
  }),
  reminders: z.object({
    warning_seconds: z.number().int().min(0).max(30),
    sound_enabled: z.boolean(),
    sound_id: z.string(),
  }),
  schedule: z.object({
    active_hours_enabled: z.boolean(),
    active_hours: ActiveHoursSchema.nullable(),
    idle_threshold_minutes: z.number().int().min(1).max(30),
  }),
  general: z.object({
    autostart: z.boolean(),
    theme: ThemeSchema,
    streaks_enabled: z.boolean(),
    onboarded: z.boolean().default(false),
  }),
});

export type Settings = z.infer<typeof SettingsSchema>;

export const DEFAULT_SETTINGS: Settings = {
  timer: { interval_minutes: 20, break_seconds: 20 },
  break: { dim_opacity: 0.78, monitors: "all", fade_in_ms: 400 },
  reminders: { warning_seconds: 10, sound_enabled: false, sound_id: "chime_soft" },
  schedule: { active_hours_enabled: false, active_hours: null, idle_threshold_minutes: 3 },
  general: { autostart: true, theme: "system", streaks_enabled: true, onboarded: false },
};
