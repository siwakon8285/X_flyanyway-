"use client";

import { Check, ChevronsUpDown, Search } from "lucide-react";
import { useId, useMemo, useRef, useState, type KeyboardEvent } from "react";

import { useLanguage } from "@/i18n/LanguageProvider";
import type { AirportReference } from "@/lib/admin/flightTypes";
import { cn } from "@/lib/utils/cn";

type AirportComboboxProps = {
  airports: readonly AirportReference[];
  disabled?: boolean;
  label: string;
  onChange: (code: string) => void;
  value: string;
};

const AirportCombobox = ({
  airports,
  disabled = false,
  label,
  onChange,
  value,
}: AirportComboboxProps) => {
  const { t } = useLanguage();
  const inputId = useId();
  const listboxId = useId();
  const control = useRef<HTMLDivElement>(null);
  const [open, setOpen] = useState(false);
  const [query, setQuery] = useState("");
  const [activeIndex, setActiveIndex] = useState(0);
  const [placement, setPlacement] = useState<"top" | "bottom">("bottom");
  const [listHeight, setListHeight] = useState(288);
  const selected = airports.find((airport) => airport.code === value);
  const selectedLabel = selected
    ? `${selected.code} · ${selected.city} · ${selected.countryName}`
    : "";
  const normalizedQuery = query.trim().toLocaleLowerCase();
  const filtered = useMemo(
    () =>
      airports.filter((airport) =>
        [airport.code, airport.name, airport.city, airport.countryName].some((field) =>
          field.toLocaleLowerCase().includes(normalizedQuery),
        ),
      ),
    [airports, normalizedQuery],
  );
  const boundedIndex = filtered.length
    ? Math.min(activeIndex, filtered.length - 1)
    : 0;

  const openList = () => {
    if (disabled) return;
    const bounds = control.current?.getBoundingClientRect();
    if (bounds) {
      const below = window.innerHeight - bounds.bottom - 16;
      const above = bounds.top - 16;
      const nextPlacement = below < 192 && above > below ? "top" : "bottom";
      const available = nextPlacement === "top" ? above : below;
      setPlacement(nextPlacement);
      setListHeight(Math.max(96, Math.min(288, available)));
    }
    const selectedIndex = filtered.findIndex((airport) => airport.code === value);
    setActiveIndex(selectedIndex >= 0 ? selectedIndex : 0);
    setOpen(true);
  };
  const choose = (airport: AirportReference) => {
    onChange(airport.code);
    setQuery("");
    setOpen(false);
  };
  const onKeyDown = (event: KeyboardEvent<HTMLInputElement>) => {
    if (event.key === "ArrowDown" || event.key === "ArrowUp") {
      event.preventDefault();
      if (!open) {
        openList();
        return;
      }
      if (!filtered.length) return;
      const delta = event.key === "ArrowDown" ? 1 : -1;
      setActiveIndex((boundedIndex + delta + filtered.length) % filtered.length);
    } else if (event.key === "Enter" && open && filtered[boundedIndex]) {
      event.preventDefault();
      choose(filtered[boundedIndex]);
    } else if (event.key === "Escape" && open) {
      event.preventDefault();
      setQuery("");
      setOpen(false);
    }
  };

  return (
    <div className="xfo-field relative min-w-0 text-sm font-semibold">
      <label htmlFor={inputId}>{label}</label>
      <div className="relative" ref={control}>
        <Search
          aria-hidden="true"
          className="pointer-events-none absolute left-3 top-1/2 z-10 size-4 -translate-y-1/2 text-black/55"
        />
        <input
          aria-activedescendant={
            open && filtered[boundedIndex]
              ? `${listboxId}-${filtered[boundedIndex].code}`
              : undefined
          }
          aria-autocomplete="list"
          aria-controls={listboxId}
          aria-expanded={open}
          aria-haspopup="listbox"
          aria-label={label}
          aria-required="true"
          autoComplete="off"
          className="xfo-control xfo-airport-input w-full font-normal placeholder:text-black/45 disabled:cursor-not-allowed disabled:bg-black/5"
          disabled={disabled}
          id={inputId}
          onBlur={() => window.setTimeout(() => setOpen(false), 0)}
          onChange={(event) => {
            setQuery(event.target.value);
            setActiveIndex(0);
            setOpen(true);
          }}
          onClick={openList}
          onFocus={openList}
          onKeyDown={onKeyDown}
          placeholder={t(open ? "flightManagement.searchAirport" : "flightManagement.chooseAirport")}
          role="combobox"
          value={open ? query : selectedLabel}
        />
        <ChevronsUpDown
          aria-hidden="true"
          className="pointer-events-none absolute right-3 top-1/2 size-4 -translate-y-1/2 text-black/55"
        />
      </div>
      {open ? (
        <div
          className={cn(
            "absolute inset-x-0 z-50 max-h-[min(18rem,45dvh)] overflow-y-auto overscroll-contain rounded-xl border border-black/15 bg-white p-1 shadow-[0_18px_45px_rgb(0_0_0/0.2)]",
            placement === "top" ? "bottom-full mb-2" : "top-full mt-2",
          )}
          data-airport-options
          id={listboxId}
          role="listbox"
          style={{ maxHeight: listHeight }}
        >
          {filtered.length ? (
            filtered.map((airport, index) => (
              <button
                aria-selected={airport.code === value}
                className={cn(
                  "flex min-h-14 w-full items-center gap-3 rounded-lg px-3 py-2 text-left outline-none transition-colors hover:bg-[#ffd400]/15 focus-visible:ring-2 focus-visible:ring-inset focus-visible:ring-focus motion-reduce:transition-none",
                  index === boundedIndex && "bg-[#ffd400]/20",
                )}
                id={`${listboxId}-${airport.code}`}
                key={airport.code}
                onMouseDown={(event) => event.preventDefault()}
                onMouseEnter={() => setActiveIndex(index)}
                onClick={() => choose(airport)}
                role="option"
                tabIndex={-1}
                type="button"
              >
                <span className="w-12 shrink-0 font-mono text-base font-bold">
                  {airport.code}
                </span>
                <span className="min-w-0 flex-1">
                  <span className="block truncate font-semibold">
                    {airport.city} · {airport.countryName}
                  </span>
                  <span className="block truncate text-xs font-normal text-black/55">
                    {airport.name}
                  </span>
                </span>
                {airport.code === value ? (
                  <Check aria-hidden="true" className="size-4 shrink-0" />
                ) : null}
              </button>
            ))
          ) : (
            <p className="px-3 py-6 text-center font-normal text-black/55" role="status">
              {t("flightManagement.noAirportsFound")}
            </p>
          )}
        </div>
      ) : null}
    </div>
  );
};

export { AirportCombobox };
export type { AirportComboboxProps };
