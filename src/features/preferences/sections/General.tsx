import type { Settings, Theme } from "../../../shared/settings";
import { RadioGroup, Row, Section, Toggle } from "./shared";

type Patch = { general?: Partial<Settings["general"]> };

const THEME_OPTIONS: { value: Theme; label: string }[] = [
  { value: "light", label: "Light" },
  { value: "dark", label: "Dark" },
  { value: "system", label: "System" },
];

export function GeneralSection({
  settings,
  onUpdate,
}: {
  settings: Settings;
  onUpdate: (patch: Patch) => void;
}) {
  return (
    <Section title="General">
      <Row label="Launch at login">
        <Toggle
          checked={settings.general.autostart}
          onChange={(v) => onUpdate({ general: { autostart: v } })}
        />
      </Row>
      <Row label="Appearance">
        <RadioGroup
          value={settings.general.theme}
          options={THEME_OPTIONS}
          onChange={(v) => onUpdate({ general: { theme: v } })}
        />
      </Row>
      <Row label="Show streak" description="Track consecutive compliant days">
        <Toggle
          checked={settings.general.streaks_enabled}
          onChange={(v) => onUpdate({ general: { streaks_enabled: v } })}
        />
      </Row>
    </Section>
  );
}
