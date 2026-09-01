import { BriefcaseBusiness, Luggage, Utensils, Armchair } from "lucide-react";

import { mealServiceKeys } from "@/components/booking/extras/extrasPresentation";
import type { ExtrasCatalog } from "@/components/booking/extras/extrasTypes";
import { useLanguage } from "@/i18n/LanguageProvider";

const IncludedBenefits = ({ catalog }: { catalog: ExtrasCatalog }) => {
  const { t } = useLanguage();
  const benefits = [
    {
      copy: t("travelExtras.included.cabinBaggage", {
        kg: catalog.allowances.cabinBaggageKg,
      }),
      icon: BriefcaseBusiness,
    },
    {
      copy: t("travelExtras.included.checkedBaggage", {
        kg: catalog.allowances.checkedBaggageKg,
      }),
      icon: Luggage,
    },
    {
      copy: t("travelExtras.included.seatSelection"),
      icon: Armchair,
    },
    {
      copy: t(mealServiceKeys[catalog.includedBenefits.mealService]),
      icon: Utensils,
    },
  ];

  return (
    <section
      aria-labelledby="included-benefits-heading"
      className="border-y border-border py-7"
      data-extras-reveal
    >
      <p className="text-caption text-brand">{t("travelExtras.included.eyebrow")}</p>
      <h2 className="mt-2 text-h3" id="included-benefits-heading">
        {t("travelExtras.included.heading")}
      </h2>
      <div className="mt-6 grid gap-4 sm:grid-cols-2">
        {benefits.map(({ copy, icon: Icon }) => (
          <div className="flex min-h-16 items-center gap-3" key={copy}>
            <span className="grid size-10 shrink-0 place-items-center rounded-full border border-brand/35 bg-brand/10 text-brand">
              <Icon aria-hidden="true" className="size-5" />
            </span>
            <span className="text-sm font-medium">{copy}</span>
          </div>
        ))}
      </div>
      <p className="mt-5 text-sm text-muted-foreground">
        {t("travelExtras.included.appliesToSeated")}
      </p>
    </section>
  );
};

export { IncludedBenefits };
