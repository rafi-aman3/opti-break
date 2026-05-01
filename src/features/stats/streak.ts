import type { DayStats } from "../../lib/stats-client";

// A day "counts" toward the streak if it has at least this many completed breaks
// (proxy for ≥30 min of active screen time) and ≥50% compliance.
const MIN_COMPLETED = 2;
const MIN_COMPLIANCE = 0.5;

function qualifies(day: DayStats): boolean {
  const attempted = day.completed + day.skipped;
  if (attempted === 0) return false;
  return day.completed >= MIN_COMPLETED && day.completed / attempted >= MIN_COMPLIANCE;
}

function hasActivity(day: DayStats): boolean {
  return day.completed + day.skipped > 0;
}

/**
 * Walk back from the most recent day. Days with no activity are skipped
 * (they don't break the streak — you just weren't at the computer).
 * The first day that has activity but doesn't qualify breaks the streak.
 */
export function computeCurrentStreak(days: DayStats[]): number {
  const sorted = [...days].sort((a, b) => b.date.localeCompare(a.date));
  let streak = 0;
  for (const day of sorted) {
    if (!hasActivity(day)) continue;
    if (qualifies(day)) {
      streak++;
    } else {
      break;
    }
  }
  return streak;
}

export function computeLongestStreak(days: DayStats[]): number {
  const sorted = [...days].sort((a, b) => a.date.localeCompare(b.date));
  let longest = 0;
  let current = 0;
  for (const day of sorted) {
    if (!hasActivity(day)) continue;
    if (qualifies(day)) {
      current++;
      if (current > longest) longest = current;
    } else {
      current = 0;
    }
  }
  return longest;
}
