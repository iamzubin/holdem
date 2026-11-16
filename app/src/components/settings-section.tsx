import React, { useEffect, useState } from 'react';
import type { ReactNode } from 'react';
import { Input } from '@/components/ui/input';
import { Label } from '@/components/ui/label';

export const SectionTitle: React.FC<{ icon: ReactNode; title: string; description: string }> = ({
  icon,
  title,
  description,
}) => (
  <div className="flex items-start gap-2">
    <span className="mt-0.5 text-primary [&>svg]:h-4 [&>svg]:w-4">{icon}</span>
    <div>
      <h2 className="text-sm font-semibold">{title}</h2>
      <p className="text-xs text-muted-foreground">{description}</p>
    </div>
  </div>
);

export const SettingRow: React.FC<{ label: string; description: string; children: ReactNode }> = ({
  label,
  description,
  children,
}) => (
  <div className="flex items-center justify-between gap-4 p-4">
    <div>
      <p className="text-sm font-medium">{label}</p>
      <p className="text-xs text-muted-foreground">{description}</p>
    </div>
    {children}
  </div>
);

interface NumberFieldProps {
  id: string;
  label: string;
  value: number;
  min: number;
  max: number;
  onChange: (value: number) => void;
  /** Stored units per displayed unit, e.g. 1000 to edit milliseconds as seconds. */
  scale?: number;
}

// Keeps the typed text as a draft and clamps only on blur, so typing "2000"
// into a field with min 100 isn't turned into 100 at the first keystroke.
export const NumberField: React.FC<NumberFieldProps> = ({
  id,
  label,
  value,
  min,
  max,
  onChange,
  scale = 1,
}) => {
  const show = (stored: number) => String(stored / scale);
  const [draft, setDraft] = useState(show(value));

  useEffect(() => setDraft(String(value / scale)), [value, scale]);

  // Parses what was typed into stored units: whole numbers, as the backend
  // rejects the whole save on a decimal.
  const toStored = (text: string) => Math.round(Number(text) * scale);

  const commit = () => {
    const parsed = toStored(draft);
    const next = draft.trim() === '' || Number.isNaN(parsed) ? value : Math.min(max, Math.max(min, parsed));
    setDraft(show(next));
    if (next !== value) onChange(next);
  };

  return (
    <div className="space-y-1.5">
      <Label htmlFor={id}>{label}</Label>
      <Input
        id={id}
        type="number"
        min={min / scale}
        max={max / scale}
        step={scale === 1 ? 1 : 0.1}
        value={draft}
        onChange={(event) => {
          setDraft(event.target.value);
          const parsed = toStored(event.target.value);
          // Apply in-range values right away so Save never misses them.
          if (event.target.value.trim() !== '' && !Number.isNaN(parsed) && parsed >= min && parsed <= max) {
            onChange(parsed);
          }
        }}
        onBlur={commit}
        onKeyDown={(event) => {
          if (event.key === 'Enter') commit();
        }}
      />
    </div>
  );
};
