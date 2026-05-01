import type { Settings } from "../../../shared/settings";
import { Section, SliderRow } from "./shared";

type Patch = { timer?: Partial<Settings["timer"]> };

export function TimerSection({
  settings,
  onUpdate,
}: {
  settings: Settings;
  onUpdate: (patch: Patch) => void;
}) {
  return (
    <Section title="Timer">
      <SliderRow
        label="Work interval"
        value={settings.timer.interval_minutes}
        min={5}
        max={60}
        unit="min"
        onChange={(v) => onUpdate({ timer: { interval_minutes: v } })}
      />
      <SliderRow
        label="Break duration"
        value={settings.timer.break_seconds}
        min={10}
        max={60}
        unit="sec"
        onChange={(v) => onUpdate({ timer: { break_seconds: v } })}
      />
    </Section>
  );
}
