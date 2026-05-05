import type { Settings } from "../../../shared/settings";
import { Row, Section, SliderRow, Toggle } from "./shared";
import { playSound, SOUND_IDS, isSoundId, type SoundId } from "../../../lib/sounds-client";

type Patch = { reminders?: Partial<Settings["reminders"]> };

export function RemindersSection({
  settings,
  onUpdate,
}: {
  settings: Settings;
  onUpdate: (patch: Patch) => void;
}) {
  const soundId: SoundId = isSoundId(settings.reminders.sound_id)
    ? settings.reminders.sound_id
    : "Glass";

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
        <>
          <Row label="Sound">
            <select
              value={soundId}
              onChange={(e) =>
                onUpdate({ reminders: { sound_id: e.target.value } })
              }
              className="rounded-md border border-neutral-200 dark:border-neutral-700 bg-white dark:bg-neutral-800 px-3 py-1.5 text-xs font-medium text-neutral-700 dark:text-neutral-300 focus:outline-none focus:ring-2 focus:ring-blue-500 cursor-pointer"
            >
              {SOUND_IDS.map((id) => (
                <option key={id} value={id}>
                  {id}
                </option>
              ))}
            </select>
          </Row>

          <SliderRow
            label="Volume"
            value={Math.round(settings.reminders.volume * 100)}
            min={0}
            max={100}
            unit="%"
            onChange={(v) => onUpdate({ reminders: { volume: v / 100 } })}
          />

          <Row label="Preview sound">
            <button
              type="button"
              onClick={() => playSound(soundId, settings.reminders.volume)}
              className="rounded-md border border-neutral-200 dark:border-neutral-700 px-3 py-1.5 text-xs font-medium text-neutral-700 dark:text-neutral-300 hover:bg-neutral-50 dark:hover:bg-neutral-800 transition-colors"
            >
              Play
            </button>
          </Row>
        </>
      )}
    </Section>
  );
}
