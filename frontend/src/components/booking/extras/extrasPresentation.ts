import type { MealService } from "@/components/booking/extras/extrasTypes";
import type { TranslationKey } from "@/i18n/types";

const productLabelKeys = {
  ASSIST_HEARING: "travelExtras.products.assistHearing",
  ASSIST_MOBILITY: "travelExtras.products.assistMobility",
  ASSIST_VISUAL: "travelExtras.products.assistVisual",
  ASSIST_WHEELCHAIR: "travelExtras.products.assistWheelchair",
  BAG_10KG: "travelExtras.products.bag10",
  BAG_20KG: "travelExtras.products.bag20",
  BAG_30KG: "travelExtras.products.bag30",
  MEAL_CHILD: "travelExtras.products.mealChild",
  MEAL_HALAL: "travelExtras.products.mealHalal",
  MEAL_KOSHER: "travelExtras.products.mealKosher",
  MEAL_VEGAN: "travelExtras.products.mealVegan",
  MEAL_VEGETARIAN: "travelExtras.products.mealVegetarian",
} as const satisfies Record<string, TranslationKey>;

const mealServiceKeys = {
  ENHANCED: "travelExtras.included.mealEnhanced",
  PREMIUM: "travelExtras.included.mealPremium",
  SIGNATURE: "travelExtras.included.mealSignature",
  STANDARD: "travelExtras.included.mealStandard",
} as const satisfies Record<MealService, TranslationKey>;

const isKnownProductCode = (code: string): code is keyof typeof productLabelKeys =>
  code in productLabelKeys;

const productLabelKey = (code: string): TranslationKey =>
  isKnownProductCode(code)
    ? productLabelKeys[code]
    : "travelExtras.products.unavailable";

export { mealServiceKeys, productLabelKey, productLabelKeys };
