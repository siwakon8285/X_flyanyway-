import {
  createEmptyPassenger,
  normalizePassengerDraft,
  normalizeBookingContact,
  validateBookingContact,
  validatePassengerDraft,
} from "@/components/booking/passengers/passengerValidation";
import type { PassengerFormValue } from "@/components/booking/passengers/passengerTypes";

const validPassenger = (overrides: Partial<PassengerFormValue> = {}): PassengerFormValue => ({
  ...createEmptyPassenger({ ordinal: 1, passengerType: "ADULT" }),
  title: "MS",
  givenName: "  Nara  ",
  familyName: " Suri ",
  dateOfBirth: "1990-01-01",
  gender: "FEMALE",
  nationalityCode: "th",
  passportNumber: " th-123 456 ",
  passportIssuingCountryCode: "th",
  ...overrides,
});

describe("passenger validation", () => {
  it("normalizes safe transport fields without changing identity values semantically", () => {
    const normalized = normalizePassengerDraft([validPassenger()]);

    expect(normalized[0]).toMatchObject({
      givenName: "Nara",
      familyName: "Suri",
      nationalityCode: "TH",
      passportNumber: "TH123456",
      passportIssuingCountryCode: "TH",
    });
  });

  it("reports required passenger, passport, and country values", () => {
    const errors = validatePassengerDraft(
      [
        validPassenger({
          givenName: " ",
          passportNumber: "@@",
          nationalityCode: "ZZ",
        }),
      ],
      "2030-05-10",
      "2026-08-31",
    );

    expect(errors).toEqual(
      expect.arrayContaining([
        expect.objectContaining({ field: "givenName", code: "REQUIRED" }),
        expect.objectContaining({ field: "passportNumber", code: "INVALID_PASSPORT" }),
        expect.objectContaining({ field: "nationalityCode", code: "INVALID_COUNTRY" }),
      ]),
    );
  });

  it("validates and normalizes the phone-only booking contact", () => {
    expect(normalizeBookingContact("66", "081-234-5678")).toEqual({
      phoneCountryCode: "+66",
      phoneNumber: "0812345678",
    });
    expect(validateBookingContact("+66", "0812345678")).toBe(true);
    expect(validateBookingContact("+6x6", "0812CALL345678")).toBe(false);
    expect(validateBookingContact("", "0812345678")).toBe(false);
    expect(validateBookingContact("+66", "")).toBe(false);
  });

  it("uses exact adult, child, and infant boundaries on outbound departure", () => {
    const passengers = [
      validPassenger({ ordinal: 1, passengerType: "ADULT", dateOfBirth: "2018-06-15" }),
      validPassenger({
        ordinal: 2,
        passengerType: "CHILD",
        dateOfBirth: "2018-06-16",
        passportNumber: "CH123456",
      }),
      validPassenger({
        ordinal: 3,
        passengerType: "INFANT",
        dateOfBirth: "2028-06-16",
        passportNumber: "IN123456",
      }),
    ];

    expect(validatePassengerDraft(passengers, "2030-06-15", "2030-06-01")).toEqual([]);
    expect(
      validatePassengerDraft(
        [validPassenger({ dateOfBirth: "2018-06-16" })],
        "2030-06-15",
        "2030-06-01",
      ),
    ).toEqual(
      expect.arrayContaining([
        expect.objectContaining({ field: "dateOfBirth", code: "AGE_CATEGORY_MISMATCH" }),
      ]),
    );
  });

  it("does not require passport dates while retaining identity and emergency-contact validation", () => {
    const first = validPassenger({
      dateOfBirth: "2026-09-01",
      emergencyContact: {
        name: "Sam Lee",
        relationship: "",
        phoneCountryCode: "+66",
        phoneNumber: "0812345678",
      },
    });
    const second = validPassenger({
      ordinal: 2,
      passengerType: "ADULT",
    });
    const errors = validatePassengerDraft([first, second], "2030-05-10", "2026-08-31");
    const codes = errors.map((error) => error.code);

    expect(codes).toEqual(
      expect.arrayContaining([
        "DATE_OF_BIRTH_FUTURE",
        "DUPLICATE_PASSPORT",
        "EMERGENCY_CONTACT_INCOMPLETE",
      ]),
    );
    expect(codes).not.toEqual(expect.arrayContaining([
      "PASSPORT_EXPIRES_BEFORE_DEPARTURE",
      "PASSPORT_ISSUE_AFTER_EXPIRY",
    ]));
  });

  it("accepts Male and Female but rejects a forged Unspecified value", () => {
    expect(validatePassengerDraft([validPassenger({ gender: "MALE" })], "2030-05-10", "2026-08-31")).toEqual([]);
    expect(validatePassengerDraft([validPassenger({ gender: "FEMALE" })], "2030-05-10", "2026-08-31")).toEqual([]);
    const forged = validPassenger({ gender: "UNSPECIFIED" as PassengerFormValue["gender"] });
    expect(validatePassengerDraft([forged], "2030-05-10", "2026-08-31")).toEqual(
      expect.arrayContaining([expect.objectContaining({ field: "gender", code: "INVALID_GENDER" })]),
    );
  });
});
