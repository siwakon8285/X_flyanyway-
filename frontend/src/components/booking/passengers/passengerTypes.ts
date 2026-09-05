import type { SeatHold } from "@/components/booking/seats/seatHoldClient";

type PassengerType = "ADULT" | "CHILD" | "INFANT";
type PassengerTitle = "MR" | "MS";
type PassengerGender = "MALE" | "FEMALE" | "UNSPECIFIED";

type PassengerSlot = {
  ordinal: number;
  passengerType: PassengerType;
};

type EmergencyContact = {
  name: string;
  phoneCountryCode: string;
  phoneNumber: string;
  relationship: string;
};

type Passenger = {
  dateOfBirth: string;
  email: string;
  emergencyContact: EmergencyContact | null;
  familyName: string;
  gender: PassengerGender;
  givenName: string;
  middleName: string | null;
  nationalityCode: string;
  ordinal: number;
  passengerType: PassengerType;
  passportIssuingCountryCode: string;
  passportNumber: string;
  phoneCountryCode: string;
  phoneNumber: string;
  title: PassengerTitle;
};

type BookingContact = {
  email: string;
  preferredLocale: "EN" | "TH";
};

type PassengerFormValue = Omit<Passenger, "gender" | "middleName" | "title"> & {
  gender: PassengerGender | "";
  middleName: string;
  title: PassengerTitle | "";
};

type PassengerContext = {
  bookingContact?: BookingContact | null;
  expectedPassengers: PassengerSlot[];
  hold: SeatHold;
  passengers: Passenger[];
  readyToContinue: boolean;
};

type PassengerValidationCode =
  | "AGE_CATEGORY_MISMATCH"
  | "DATE_OF_BIRTH_FUTURE"
  | "DUPLICATE_PASSPORT"
  | "EMERGENCY_CONTACT_INCOMPLETE"
  | "INVALID_COUNTRY"
  | "INVALID_DATE"
  | "INVALID_EMAIL"
  | "INVALID_NAME"
  | "INVALID_PASSPORT"
  | "INVALID_PHONE"
  | "REQUIRED";

type PassengerFieldName = keyof PassengerFormValue | "emergencyContact";

type PassengerValidationError = {
  code: PassengerValidationCode;
  field: PassengerFieldName;
  passenger: number;
};

export type {
  BookingContact,
  EmergencyContact,
  Passenger,
  PassengerContext,
  PassengerFieldName,
  PassengerFormValue,
  PassengerGender,
  PassengerSlot,
  PassengerTitle,
  PassengerType,
  PassengerValidationCode,
  PassengerValidationError,
};
