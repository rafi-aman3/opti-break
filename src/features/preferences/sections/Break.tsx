import type { MonitorSelection, Settings } from "../../../shared/settings";
import { RadioGroup, Row, Section, SliderRow } from "./shared";

type Patch = { break?: Partial<Settings["break"]> };

const MONITOR_OPTIONS: { value: MonitorSelection; label: string }[] = [
  { value: "all", label: "All monitors" },
  { value: "primary", label: "Primary only" },
];

export function BreakSection({
  settings,
  onUpdate,
}: {
  settings: Settings;
  onUpdate: (patch: Patch) => void;
}) {
  return (
    <Section title="Break overlay">
      <SliderRow
        label="Dim opacity"
        value={Math.round(settings.break.dim_opacity * 100)}
        min={50}
        max={95}
        unit="%"
        onChange={(v) => onUpdate({ break: { dim_opacity: v / 100 } })}
      />
      <Row label="Covered monitors">
        <RadioGroup
          value={settings.break.monitors}
          options={MONITOR_OPTIONS}
          onChange={(v) => onUpdate({ break: { monitors: v } })}
        />
      </Row>
    </Section>
  );
}
