import { invoke } from "@tauri-apps/api/core";

export interface DayStats {
  date: string; // YYYY-MM-DD local
  completed: number;
  skipped: number;
  postponed: number;
}

export interface Aggregates {
  total_completed: number;
  total_skipped: number;
  total_postponed: number;
  total_duration_secs: number;
}

export const statsClient = {
  getDayStats: (days: number) => invoke<DayStats[]>("get_day_stats", { days }),
  getAggregates: () => invoke<Aggregates>("get_aggregates"),
};
