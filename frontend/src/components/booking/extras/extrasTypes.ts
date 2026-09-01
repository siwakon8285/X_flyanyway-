import type { PassengerSlot, PassengerType } from "@/components/booking/passengers/passengerTypes";
import type { SeatHold } from "@/components/booking/seats/seatHoldClient";

type ExtraCategory = "ASSISTANCE" | "BAGGAGE" | "MEAL";
type MealService = "ENHANCED" | "PREMIUM" | "SIGNATURE" | "STANDARD";

type Money = {
  amount: number;
  currencyCode: string;
};

type ExtraProduct = {
  category: ExtraCategory;
  code: string;
  eligiblePassengerTypes: PassengerType[];
  maxQuantity: number;
  unitPrice: Money;
};

type ExtrasCatalog = {
  allowances: {
    appliesToPassengerTypes: PassengerType[];
    cabinBaggageKg: number;
    checkedBaggageKg: number;
  };
  currencyCode: string;
  includedBenefits: {
    mealService: MealService;
    seatSelectionIncluded: boolean;
  };
  products: ExtraProduct[];
};

type ExtraSelectionInput = {
  passengerOrdinal: number;
  productCode: string;
  quantity: number;
};

type ExtraSelection = ExtraSelectionInput & {
  category: ExtraCategory;
  lineTotal: Money;
  unitPrice: Money;
};

type ExtrasContext = {
  catalog: ExtrasCatalog;
  hold: SeatHold;
  passengers: PassengerSlot[];
  readyToContinue: boolean;
  savedAt: string | null;
  selections: ExtraSelection[];
  total: Money;
};

export type {
  ExtraCategory,
  ExtraProduct,
  ExtraSelection,
  ExtraSelectionInput,
  ExtrasCatalog,
  ExtrasContext,
  MealService,
  Money,
};
