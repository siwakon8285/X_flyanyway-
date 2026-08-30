import type { Locale } from "@/i18n/config";

const dateLocales: Record<Locale, string> = {
  en: "en-GB-u-ca-gregory-nu-latn",
  th: "th-TH-u-ca-gregory-nu-latn",
};

const numberLocales: Record<Locale, string> = {
  en: "en-US-u-nu-latn",
  th: "th-TH-u-nu-latn",
};

const formatDate = (value: string, locale: Locale) => {
  const [year, month, day] = value.split("-").map(Number);
  return new Intl.DateTimeFormat(dateLocales[locale], {
    day: "2-digit",
    month: "short",
    timeZone: "UTC",
    year: "numeric",
  }).format(new Date(Date.UTC(year, month - 1, day)));
};

const formatPrice = (amount: number, locale: Locale) =>
  `THB ${new Intl.NumberFormat(numberLocales[locale], {
    maximumFractionDigits: 0,
  }).format(amount)}`;

const formatDuration = (durationMinutes: number, locale: Locale) => {
  const hours = Math.floor(durationMinutes / 60);
  const minutes = durationMinutes % 60;

  if (locale === "th") {
    return minutes === 0 ? `${hours} ชม.` : `${hours} ชม. ${minutes} นาที`;
  }

  return minutes === 0 ? `${hours}h` : `${hours}h ${minutes}m`;
};

export { formatDate, formatDuration, formatPrice };
