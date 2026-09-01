import {
  createEmptyPassenger,
  normalizePassengerDraft,
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
  email: " nara@example.com ",
  phoneCountryCode: "66",
  phoneNumber: "081-234-5678",
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
      phoneCountryCode: "+66",
      phoneNumber: "0812345678",
    });
  });

  it("reports required fields and malformed email, phone, passport, and country values", () => {
    const errors = validatePassengerDraft(
      [
        validPassenger({
          givenName: " ",
          email: "invalid",
          phoneNumber: "12",
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
        expect.objectContaining({ field: "email", code: "INVALID_EMAIL" }),
        expect.objectContaining({ field: "phoneNumber", code: "INVALID_PHONE" }),
        expect.objectContaining({ field: "passportNumber", code: "INVALID_PASSPORT" }),
        expect.objectContaining({ field: "nationalityCode", code: "INVALID_COUNTRY" }),
      ]),
    );
  });

  it("rejects letters in phone fields instead of normalizing them away", () => {
    const errors = validatePassengerDraft(
      [
        validPassenger({
          phoneCountryCode: "+6x6",
          phoneNumber: "0812CALL345678",
        }),
      ],
      "2030-05-10",
      "2026-08-31",
    );

    expect(errors).toEqual(
      expect.arrayContaining([
        expect.objectContaining({ field: "phoneNumber", code: "INVALID_PHONE" }),
      ]),
    );
  });

  it("uses exact adult, child, and infant boundaries on outbound departure", () => {
    const passengers = [
      validPassenger({ ordinal: 1, passengerType: "ADULT", dateOfBirth: "2018-06-15" }),
      validPassenger({
        ordinal: 2,
        passengerType: "CHILD",
        dateOfBirth: "2018-06-16",
        email: "child@example.com",
        passportNumber: "CH123456",
      }),
      validPassenger({
        ordinal: 3,
        passengerType: "INFANT",
        dateOfBirth: "2028-06-16",
        email: "infant@example.com",
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
      email: "second@example.com",
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
});
