"use client";

import { CalendarDays } from "lucide-react";
import { useId, useRef } from "react";
import { cn } from "@/lib/utils/cn";

type DatePickerFieldProps = {
  className?: string;
  disabled?: boolean;
  label: string;
  onChange: (value: string) => void;
  required?: boolean;
  value: string;
};

const DatePickerField = ({
  disabled = false,
  className,
  label,
  onChange,
  required = false,
  value,
}: DatePickerFieldProps) => {
  const input = useRef<HTMLInputElement>(null);
  const inputId = useId();
  const openPicker = () => {
    if (disabled) return;
    input.current?.focus();
    try {
      input.current?.showPicker?.();
    } catch {
      // Focusing the native date input is the cross-browser fallback.
    }
  };

  return (
    <div className={cn("xfo-field text-sm font-semibold", className)}>
      <label htmlFor={inputId}>{label}</label>
      <div
        className="relative"
        data-date-picker-control
        onClick={(event) => {
          if (event.target === event.currentTarget) openPicker();
        }}
      >
        <CalendarDays
          aria-hidden="true"
          className="pointer-events-none absolute left-3 top-1/2 z-10 size-5 -translate-y-1/2 text-black/70"
          data-calendar-icon
        />
        <input
          aria-label={label}
          className="xfo-control xfo-date-input w-full cursor-pointer font-normal disabled:cursor-not-allowed disabled:bg-black/5"
          disabled={disabled}
          id={inputId}
          onChange={(event) => onChange(event.target.value)}
          onClick={openPicker}
          ref={input}
          required={required}
          type="date"
          value={value}
        />
      </div>
    </div>
  );
};

export { DatePickerField };
export type { DatePickerFieldProps };
