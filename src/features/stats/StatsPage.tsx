import { useEffect, useState } from "react";

import { statsClient, type Aggregates, type DayStats } from "../../lib/stats-client";
import { getSettings } from "../../lib/settings-client";
import { computeCurrentStreak, computeLongestStreak } from "./streak";

const DAY_LABELS = ["Sun", "Mon", "Tue", "Wed", "Thu", "Fri", "Sat"];

function todayLocalDate(): string {
  return new Date().toLocaleDateString("sv"); // YYYY-MM-DD
}

function last7Days(): string[] {
  const days: string[] = [];
  for (let i = 6; i >= 0; i--) {
    const d = new Date();
    d.setDate(d.getDate() - i);
    days.push(d.toLocaleDateString("sv"));
  }
  return days;
}

function fmtHours(secs: number): string {
  const h = secs / 3600;
  return h < 1 ? `${Math.round(h * 60)} min` : `${h.toFixed(1)} h`;
}

// ── Bar chart ─────────────────────────────────────────────────────────────────

function BarChart({ days, dayData }: { days: string[]; dayData: Record<string, DayStats> }) {
  const maxTotal = Math.max(
    1,
    ...days.map((d) => (dayData[d]?.completed ?? 0) + (dayData[d]?.skipped ?? 0))
  );

  return (
    <svg viewBox={`0 0 ${days.length * 40} 80`} className="w-full" aria-hidden="true">
      {days.map((date, i) => {
        const stats = dayData[date];
        const completed = stats?.completed ?? 0;
        const total = completed + (stats?.skipped ?? 0);
        const barH = total > 0 ? Math.max(4, Math.round((total / maxTotal) * 60)) : 2;
        const compH = total > 0 ? Math.round((completed / total) * barH) : 0;
        const x = i * 40 + 8;
        const weekday = new Date(date + "T12:00:00").getDay();
        const isToday = date === todayLocalDate();

        return (
          <g key={date}>
            {/* background bar */}
            <rect x={x} y={8} width={24} height={60} rx={4} fill="currentColor"
              className="text-neutral-100 dark:text-neutral-800" />
            {/* skipped portion */}
            {total > 0 && (
              <rect x={x} y={8 + (60 - barH)} width={24} height={barH} rx={4}
                fill="currentColor" className="text-neutral-300 dark:text-neutral-600" />
            )}
            {/* completed portion */}
            {compH > 0 && (
              <rect x={x} y={8 + (60 - compH)} width={24} height={compH} rx={4}
                fill="currentColor" className="text-blue-500 dark:text-blue-400" />
            )}
            {/* day label */}
            <text x={x + 12} y={78} textAnchor="middle" fontSize={9}
              fill="currentColor"
              className={`font-sans ${isToday ? "text-blue-500 dark:text-blue-400 font-semibold" : "text-neutral-400 dark:text-neutral-500"}`}>
              {DAY_LABELS[weekday]}
            </text>
          </g>
        );
      })}
    </svg>
  );
}

// ── Stat card ─────────────────────────────────────────────────────────────────

function StatCard({ label, value, sub }: { label: string; value: string; sub?: string }) {
  return (
    <div className="rounded-xl bg-neutral-50 dark:bg-neutral-800 p-4">
      <p className="text-xs text-neutral-400 dark:text-neutral-500 uppercase tracking-wide mb-1">{label}</p>
      <p className="text-2xl font-semibold tabular-nums text-neutral-900 dark:text-neutral-100">{value}</p>
      {sub && <p className="text-xs text-neutral-400 dark:text-neutral-500 mt-0.5">{sub}</p>}
    </div>
  );
}

// ── Main page ─────────────────────────────────────────────────────────────────

export function StatsPage() {
  const [days365, setDays365] = useState<DayStats[]>([]);
  const [aggregates, setAggregates] = useState<Aggregates | null>(null);
  const [streaksEnabled, setStreaksEnabled] = useState(true);
  const [loading, setLoading] = useState(true);

  useEffect(() => {
    Promise.all([
      statsClient.getDayStats(365),
      statsClient.getAggregates(),
      getSettings(),
    ]).then(([d, agg, s]) => {
      setDays365(d);
      setAggregates(agg);
      setStreaksEnabled(s.general.streaks_enabled);
    }).catch(console.error).finally(() => setLoading(false));
  }, []);

  const today = todayLocalDate();
  const week7 = last7Days();
  const dayIndex = Object.fromEntries(days365.map((d) => [d.date, d]));

  // Today's compliance
  const todayStats = dayIndex[today];
  const todayAttempted = (todayStats?.completed ?? 0) + (todayStats?.skipped ?? 0);
  const todayCompliance = todayAttempted > 0
    ? Math.round(((todayStats?.completed ?? 0) / todayAttempted) * 100)
    : null;

  // This week's completed total
  const weekCompleted = week7.reduce((sum, d) => sum + (dayIndex[d]?.completed ?? 0), 0);

  // Streak
  const currentStreak = computeCurrentStreak(days365);
  const longestStreak = computeLongestStreak(days365);

  if (loading) {
    return (
      <div className="flex items-center justify-center h-full text-neutral-400 dark:text-neutral-500 text-sm">
        Loading…
      </div>
    );
  }

  const hasAnyData = (aggregates?.total_completed ?? 0) + (aggregates?.total_skipped ?? 0) > 0;

  if (!hasAnyData) {
    return (
      <div className="flex flex-col items-center justify-center h-full gap-2 text-center px-8">
        <p className="text-neutral-400 dark:text-neutral-500 text-sm">No data yet.</p>
        <p className="text-neutral-300 dark:text-neutral-600 text-xs">
          Your first break stats will appear here after the next cycle.
        </p>
      </div>
    );
  }

  return (
    <div className="px-8 py-6 space-y-6 overflow-y-auto h-full">
      {/* Top cards */}
      <div className="grid grid-cols-3 gap-3">
        <StatCard
          label="Today's compliance"
          value={todayCompliance !== null ? `${todayCompliance}%` : "—"}
          sub={todayAttempted > 0
            ? `${todayStats?.completed ?? 0} of ${todayAttempted} breaks`
            : "No breaks yet today"}
        />
        <StatCard
          label="This week"
          value={String(weekCompleted)}
          sub="breaks completed"
        />
        {streaksEnabled && (
          <StatCard
            label="Current streak"
            value={String(currentStreak)}
            sub={currentStreak === 1 ? "day" : "days"}
          />
        )}
        {!streaksEnabled && (
          <StatCard
            label="Eye time"
            value={fmtHours(aggregates?.total_duration_secs ?? 0)}
            sub="total looking far"
          />
        )}
      </div>

      {/* 7-day chart */}
      <div>
        <p className="text-xs font-semibold uppercase tracking-widest text-neutral-400 dark:text-neutral-500 mb-3">
          Last 7 days
        </p>
        <BarChart days={week7} dayData={dayIndex} />
        <div className="flex gap-4 mt-2 text-xs text-neutral-400 dark:text-neutral-500">
          <span className="flex items-center gap-1.5">
            <span className="inline-block w-2.5 h-2.5 rounded-sm bg-blue-500 dark:bg-blue-400" />
            Completed
          </span>
          <span className="flex items-center gap-1.5">
            <span className="inline-block w-2.5 h-2.5 rounded-sm bg-neutral-300 dark:bg-neutral-600" />
            Skipped
          </span>
        </div>
      </div>

      {/* All-time footer */}
      <div className="border-t border-neutral-200 dark:border-neutral-700 pt-5">
        <p className="text-xs font-semibold uppercase tracking-widest text-neutral-400 dark:text-neutral-500 mb-3">
          All time
        </p>
        <div className="grid grid-cols-2 gap-x-8 gap-y-2 text-sm">
          <Row label="Total breaks completed" value={String(aggregates?.total_completed ?? 0)} />
          <Row label="Total breaks skipped" value={String(aggregates?.total_skipped ?? 0)} />
          <Row label="Looking-far-away time" value={fmtHours(aggregates?.total_duration_secs ?? 0)} />
          {streaksEnabled && (
            <Row label="Longest streak" value={`${longestStreak} day${longestStreak === 1 ? "" : "s"}`} />
          )}
        </div>
      </div>
    </div>
  );
}

function Row({ label, value }: { label: string; value: string }) {
  return (
    <div className="flex items-baseline justify-between gap-2">
      <span className="text-neutral-500 dark:text-neutral-400">{label}</span>
      <span className="font-semibold tabular-nums text-neutral-900 dark:text-neutral-100 shrink-0">
        {value}
      </span>
    </div>
  );
}
