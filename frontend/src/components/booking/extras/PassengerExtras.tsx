import type {
  ExtraCategory,
  ExtraProduct,
  ExtraSelectionInput,
} from "@/components/booking/extras/extrasTypes";
import { productLabelKey } from "@/components/booking/extras/extrasPresentation";
import type { PassengerSlot } from "@/components/booking/passengers/passengerTypes";
import { formatPrice } from "@/i18n/formatters";
import { useLanguage } from "@/i18n/LanguageProvider";
import { cn } from "@/lib/utils/cn";

const PassengerExtras = ({
  onChange,
  passenger,
  products,
  selections,
}: {
  onChange: (selections: ExtraSelectionInput[]) => void;
  passenger: PassengerSlot;
  products: ExtraProduct[];
  selections: ExtraSelectionInput[];
}) => {
  const { locale, t } = useLanguage();
  const passengerLabel = t("passengerInformation.passenger.label", {
    ordinal: passenger.ordinal,
    type: t(
      passenger.passengerType === "ADULT"
        ? "passengerInformation.passenger.adult"
        : passenger.passengerType === "CHILD"
          ? "passengerInformation.passenger.child"
          : "passengerInformation.passenger.infant",
    ),
  });
  const passengerSelections = selections.filter(
    (selection) => selection.passengerOrdinal === passenger.ordinal,
  );
  const selectedCode = (category: ExtraCategory) =>
    passengerSelections.find((selection) => {
      const product = products.find((item) => item.code === selection.productCode);
      return product?.category === category;
    })?.productCode;
  const productsFor = (category: ExtraCategory) =>
    products.filter(
      (product) =>
        product.category === category &&
        product.eligiblePassengerTypes.includes(passenger.passengerType),
    );
  const replaceCategory = (category: ExtraCategory, productCode: string | null) => {
    const withoutCategory = selections.filter((selection) => {
      if (selection.passengerOrdinal !== passenger.ordinal) return true;
      return products.find((product) => product.code === selection.productCode)?.category !== category;
    });
    onChange(
      productCode
        ? [
            ...withoutCategory,
            { passengerOrdinal: passenger.ordinal, productCode, quantity: 1 },
          ]
        : withoutCategory,
    );
  };
  const toggleAssistance = (productCode: string, checked: boolean) => {
    const withoutProduct = selections.filter(
      (selection) =>
        selection.passengerOrdinal !== passenger.ordinal ||
        selection.productCode !== productCode,
    );
    onChange(
      checked
        ? [
            ...withoutProduct,
            { passengerOrdinal: passenger.ordinal, productCode, quantity: 1 },
          ]
        : withoutProduct,
    );
  };

  if (passenger.passengerType === "INFANT") {
    return (
      <section className="mt-6 rounded-surface border border-border bg-surface/55 p-6" aria-label={passengerLabel}>
        <p className="text-caption text-brand">{passengerLabel}</p>
        <h2 className="mt-2 text-h3">{t("travelExtras.infant.heading")}</h2>
        <p className="mt-4 max-w-xl text-body text-muted-foreground">
          {t("travelExtras.infant.body")}
        </p>
      </section>
    );
  }

  const baggage = productsFor("BAGGAGE");
  const meals = productsFor("MEAL");
  const assistance = productsFor("ASSISTANCE");
  const choiceClass = (selected: boolean) =>
    cn(
      "flex min-h-12 cursor-pointer items-center gap-3 rounded-control border px-4 py-3 text-sm outline-none transition-[border-color,background-color,box-shadow,transform] duration-200 hover:bg-surface-elevated focus-within:border-focus focus-within:ring-2 focus-within:ring-focus/25 motion-safe:hover:-translate-y-0.5 motion-safe:active:translate-y-0 motion-reduce:transition-none",
      selected
        ? "border-brand/70 bg-brand/10 text-foreground shadow-[inset_0_1px_0_rgb(255_255_255/0.025)] motion-safe:scale-[1.005]"
        : "border-border bg-surface hover:border-brand/30",
    );

  return (
    <section className="mt-6 rounded-surface border border-border bg-surface/55 p-5 sm:p-7" aria-label={passengerLabel}>
      <p className="text-caption text-brand">{passengerLabel}</p>
      <h2 className="mt-2 text-h3">{t("travelExtras.passenger.heading")}</h2>

      <fieldset className="mt-7">
        <legend className="text-xl font-semibold">
          {t("travelExtras.category.baggage")}
          <span className="sr-only"> {t("travelExtras.forPassenger", { passenger: passengerLabel })}</span>
        </legend>
        <p className="mt-2 text-sm text-muted-foreground">{t("travelExtras.baggage.note")}</p>
        <div className="mt-4 grid gap-3 sm:grid-cols-2">
          <label
            className={choiceClass(!selectedCode("BAGGAGE"))}
            data-selected={!selectedCode("BAGGAGE")}
          >
            <input
              className="accent-brand"
              checked={!selectedCode("BAGGAGE")}
              name={`baggage-${passenger.ordinal}`}
              onChange={() => replaceCategory("BAGGAGE", null)}
              type="radio"
            />
            {t("travelExtras.baggage.none")}
          </label>
          {baggage.map((product) => {
            const selected = selectedCode("BAGGAGE") === product.code;
            return (
              <label
                className={choiceClass(selected)}
                data-selected={selected}
                key={product.code}
              >
                <input
                  className="accent-brand"
                  checked={selected}
                  name={`baggage-${passenger.ordinal}`}
                  onChange={() => replaceCategory("BAGGAGE", product.code)}
                  type="radio"
                />
                <span>{t(productLabelKey(product.code))}</span>
                <span aria-hidden="true">—</span>
                <span className="ml-auto font-semibold text-brand">
                  {formatPrice(product.unitPrice.amount, locale)}
                </span>
              </label>
            );
          })}
        </div>
      </fieldset>

      <fieldset className="mt-9 border-t border-border pt-7">
        <legend className="text-xl font-semibold">{t("travelExtras.category.meal")}</legend>
        <p className="mt-2 text-sm text-muted-foreground">{t("travelExtras.meal.note")}</p>
        <div className="mt-4 grid gap-3 sm:grid-cols-2">
          <label
            className={choiceClass(!selectedCode("MEAL"))}
            data-selected={!selectedCode("MEAL")}
          >
            <input
              className="accent-brand"
              checked={!selectedCode("MEAL")}
              name={`meal-${passenger.ordinal}`}
              onChange={() => replaceCategory("MEAL", null)}
              type="radio"
            />
            {t("travelExtras.meal.none")}
          </label>
          {meals.map((product) => {
            const selected = selectedCode("MEAL") === product.code;
            return (
              <label
                className={choiceClass(selected)}
                data-selected={selected}
                key={product.code}
              >
                <input
                  className="accent-brand"
                  checked={selected}
                  name={`meal-${passenger.ordinal}`}
                  onChange={() => replaceCategory("MEAL", product.code)}
                  type="radio"
                />
                {t(productLabelKey(product.code))}
              </label>
            );
          })}
        </div>
      </fieldset>

      <fieldset className="mt-9 border-t border-border pt-7">
        <legend className="text-xl font-semibold">{t("travelExtras.category.assistance")}</legend>
        <p className="mt-2 text-sm text-muted-foreground">{t("travelExtras.assistance.note")}</p>
        <div className="mt-4 grid gap-3 sm:grid-cols-2">
          {assistance.map((product) => {
            const selected = passengerSelections.some(
              (selection) => selection.productCode === product.code,
            );
            return (
              <label
                className={choiceClass(selected)}
                data-selected={selected}
                key={product.code}
              >
                <input
                  className="accent-brand"
                  checked={selected}
                  onChange={(event) => toggleAssistance(product.code, event.target.checked)}
                  type="checkbox"
                />
                {t(productLabelKey(product.code))}
              </label>
            );
          })}
        </div>
      </fieldset>
    </section>
  );
};

export { PassengerExtras };
