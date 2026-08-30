import type { en } from "@/i18n/locales/en";

type StringShape<T> = {
  readonly [Key in keyof T]: T[Key] extends string ? string : StringShape<T[Key]>;
};

type Join<Prefix extends string, Key extends string> = Prefix extends ""
  ? Key
  : `${Prefix}.${Key}`;

type NestedKeyOf<T, Prefix extends string = ""> = {
  [Key in keyof T & string]: T[Key] extends string
    ? Join<Prefix, Key>
    : NestedKeyOf<T[Key], Join<Prefix, Key>>;
}[keyof T & string];

type TranslationDictionary = StringShape<typeof en>;
type TranslationKey = NestedKeyOf<TranslationDictionary>;
type TranslationVariables = Readonly<Record<string, number | string>>;

export type {
  TranslationDictionary,
  TranslationKey,
  TranslationVariables,
};
