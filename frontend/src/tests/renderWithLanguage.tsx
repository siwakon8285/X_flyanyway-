import {
  render as renderTestingLibrary,
  type RenderOptions,
} from "@testing-library/react";
import type { ReactNode, ReactElement } from "react";

import { LanguageProvider } from "@/i18n/LanguageProvider";
import type { Locale } from "@/i18n/config";

const render = (
  ui: ReactElement,
  options?: RenderOptions & { locale?: Locale },
) => {
  const { locale = "en", ...renderOptions } = options ?? {};

  return renderTestingLibrary(
    <LanguageProvider initialLocale={locale}>{ui as ReactNode}</LanguageProvider>,
    renderOptions,
  );
};

export { render };
