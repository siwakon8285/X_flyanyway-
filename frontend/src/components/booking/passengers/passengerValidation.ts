import countryCodes from "@/data/country-codes.json";

import type {
  Passenger,
  PassengerFormValue,
  PassengerSlot,
  PassengerType,
  PassengerValidationCode,
  PassengerValidationError,
} from "@/components/booking/passengers/passengerTypes";

const COUNTRY_CODES = new Set<string>(countryCodes);

const createEmptyPassenger = ({
  ordinal,
  passengerType,
}: PassengerSlot): PassengerFormValue => ({
  dateOfBirth: "",
  email: "",
  emergencyContact: null,
  familyName: "",
  gender: "",
  givenName: "",
  middleName: "",
  nationalityCode: "",
  ordinal,
  passengerType,
  passportIssuingCountryCode: "",
  passportNumber: "",
  phoneCountryCode: "",
  phoneNumber: "",
  title: "",
});

const normalizeName = (value: string) => value.trim().replace(/\s+/g, " ");
const normalizeCountryCode = (value: string) => value.trim().toUpperCase();
const normalizePassport = (value: string) =>
  value.replace(/[\s-]/g, "").toUpperCase();
const normalizePhoneCountryCode = (value: string) =>
  `+${value.replace(/\D/g, "")}`;
const normalizePhoneNumber = (value: string) => value.replace(/\D/g, "");

const normalizePassengerDraft = (values: PassengerFormValue[]): Passenger[] =>
  values.map((value) => ({
    ...value,
    title: value.title as Passenger["title"],
    givenName: normalizeName(value.givenName),
    middleName: normalizeName(value.middleName) || null,
    familyName: normalizeName(value.familyName),
    gender: value.gender as Passenger["gender"],
    nationalityCode: normalizeCountryCode(value.nationalityCode),
    passportNumber: normalizePassport(value.passportNumber),
    passportIssuingCountryCode: normalizeCountryCode(
      value.passportIssuingCountryCode,
    ),
    email: value.email.trim(),
    phoneCountryCode: normalizePhoneCountryCode(value.phoneCountryCode),
    phoneNumber: normalizePhoneNumber(value.phoneNumber),
    emergencyContact: value.emergencyContact
      ? {
          name: normalizeName(value.emergencyContact.name),
          relationship: normalizeName(value.emergencyContact.relationship),
          phoneCountryCode: normalizePhoneCountryCode(
            value.emergencyContact.phoneCountryCode,
          ),
          phoneNumber: normalizePhoneNumber(value.emergencyContact.phoneNumber),
        }
      : null,
  }));

const parseDate = (value: string) => {
  const match = /^(\d{4})-(\d{2})-(\d{2})$/.exec(value);
  if (!match) return null;
  const year = Number(match[1]);
  const month = Number(match[2]);
  const day = Number(match[3]);
  const date = new Date(Date.UTC(year, month - 1, day));
  return date.getUTCFullYear() === year &&
    date.getUTCMonth() === month - 1 &&
    date.getUTCDate() === day
    ? { day, month, year }
    : null;
};

const passengerTypeAt = (dateOfBirth: string, departure: string): PassengerType | null => {
  const birth = parseDate(dateOfBirth);
  const flight = parseDate(departure);
  if (!birth || !flight) return null;
  let age = flight.year - birth.year;
  if (
    flight.month < birth.month ||
    (flight.month === birth.month && flight.day < birth.day)
  ) {
    age -= 1;
  }
  if (age >= 12) return "ADULT";
  if (age >= 2) return "CHILD";
  return "INFANT";
};

const validName = (value: string) =>
  value.length <= 100 && /^[\p{L} .\-'’]+$/u.test(value);

const validEmail = (value: string) => {
  if (value.length > 254 || /\s/.test(value)) return false;
  const parts = value.split("@");
  if (parts.length !== 2) return false;
  const [local, domain] = parts;
  return Boolean(
    local &&
      local.length <= 64 &&
      domain &&
      domain.includes(".") &&
      !domain.startsWith(".") &&
      !domain.endsWith("."),
  );
};

const validPhone = (countryCode: string, phoneNumber: string) => {
  const countryDigits = countryCode.replace(/\D/g, "");
  const numberDigits = phoneNumber.replace(/\D/g, "");
  const total = countryDigits.length + numberDigits.length;
  return (
    /^[1-9]\d{0,2}$/.test(countryDigits) &&
    numberDigits.length >= 4 &&
    total >= 7 &&
    total <= 15
  );
};

const validPhoneInput = (countryCode: string, phoneNumber: string) =>
  /^\s*\+?[\d\s-]+\s*$/.test(countryCode) && /^[\d\s().-]+$/.test(phoneNumber);

const validatePassengerDraft = (
  values: PassengerFormValue[],
  departureDate: string,
  today: string,
): PassengerValidationError[] => {
  const normalized = normalizePassengerDraft(values);
  const errors: PassengerValidationError[] = [];
  const passports = new Set<string>();
  const add = (
    passenger: number,
    field: PassengerValidationError["field"],
    code: PassengerValidationCode,
  ) => errors.push({ code, field, passenger });

  normalized.forEach((passenger, index) => {
    for (const field of ["givenName", "familyName"] as const) {
      if (!passenger[field]) add(passenger.ordinal, field, "REQUIRED");
      else if (!validName(passenger[field])) {
        add(passenger.ordinal, field, "INVALID_NAME");
      }
    }
    if (passenger.middleName && !validName(passenger.middleName)) {
      add(passenger.ordinal, "middleName", "INVALID_NAME");
    }
    if (!passenger.title) add(passenger.ordinal, "title", "REQUIRED");
    if (!passenger.gender) add(passenger.ordinal, "gender", "REQUIRED");

    if (!passenger.dateOfBirth) {
      add(passenger.ordinal, "dateOfBirth", "REQUIRED");
    } else if (!parseDate(passenger.dateOfBirth)) {
      add(passenger.ordinal, "dateOfBirth", "INVALID_DATE");
    } else if (passenger.dateOfBirth > today) {
      add(passenger.ordinal, "dateOfBirth", "DATE_OF_BIRTH_FUTURE");
    } else if (
      passengerTypeAt(passenger.dateOfBirth, departureDate) !== passenger.passengerType
    ) {
      add(passenger.ordinal, "dateOfBirth", "AGE_CATEGORY_MISMATCH");
    }

    for (const [field, code] of [
      ["nationalityCode", passenger.nationalityCode],
      ["passportIssuingCountryCode", passenger.passportIssuingCountryCode],
    ] as const) {
      if (!code) add(passenger.ordinal, field, "REQUIRED");
      else if (!COUNTRY_CODES.has(code)) add(passenger.ordinal, field, "INVALID_COUNTRY");
    }

    if (!passenger.passportNumber) {
      add(passenger.ordinal, "passportNumber", "REQUIRED");
    } else if (!/^[A-Z0-9]{3,20}$/.test(passenger.passportNumber)) {
      add(passenger.ordinal, "passportNumber", "INVALID_PASSPORT");
    } else if (passports.has(passenger.passportNumber)) {
      add(passenger.ordinal, "passportNumber", "DUPLICATE_PASSPORT");
    } else {
      passports.add(passenger.passportNumber);
    }

    if (!passenger.email) add(passenger.ordinal, "email", "REQUIRED");
    else if (!validEmail(passenger.email)) add(passenger.ordinal, "email", "INVALID_EMAIL");
    if (!passenger.phoneCountryCode) {
      add(passenger.ordinal, "phoneCountryCode", "REQUIRED");
    }
    if (!passenger.phoneNumber) add(passenger.ordinal, "phoneNumber", "REQUIRED");
    else if (
      !validPhoneInput(values[index]?.phoneCountryCode ?? "", values[index]?.phoneNumber ?? "") ||
      !validPhone(passenger.phoneCountryCode, passenger.phoneNumber)
    ) {
      add(passenger.ordinal, "phoneNumber", "INVALID_PHONE");
    }

    if (passenger.emergencyContact) {
      const contact = passenger.emergencyContact;
      if (
        !validName(contact.name) ||
        !validName(contact.relationship) ||
        !validPhoneInput(
          values[index]?.emergencyContact?.phoneCountryCode ?? "",
          values[index]?.emergencyContact?.phoneNumber ?? "",
        ) ||
        !validPhone(contact.phoneCountryCode, contact.phoneNumber)
      ) {
        add(passenger.ordinal, "emergencyContact", "EMERGENCY_CONTACT_INCOMPLETE");
      }
    }
  });

  return errors;
};

export {
  createEmptyPassenger,
  normalizePassengerDraft,
  validatePassengerDraft,
};
