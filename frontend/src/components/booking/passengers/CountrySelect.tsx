"use client";

import { Check, ChevronDown } from "lucide-react";
import { motion, useReducedMotion } from "motion/react";
import { useEffect, useMemo, useRef, useState, type KeyboardEvent } from "react";

import countryCodes from "@/data/country-codes.json";
import { callingCodeCountries, countryCallingCode, preferredCountryForCallingCode } from "@/components/booking/passengers/countryCallingCodes";
import { useLanguage } from "@/i18n/LanguageProvider";
import { cn } from "@/lib/utils/cn";

type SelectorMode = "country" | "callingCode";

type Option = { code: string; label: string; value: string };

const CountrySelect = ({
  describedBy,
  error,
  id,
  label,
  mode = "country",
  onOpenChange,
  onChange,
  open,
  value,
}: {
  describedBy?: string;
  error?: boolean;
  id: string;
  label: string;
  mode?: SelectorMode;
  onChange: (value: string) => void;
  onOpenChange: (open: boolean) => void;
  open: boolean;
  value: string;
}) => {
  const { locale, t } = useLanguage();
  const prefersReducedMotion = useReducedMotion();
  const triggerRef = useRef<HTMLButtonElement>(null);
  const searchRef = useRef<HTMLInputElement>(null);
  const rootRef = useRef<HTMLDivElement>(null);
  const [query, setQuery] = useState("");
  const [activeIndex, setActiveIndex] = useState(0);
  const displayNames = useMemo(() => new Intl.DisplayNames([locale], { type: "region" }), [locale]);
  const options = useMemo(() => {
    const codes = mode === "callingCode" ? callingCodeCountries : countryCodes;
    return codes.map((code) => ({
      code,
      label: displayNames.of(code) ?? code,
      value: mode === "callingCode" ? countryCallingCode(code) ?? "" : code,
    })).sort((left, right) => left.label.localeCompare(right.label, locale));
  }, [displayNames, locale, mode]);
  const filteredOptions = useMemo(() => {
    const normalized = query.trim().toLocaleLowerCase(locale);
    if (!normalized) return options;
    return options.filter((option) => `${option.label} ${option.code} ${option.value}`.toLocaleLowerCase(locale).includes(normalized));
  }, [locale, options, query]);
  const selected = mode === "callingCode"
    ? options.find((option) => option.code === preferredCountryForCallingCode(value))
    : options.find((option) => option.value === value);
  const listboxId = `${id}-options`;

  useEffect(() => {
    if (!open) return;
    const frame = window.requestAnimationFrame(() => {
      setActiveIndex(Math.max(0, filteredOptions.findIndex((option) => option.value === value)));
      searchRef.current?.focus();
    });
    return () => window.cancelAnimationFrame(frame);
  }, [filteredOptions, open, value]);

  useEffect(() => {
    if (!open) return;
    const onPointerDown = (event: PointerEvent) => {
      if (!rootRef.current?.contains(event.target as Node)) onOpenChange(false);
    };
    document.addEventListener("pointerdown", onPointerDown);
    return () => document.removeEventListener("pointerdown", onPointerDown);
  }, [onOpenChange, open]);

  const close = (restoreFocus = true) => {
    onOpenChange(false);
    setQuery("");
    if (restoreFocus) triggerRef.current?.focus();
  };
  const select = (option: Option) => {
    onChange(option.value);
    close();
  };
  const onKeyDown = (event: KeyboardEvent<HTMLElement>) => {
    if (event.key === "Escape") {
      event.preventDefault();
      close();
      return;
    }
    if (!filteredOptions.length) return;
    if (event.key === "ArrowDown" || event.key === "ArrowUp") {
      event.preventDefault();
      setActiveIndex((index) => (index + (event.key === "ArrowDown" ? 1 : -1) + filteredOptions.length) % filteredOptions.length);
      return;
    }
    if (event.key === "Enter") {
      const option = filteredOptions[activeIndex];
      if (option) {
        event.preventDefault();
        select(option);
      }
    }
  };
  const optionLabel = (option: Option) => mode === "callingCode" ? `${option.label} ${option.value}` : option.label;

  return (
    <div className="relative" ref={rootRef}>
      <button aria-controls={open ? listboxId : undefined} aria-describedby={describedBy} aria-expanded={open} aria-haspopup="listbox" aria-label={label} className={cn("flex h-12 w-full items-center justify-between gap-3 rounded-control border border-border bg-surface px-4 text-left text-base text-foreground outline-none transition-[border-color,box-shadow,transform] duration-150 hover:border-border-strong focus-visible:border-focus focus-visible:ring-2 focus-visible:ring-focus/25 motion-safe:focus-visible:scale-[1.005] motion-reduce:transition-none", error && "border-destructive ring-2 ring-destructive/25")} id={id} onClick={() => onOpenChange(!open)} ref={triggerRef} type="button">
        <span className={cn("truncate", !selected && "text-muted-foreground")}>{selected ? optionLabel(selected) : t(mode === "callingCode" ? "passengerInformation.choosePhoneCountry" : "passengerInformation.chooseCountry")}</span>
        <ChevronDown aria-hidden="true" className={cn("size-4 shrink-0 transition-transform duration-150 motion-reduce:transition-none", open && "rotate-180")} />
      </button>
      {open ? (
          <motion.div animate={{ opacity: 1, scale: 1, y: 0 }} className="absolute z-30 mt-2 flex h-[min(26rem,60dvh)] w-full min-w-64 flex-col overflow-hidden rounded-control border border-border-strong bg-surface-elevated p-2 shadow-[0_18px_42px_rgb(0_0_0/0.35)]" initial={{ opacity: 0, scale: 0.99, y: -2 }} transition={{ duration: prefersReducedMotion ? 0 : 0.12, ease: "easeOut" }}>
            <input aria-activedescendant={filteredOptions[activeIndex] ? `${id}-option-${filteredOptions[activeIndex].code}` : undefined} aria-controls={listboxId} aria-expanded="true" aria-label={t("passengerInformation.countrySelector.searchLabel")} autoComplete="off" className="h-10 shrink-0 flex-none rounded-control border border-border bg-surface px-3 text-sm text-foreground outline-none placeholder:text-muted-foreground focus-visible:border-focus focus-visible:ring-2 focus-visible:ring-focus/25" onChange={(event) => { setQuery(event.target.value); setActiveIndex(0); }} onKeyDown={onKeyDown} placeholder={t("passengerInformation.countrySelector.searchPlaceholder")} ref={searchRef} role="combobox" value={query} />
            <div aria-label={label} className="mt-2 min-h-0 flex-1 overflow-y-auto overscroll-contain" data-lenis-prevent-wheel id={listboxId} role="listbox">
              {filteredOptions.length ? filteredOptions.map((option, index) => (
                <button aria-label={optionLabel(option)} aria-selected={option.value === value} className={cn("flex min-h-11 w-full items-center justify-between gap-3 rounded-control px-3 py-2 text-left text-sm text-foreground outline-none transition-colors hover:bg-surface-highlight focus-visible:bg-surface-highlight focus-visible:ring-2 focus-visible:ring-focus/25", index === activeIndex && "bg-surface-highlight")} id={`${id}-option-${option.code}`} key={option.code} onClick={() => select(option)} onKeyDown={onKeyDown} onMouseEnter={() => setActiveIndex(index)} role="option" type="button">
                  <span className="min-w-0 truncate">{option.label}</span>
                  <span className="flex shrink-0 items-center gap-2 text-muted-foreground">{mode === "callingCode" ? option.value : option.code}{option.value === value ? <Check aria-hidden="true" className="size-4 text-brand" /> : null}</span>
                </button>
              )) : <p className="px-3 py-4 text-sm text-muted-foreground">{t("passengerInformation.countrySelector.noResults")}</p>}
            </div>
          </motion.div>
        ) : null}
    </div>
  );
};

export { CountrySelect };
