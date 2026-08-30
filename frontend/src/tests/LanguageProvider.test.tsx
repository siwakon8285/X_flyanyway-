import { fireEvent, render, screen } from "@testing-library/react";

import { LanguageToggle } from "@/components/layout/LanguageToggle";
import {
  LanguageProvider,
  useLanguage,
} from "@/i18n/LanguageProvider";

const SampleCopy = () => {
  const { t } = useLanguage();
  return <p>{t("navigation.bookFlight")}</p>;
};

describe("LanguageProvider", () => {
  beforeEach(() => {
    document.cookie = "app-locale=; Max-Age=0; Path=/";
    document.documentElement.lang = "en";
    window.history.replaceState({}, "", "/flights?from=BKK&to=LHR#results");
  });

  it("renders English by default and switches to persisted Thai without navigation", () => {
    const initialUrl = window.location.href;
    const initialScrollY = window.scrollY;

    render(
      <LanguageProvider initialLocale="en">
        <LanguageToggle />
        <SampleCopy />
      </LanguageProvider>,
    );

    const toggle = screen.getByRole("button", {
      name: "Current language: English. Switch to Thai.",
    });
    expect(toggle).toHaveAttribute("type", "button");
    expect(toggle).toHaveTextContent("EN");

    toggle.focus();
    fireEvent.click(toggle);

    expect(toggle).toHaveFocus();
    expect(toggle).toHaveTextContent("TH");
    expect(toggle).toHaveAccessibleName(
      "ภาษาปัจจุบัน: ไทย เปลี่ยนเป็นภาษาอังกฤษ",
    );
    expect(screen.getByText("จองเที่ยวบิน")).toBeInTheDocument();
    expect(document.documentElement).toHaveAttribute("lang", "th");
    expect(document.cookie).toContain("app-locale=th");
    expect(window.location.href).toBe(initialUrl);
    expect(window.scrollY).toBe(initialScrollY);

    fireEvent.click(toggle);
    expect(toggle).toHaveTextContent("EN");
    expect(screen.getByText("Book a Flight")).toBeInTheDocument();
    expect(document.documentElement).toHaveAttribute("lang", "en");
    expect(document.cookie).toContain("app-locale=en");
  });

  it("uses persisted Thai as the initial visible render", () => {
    render(
      <LanguageProvider initialLocale="th">
        <LanguageToggle />
        <SampleCopy />
      </LanguageProvider>,
    );

    expect(screen.getByText("จองเที่ยวบิน")).toBeInTheDocument();
    expect(screen.queryByText("Book a Flight")).not.toBeInTheDocument();
    expect(screen.getByRole("button")).toHaveTextContent("TH");
  });
});
