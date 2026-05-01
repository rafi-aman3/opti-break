import { useRef, type ReactNode } from "react";

// ── Shared primitives used by all preference sections ─────────────────────────

export function Section({ title, children }: { title: string; children: ReactNode }) {
  return (
    <section className="space-y-4">
      <h2 className="text-[11px] font-semibold uppercase tracking-widest text-neutral-400 dark:text-neutral-500">
        {title}
      </h2>
      <div className="space-y-3">{children}</div>
    </section>
  );
}

export function Row({
  label,
  description,
  children,
}: {
  label: string;
  description?: string;
  children: ReactNode;
}) {
  return (
    <div className="flex items-center justify-between gap-6">
      <div className="min-w-0">
        <p className="text-sm font-medium text-neutral-800 dark:text-neutral-200">{label}</p>
        {description && (
          <p className="text-xs text-neutral-400 dark:text-neutral-500 mt-0.5">{description}</p>
        )}
      </div>
      <div className="shrink-0">{children}</div>
    </div>
  );
}

export function Toggle({
  checked,
  onChange,
}: {
  checked: boolean;
  onChange: (v: boolean) => void;
}) {
  return (
    <button
      type="button"
      role="switch"
      aria-checked={checked}
      onClick={() => onChange(!checked)}
      className={`relative inline-flex h-5 w-9 shrink-0 cursor-pointer items-center rounded-full transition-colors focus-visible:outline-none focus-visible:ring-2 focus-visible:ring-blue-500 ${
        checked
          ? "bg-blue-500"
          : "bg-neutral-300 dark:bg-neutral-600"
      }`}
    >
      <span
        className={`inline-block h-4 w-4 transform rounded-full bg-white shadow transition-transform ${
          checked ? "translate-x-4" : "translate-x-0.5"
        }`}
      />
    </button>
  );
}

/** Slider with debounced onChange. */
export function SliderRow({
  label,
  value,
  min,
  max,
  unit,
  onChange,
}: {
  label: string;
  value: number;
  min: number;
  max: number;
  unit: string;
  onChange: (v: number) => void;
}) {
  const timerRef = useRef<ReturnType<typeof setTimeout> | undefined>(undefined);

  function handleChange(raw: string) {
    const v = parseInt(raw, 10);
    clearTimeout(timerRef.current);
    timerRef.current = setTimeout(() => onChange(v), 250);
  }

  return (
    <div className="space-y-1.5">
      <div className="flex items-baseline justify-between text-sm">
        <span className="font-medium text-neutral-800 dark:text-neutral-200">{label}</span>
        <span className="tabular-nums text-neutral-400 dark:text-neutral-500 text-xs">
          {value} {unit}
        </span>
      </div>
      <input
        type="range"
        min={min}
        max={max}
        defaultValue={value}
        key={value} // re-mount when reset externally
        onChange={(e) => handleChange(e.target.value)}
        className="w-full accent-blue-500 cursor-pointer"
      />
      <div className="flex justify-between text-[10px] text-neutral-300 dark:text-neutral-600 select-none">
        <span>{min}</span>
        <span>{max}</span>
      </div>
    </div>
  );
}

export function RadioGroup<T extends string>({
  value,
  options,
  onChange,
}: {
  value: T;
  options: { value: T; label: string }[];
  onChange: (v: T) => void;
}) {
  return (
    <div className="flex rounded-lg overflow-hidden border border-neutral-200 dark:border-neutral-700 text-sm">
      {options.map((opt, i) => (
        <button
          key={opt.value}
          type="button"
          onClick={() => onChange(opt.value)}
          className={`flex-1 py-1.5 px-3 text-xs font-medium transition-colors focus-visible:outline-none ${
            i > 0 ? "border-l border-neutral-200 dark:border-neutral-700" : ""
          } ${
            value === opt.value
              ? "bg-blue-500 text-white"
              : "bg-white dark:bg-neutral-800 text-neutral-600 dark:text-neutral-300 hover:bg-neutral-50 dark:hover:bg-neutral-700"
          }`}
        >
          {opt.label}
        </button>
      ))}
    </div>
  );
}
