import type {
  ExtraSelection,
  MealService,
  Money,
} from "@/components/booking/extras/extrasTypes";
import type {
  PassengerTitle,
  PassengerType,
} from "@/components/booking/passengers/passengerTypes";
import type { SeatHold } from "@/components/booking/seats/seatHoldClient";

type ReviewJourney = {
  aircraftCode: string;
  arrivalDayOffset: number;
  arrivalTime: string;
  departureTime: string;
  destinationCode: string;
  durationMinutes: number;
  flightNumber: string;
  originCode: string;
  stops: "DIRECT" | "ONE_STOP";
};

type ReviewPassenger = {
  displayName: string;
  extras: ExtraSelection[];
  nationalityCode: string;
  ordinal: number;
  passengerType: PassengerType;
  title: PassengerTitle;
  travelDocumentComplete: boolean;
};

type ReviewPricingLine = {
  amount: Money;
  code: "DEMO_AIRPORT_FEE" | "DEMO_BOOKING_FEE" | "DEMO_PASSENGER_TAX";
  quantity: number;
  unitAmount: Money;
};

type ReviewPricing = {
  baseFare: {
    amount: Money;
    lines: Array<{
      amount: Money;
      passengerType: PassengerType;
      quantity: number;
      unitAmount: Money;
    }>;
  };
  currencyCode: string;
  extras: Money;
  fees: ReviewPricingLine[];
  grandTotal: Money;
  pricedAt: string;
  taxes: ReviewPricingLine[];
};

type ReviewContext = {
  fareConditions: {
    changesAllowed: boolean;
    code: "DEMO_FIXTURE_NONREFUNDABLE_NO_CHANGES";
    fixture: boolean;
    refundable: boolean;
  };
  hold: SeatHold;
  includedBenefits: {
    allowances: {
      appliesToPassengerTypes: PassengerType[];
      cabinBaggageKg: number;
      checkedBaggageKg: number;
    };
    benefits: {
      mealService: MealService;
      seatSelectionIncluded: boolean;
    };
  };
  journey: ReviewJourney;
  passengers: ReviewPassenger[];
  pricing: ReviewPricing;
  readyForPayment: boolean;
  seats: Array<{ seatNumber: string }>;
};

export type {
  ReviewContext,
  ReviewJourney,
  ReviewPassenger,
  ReviewPricing,
  ReviewPricingLine,
};
