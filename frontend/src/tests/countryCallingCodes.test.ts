import {
  callingCodeForCountry,
  callingCodeOptions,
} from "@/components/booking/passengers/countryOptions";

describe("country calling-code data", () => {
  it.each([
    ["TH", "+66"],
    ["JP", "+81"],
    ["GB", "+44"],
    ["US", "+1"],
  ])("maps %s to its canonical calling code", (countryCode, callingCode) => {
    expect(callingCodeForCountry(countryCode)).toBe(callingCode);
  });

  it("keeps ISO keys and canonical calling codes separate", () => {
    expect(callingCodeOptions.some((option) => option.countryCode === "TH" && option.value === "+66")).toBe(true);
    expect(callingCodeOptions.every((option) => /^\+[1-9]\d{0,2}$/.test(option.value))).toBe(true);
  });
});
