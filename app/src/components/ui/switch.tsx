import React from 'react';

interface SwitchProps {
  id: string;
  checked: boolean;
  onChange: () => void;
  label?: string;
}

export const Switch: React.FC<SwitchProps> = ({ id, checked, onChange, label }) => (
  <button
    id={id}
    type="button"
    role="switch"
    aria-checked={checked}
    aria-label={label ?? id}
    onClick={onChange}
    className={
      checked
        ? 'relative inline-flex h-6 w-11 shrink-0 rounded-full border-2 border-transparent transition-colors active:scale-[.97] focus-visible:outline-none focus-visible:ring-2 focus-visible:ring-ring bg-primary'
        : 'relative inline-flex h-6 w-11 shrink-0 rounded-full border-2 border-transparent transition-colors active:scale-[.97] focus-visible:outline-none focus-visible:ring-2 focus-visible:ring-ring bg-input'
    }
  >
    <span
      className={
        checked
          ? 'pointer-events-none block h-5 w-5 rounded-full bg-background shadow transition-transform duration-150 ease-out translate-x-5 rtl:-translate-x-5'
          : 'pointer-events-none block h-5 w-5 rounded-full bg-background shadow transition-transform duration-150 ease-out'
      }
    />
  </button>
);
