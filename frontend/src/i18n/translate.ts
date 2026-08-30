import type { Locale } from "@/i18n/config";
import { en } from "@/i18n/locales/en";
import { th } from "@/i18n/locales/th";
import type {
  TranslationDictionary,
  TranslationKey,
  TranslationVariables,
} from "@/i18n/types";

const dictionaries = { en, th } satisfies Record<Locale, TranslationDictionary>;

const collectDictionaryLeafKeys = (
  dictionary: Readonly<Record<string, unknown>>,
  prefix: string,
): string[] =>
  Object.entries(dictionary).flatMap(([key, value]) => {
    const path = prefix ? `${prefix}.${key}` : key;
    return typeof value === "string"
      ? [path]
      : collectDictionaryLeafKeys(value as Readonly<Record<string, unknown>>, path);
  });

const getDictionaryLeafKeys = (dictionary: TranslationDictionary): string[] =>
  collectDictionaryLeafKeys(dictionary, "");

const getValue = (dictionary: TranslationDictionary, key: string) => {
  let current: unknown = dictionary;

  for (const segment of key.split(".")) {
    if (typeof current !== "object" || current === null || !(segment in current)) {
      return undefined;
    }
    current = (current as Record<string, unknown>)[segment];
  }

  return typeof current === "string" ? current : undefined;
};

const interpolate = (template: string, variables: TranslationVariables = {}) =>
  template.replace(/\{([^}]+)\}/g, (placeholder, name: string) => {
    const value = variables[name];
    if (value === undefined) {
      throw new Error(`Missing translation variable: ${name}`);
    }
    return String(value);
  });

const translate = (
  locale: Locale,
  key: TranslationKey,
  variables?: TranslationVariables,
) => {
  const localized = getValue(dictionaries[locale], key);
  if (localized !== undefined) return interpolate(localized, variables);

  const fallback = getValue(en, key);
  if (fallback !== undefined && process.env.NODE_ENV === "production") {
    return interpolate(fallback, variables);
  }

  throw new Error(`Missing translation key: ${key}`);
};

export { dictionaries, getDictionaryLeafKeys, translate };
