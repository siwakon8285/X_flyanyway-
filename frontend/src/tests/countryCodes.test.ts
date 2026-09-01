import countryCodes from "@/data/country-codes.json";

describe("frontend country-code data", () => {
  it("resolves inside the Next.js project and contains unique ISO alpha-2 values", () => {
    expect(countryCodes).toContain("TH");
    expect(countryCodes).toContain("GB");
    expect(new Set(countryCodes).size).toBe(countryCodes.length);
    expect(countryCodes.every((code) => /^[A-Z]{2}$/.test(code))).toBe(true);
  });
});
