import type { Settings } from "../../../shared/settings";
import { Row, Section, SliderRow, Toggle } from "./shared";

type Patch = { schedule?: Partial<Settings["schedule"]> };

const DAYS = [
  { value: 1, label: "M" },
  { value: 2, label: "T" },
  { value: 3, label: "W" },
  { value: 4, label: "T" },
  { value: 5, label: "F" },
  { value: 6, label: "S" },
  { value: 0, label: "S" },
];

export function ScheduleSection({
  settings,
  onUpdate,
}: {
  settings: Settings;
  onUpdate: (patch: Patch) => void;
}) {
  const ah = settings.schedule.active_hours ?? {
    start: "09:00",
    end: "18:00",
    days: [1, 2, 3, 4, 5],
  };

  function toggleDay(day: number) {
    const current = ah.days;
    const next = current.includes(day)
      ? current.filter((d) => d !== day)
      : [...current, day].sort((a, b) => a - b);
    onUpdate({ schedule: { active_hours: { ...ah, days: next } } });
  }

  return (
    <Section title="Schedule">
      <SliderRow
        label="Idle pause threshold"
        value={settings.schedule.idle_threshold_minutes}
        min={1}
        max={10}
        unit="min"
        onChange={(v) => onUpdate({ schedule: { idle_threshold_minutes: v } })}
      />
      <Row label="Active hours" description="Pause outside these hours">
        <Toggle
          checked={settings.schedule.active_hours_enabled}
          onChange={(v) =>
            onUpdate({
              schedule: {
                active_hours_enabled: v,
                active_hours: v ? ah : ah,
              },
            })
          }
        />
      </Row>

      {settings.schedule.active_hours_enabled && (
        <>
          {/* Time range */}
          <div className="flex items-center gap-3 text-sm">
            <input
              type="time"
              value={ah.start}
              onChange={(e) =>
                onUpdate({ schedule: { active_hours: { ...ah, start: e.target.value } } })
              }
              className="rounded-md border border-neutral-200 dark:border-neutral-700 bg-white dark:bg-neutral-800 px-2 py-1.5 text-xs text-neutral-800 dark:text-neutral-200 focus:outline-none focus:ring-2 focus:ring-blue-500"
            />
            <span className="text-neutral-400 dark:text-neutral-500 text-xs">to</span>
            <input
              type="time"
              value={ah.end}
              onChange={(e) =>
                onUpdate({ schedule: { active_hours: { ...ah, end: e.target.value } } })
              }
              className="rounded-md border border-neutral-200 dark:border-neutral-700 bg-white dark:bg-neutral-800 px-2 py-1.5 text-xs text-neutral-800 dark:text-neutral-200 focus:outline-none focus:ring-2 focus:ring-blue-500"
            />
          </div>

          {/* Day picker */}
          <div className="flex gap-1.5">
            {DAYS.map((d, i) => {
              const active = ah.days.includes(d.value);
              return (
                <button
                  key={`${d.value}-${i}`}
                  type="button"
                  onClick={() => toggleDay(d.value)}
                  className={`w-8 h-8 rounded-full text-xs font-semibold transition-colors ${
                    active
                      ? "bg-blue-500 text-white"
                      : "bg-neutral-100 dark:bg-neutral-800 text-neutral-500 dark:text-neutral-400 hover:bg-neutral-200 dark:hover:bg-neutral-700"
                  }`}
                >
                  {d.label}
                </button>
              );
            })}
          </div>
        </>
      )}
    </Section>
  );
}
