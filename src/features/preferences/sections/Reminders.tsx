import type { Settings } from "../../../shared/settings";
import { Row, Section, SliderRow, Toggle } from "./shared";

type Patch = { reminders?: Partial<Settings["reminders"]> };

function playPreviewChime() {
  try {
    const ctx = new AudioContext();
    const osc = ctx.createOscillator();
    const gain = ctx.createGain();
    osc.connect(gain);
    gain.connect(ctx.destination);
    osc.type = "sine";
    osc.frequency.value = 528;
    gain.gain.setValueAtTime(0.3, ctx.currentTime);
    gain.gain.exponentialRampToValueAtTime(0.001, ctx.currentTime + 1.5);
    osc.start();
    osc.stop(ctx.currentTime + 1.5);
    osc.onended = () => ctx.close();
  } catch {
    // ignore
  }
}

export function RemindersSection({
  settings,
  onUpdate,
}: {
  settings: Settings;
  onUpdate: (patch: Patch) => void;
}) {
  return (
    <Section title="Reminders">
      <SliderRow
        label="Warning time before break"
        value={settings.reminders.warning_seconds}
        min={5}
        max={30}
        unit="sec"
        onChange={(v) => onUpdate({ reminders: { warning_seconds: v } })}
      />
      <Row label="Sound alert">
        <Toggle
          checked={settings.reminders.sound_enabled}
          onChange={(v) => onUpdate({ reminders: { sound_enabled: v } })}
        />
      </Row>
      {settings.reminders.sound_enabled && (
        <Row label="Preview sound">
          <button
            type="button"
            onClick={playPreviewChime}
            className="rounded-md border border-neutral-200 dark:border-neutral-700 px-3 py-1.5 text-xs font-medium text-neutral-700 dark:text-neutral-300 hover:bg-neutral-50 dark:hover:bg-neutral-800 transition-colors"
          >
            Play
          </button>
        </Row>
      )}
    </Section>
  );
}
