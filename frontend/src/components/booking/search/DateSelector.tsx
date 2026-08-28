import { CalendarDays } from "lucide-react";

import { cn } from "@/lib/utils/cn";

type DateSelectorProps = {
  error?: string;
  id: string;
  label: string;
  min: string;
  onChange: (value: string) => void;
  value: string;
};

const DateSelector = ({ error, id, label, min, onChange, value }: DateSelectorProps) => {
  const errorId = `${id}-error`;

  return (
    <div className="min-w-0">
      <label
        className={cn(
          "group flex min-h-24 cursor-pointer flex-col justify-between rounded-control border border-border bg-surface/45 px-4 py-4 shadow-[inset_0_1px_0_rgb(255_255_255/0.02)] transition-[background-color,border-color,box-shadow,transform] duration-200 hover:border-brand/40 hover:bg-surface/75 hover:shadow-[0_8px_24px_rgb(255_212_0/0.045)] motion-safe:hover:-translate-y-0.5 motion-safe:hover:scale-[1.01] motion-safe:active:translate-y-0 motion-safe:active:scale-[0.99] focus-within:border-focus focus-within:bg-surface/75 focus-within:ring-2 focus-within:ring-focus/35 focus-within:ring-offset-2 focus-within:ring-offset-background motion-reduce:transition-none",
          error && "border-destructive",
        )}
        htmlFor={id}
      >
        <span className="flex items-center justify-between gap-3 text-label text-muted-foreground">
          {label}
          <CalendarDays
            aria-hidden="true"
            className="size-4 text-muted-foreground transition-colors group-hover:text-brand group-focus-within:text-brand"
          />
        </span>
        <input
          aria-describedby={error ? errorId : undefined}
          aria-invalid={Boolean(error)}
          className="mt-3 min-h-9 min-w-0 cursor-pointer bg-transparent text-base text-foreground outline-none [color-scheme:dark]"
          id={id}
          min={min}
          onChange={(event) => onChange(event.target.value)}
          type="date"
          value={value}
        />
      </label>
      {error ? (
        <p className="mt-2 text-body-sm text-destructive" id={errorId}>
          {error}
        </p>
      ) : null}
    </div>
  );
};

export { DateSelector };
export type { DateSelectorProps };
