import type {
  PassengerType,
  PassengerValidationCode,
} from "@/components/booking/passengers/passengerTypes";
import type { TranslationKey } from "@/i18n/types";

const passengerTypeKeys = {
  ADULT: "passengerInformation.passenger.adult",
  CHILD: "passengerInformation.passenger.child",
  INFANT: "passengerInformation.passenger.infant",
} as const satisfies Record<PassengerType, TranslationKey>;

const validationKeys = {
  AGE_CATEGORY_MISMATCH: "passengerInformation.validation.ageCategory",
  DATE_OF_BIRTH_FUTURE: "passengerInformation.validation.futureBirth",
  DUPLICATE_PASSPORT: "passengerInformation.validation.duplicatePassport",
  EMERGENCY_CONTACT_INCOMPLETE: "passengerInformation.validation.emergencyIncomplete",
  INVALID_COUNTRY: "passengerInformation.validation.invalidCountry",
  INVALID_DATE: "passengerInformation.validation.invalidDate",
  INVALID_EMAIL: "passengerInformation.validation.invalidEmail",
  INVALID_NAME: "passengerInformation.validation.invalidName",
  INVALID_PASSPORT: "passengerInformation.validation.invalidPassport",
  INVALID_PHONE: "passengerInformation.validation.invalidPhone",
  REQUIRED: "passengerInformation.validation.required",
} as const satisfies Record<PassengerValidationCode, TranslationKey>;

export { passengerTypeKeys, validationKeys };
