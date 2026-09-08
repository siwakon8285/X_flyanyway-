"use client";

import { Clock3 } from "lucide-react";
import { useId, useRef } from "react";

type TimePickerFieldProps = {
  disabled?: boolean;
  label: string;
  onChange: (value: string) => void;
  required?: boolean;
  value: string;
};

const TimePickerField = ({
  disabled = false,
  label,
  onChange,
  required = false,
  value,
}: TimePickerFieldProps) => {
  const input = useRef<HTMLInputElement>(null);
  const inputId = useId();
  const openPicker = () => {
    if (disabled) return;
    input.current?.focus();
    try {
      input.current?.showPicker?.();
    } catch {
      // Native focus and keyboard entry remain the cross-browser fallback.
    }
  };

  return (
    <div className="xfo-field text-sm font-semibold">
      <label htmlFor={inputId}>{label}</label>
      <div
        className="relative"
        data-time-picker-control
        onClick={(event) => {
          if (event.target === event.currentTarget) openPicker();
        }}
      >
        <Clock3
          aria-hidden="true"
          className="pointer-events-none absolute left-3 top-1/2 z-10 size-5 -translate-y-1/2 text-black/70"
          data-clock-icon
        />
        <input
          aria-label={label}
          className="xfo-control xfo-time-input w-full cursor-pointer font-normal disabled:cursor-not-allowed disabled:bg-black/5"
          disabled={disabled}
          id={inputId}
          onChange={(event) => onChange(event.target.value)}
          onClick={openPicker}
          ref={input}
          required={required}
          type="time"
          value={value}
        />
      </div>
    </div>
  );
};

export { TimePickerField };
export type { TimePickerFieldProps };
