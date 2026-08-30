import { DEFAULT_LOCALE, getLocaleFromCookieValue } from "@/i18n/config";
import { formatDate, formatDuration, formatPrice } from "@/i18n/formatters";
import { en } from "@/i18n/locales/en";
import { th } from "@/i18n/locales/th";
import { getDictionaryLeafKeys, translate } from "@/i18n/translate";

describe("i18n foundation", () => {
  it("defaults missing and unsupported preferences to English", () => {
    expect(DEFAULT_LOCALE).toBe("en");
    expect(getLocaleFromCookieValue(undefined)).toBe("en");
    expect(getLocaleFromCookieValue("fr")).toBe("en");
    expect(getLocaleFromCookieValue("th")).toBe("th");
  });

  it("keeps English and Thai dictionaries structurally identical", () => {
    expect(getDictionaryLeafKeys(th)).toEqual(getDictionaryLeafKeys(en));
  });

  it("translates semantic keys and interpolates dynamic values", () => {
    expect(translate("en", "navigation.bookFlight")).toBe("Book a Flight");
    expect(translate("th", "navigation.bookFlight")).toBe("จองเที่ยวบิน");
    expect(translate("th", "flightResults.flightCount", { count: 3 })).toBe(
      "มีเที่ยวบิน 3 เที่ยว",
    );
  });

  it("throws deterministically for an unknown translation key", () => {
    expect(() =>
      translate("th", "navigation.missing" as never),
    ).toThrow("Missing translation key: navigation.missing");
  });

  it("formats Thai dates with Gregorian years and preserves domain currency", () => {
    expect(formatDate("2099-05-10", "en")).toContain("2099");
    expect(formatDate("2099-05-10", "th")).toContain("2099");
    expect(formatDate("2099-05-10", "th")).not.toContain("2642");
    expect(formatPrice(42000, "th")).toBe("THB 42,000");
    expect(formatDuration(150, "th")).toBe("2 ชม. 30 นาที");
  });
});
