const SUPPORTED_LOCALES = ["en", "th"] as const;

type Locale = (typeof SUPPORTED_LOCALES)[number];

const DEFAULT_LOCALE: Locale = "en";
const LOCALE_COOKIE_NAME = "app-locale";
const LOCALE_COOKIE_MAX_AGE = 60 * 60 * 24 * 365;

const isLocale = (value: unknown): value is Locale =>
  typeof value === "string" &&
  SUPPORTED_LOCALES.some((locale) => locale === value);

const getLocaleFromCookieValue = (value: string | undefined): Locale =>
  isLocale(value) ? value : DEFAULT_LOCALE;

const serializeLocaleCookie = (locale: Locale) => {
  const secure =
    typeof window !== "undefined" && window.location.protocol === "https:"
      ? "; Secure"
      : "";

  return `${LOCALE_COOKIE_NAME}=${locale}; Path=/; Max-Age=${LOCALE_COOKIE_MAX_AGE}; SameSite=Lax${secure}`;
};

export {
  DEFAULT_LOCALE,
  getLocaleFromCookieValue,
  isLocale,
  LOCALE_COOKIE_MAX_AGE,
  LOCALE_COOKIE_NAME,
  serializeLocaleCookie,
  SUPPORTED_LOCALES,
};
export type { Locale };
