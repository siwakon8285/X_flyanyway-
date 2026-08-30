"use client";

import { Armchair } from "lucide-react";

import type { CabinClass } from "@/components/booking/search/searchTypes";
import {
  Select,
  SelectContent,
  SelectItem,
  SelectTrigger,
  SelectValue,
} from "@/components/ui/Select";
import { useLanguage } from "@/i18n/LanguageProvider";
import type { TranslationKey } from "@/i18n/types";

type CabinSelectorProps = {
  onChange: (cabin: CabinClass) => void;
  value: CabinClass;
};

const cabins = [
  { labelKey: "common.cabins.economy", value: "economy" },
  { labelKey: "common.cabins.premiumEconomy", value: "premium-economy" },
  { labelKey: "common.cabins.business", value: "business" },
  { labelKey: "common.cabins.first", value: "first" },
] as const satisfies readonly { labelKey: TranslationKey; value: CabinClass }[];

const CabinSelector = ({ onChange, value }: CabinSelectorProps) => {
  const { t } = useLanguage();
  const handleChange = (nextValue: string) => {
    const cabin = cabins.find((option) => option.value === nextValue);
    if (cabin) onChange(cabin.value);
  };

  return (
    <div className="group min-h-24 min-w-0 rounded-control border border-border bg-surface/45 px-4 py-4 shadow-[inset_0_1px_0_rgb(255_255_255/0.02)] transition-[background-color,border-color,box-shadow,transform] duration-200 hover:border-brand/40 hover:bg-surface/75 hover:shadow-[0_8px_24px_rgb(255_212_0/0.045)] motion-safe:hover:-translate-y-0.5 motion-safe:hover:scale-[1.01] focus-within:border-focus focus-within:bg-surface/75 focus-within:ring-2 focus-within:ring-focus/35 focus-within:ring-offset-2 focus-within:ring-offset-background motion-reduce:transition-none">
      <span className="flex items-center justify-between gap-3 text-label text-muted-foreground">
        {t("flightSearch.cabin")}
        <Armchair
          aria-hidden="true"
          className="size-4 text-muted-foreground transition-colors group-hover:text-brand group-focus-within:text-brand"
        />
      </span>
      <Select onValueChange={handleChange} value={value}>
        <SelectTrigger
          aria-label={t("flightSearch.cabinClass")}
          className="mt-2 h-10 cursor-pointer border-0 bg-transparent px-0 text-base shadow-none hover:bg-transparent focus-visible:ring-0"
        >
          <SelectValue />
        </SelectTrigger>
        <SelectContent>
          {cabins.map((cabin) => (
            <SelectItem key={cabin.value} value={cabin.value}>
              {t(cabin.labelKey)}
            </SelectItem>
          ))}
        </SelectContent>
      </Select>
    </div>
  );
};

export { CabinSelector };
export type { CabinSelectorProps };
