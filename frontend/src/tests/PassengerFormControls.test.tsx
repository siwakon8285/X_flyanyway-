import { fireEvent, screen } from "@testing-library/react";
import { useState } from "react";

import { PassengerForm } from "@/components/booking/passengers/PassengerForm";
import type { PassengerFormValue } from "@/components/booking/passengers/passengerTypes";
import { render } from "@/tests/renderWithLanguage";

const passenger = (overrides: Partial<PassengerFormValue> = {}): PassengerFormValue => ({
  dateOfBirth: "1990-01-01",
  email: "nara@example.com",
  emergencyContact: null,
  familyName: "Suri",
  gender: "FEMALE",
  givenName: "Nara",
  middleName: "",
  nationalityCode: "",
  ordinal: 1,
  passengerType: "ADULT",
  passportIssuingCountryCode: "TH",
  passportNumber: "TH123456",
  phoneCountryCode: "",
  phoneNumber: "812345678",
  title: "MS",
  ...overrides,
});

const renderForm = (
  value = passenger(),
  options?: { locale?: "en" | "th" },
) => {
  const onValuesChange = jest.fn();
  render(
    <PassengerForm
      errors={[]}
      onSave={jest.fn()}
      onValuesChange={onValuesChange}
      ready={false}
      recentlySaved={false}
      saving={false}
      values={[value]}
    />,
    options,
  );
  return onValuesChange;
};

describe("Passenger form controls", () => {
  afterEach(() => {
    Reflect.deleteProperty(HTMLInputElement.prototype, "showPicker");
  });

  it("opens the native picker when any Passenger date field is clicked", () => {
    const showPicker = jest.fn();
    Object.defineProperty(HTMLInputElement.prototype, "showPicker", {
      configurable: true,
      value: showPicker,
    });
    renderForm();

    fireEvent.click(screen.getByLabelText("Date of birth"));

    expect(showPicker).toHaveBeenCalledTimes(1);
  });

  it("offers only Mr and Ms titles and omits passport date inputs", () => {
    renderForm();

    expect(screen.getByRole("option", { name: "Mr" })).toBeInTheDocument();
    expect(screen.getByRole("option", { name: "Ms" })).toBeInTheDocument();
    expect(screen.queryByRole("option", { name: "Mrs" })).not.toBeInTheDocument();
    expect(screen.queryByLabelText("Passport issue date")).not.toBeInTheDocument();
    expect(screen.queryByLabelText("Passport expiry date")).not.toBeInTheDocument();
  });

  it("keeps date fields focusable when showPicker is unavailable", () => {
    renderForm();
    const dateOfBirth = screen.getByLabelText("Date of birth");

    expect(() => fireEvent.click(dateOfBirth)).not.toThrow();
    expect(dateOfBirth).toHaveFocus();
  });

  it("uses destructive styling only for required indicators", () => {
    renderForm();

    expect(screen.getAllByText("Required")[0]).toHaveClass("text-destructive");
    expect(screen.getAllByText("Optional")[0]).toHaveClass("text-muted-foreground");
  });

  it("uses bounded searchable listboxes for country and calling-code fields", () => {
    renderForm();

    fireEvent.click(screen.getByRole("button", { name: "Nationality" }));
    expect(screen.getByRole("listbox", { name: "Nationality" })).toHaveClass("overflow-y-auto");
    expect(screen.getByPlaceholderText("Search countries")).toBeInTheDocument();

    fireEvent.click(screen.getByRole("button", { name: "Phone country code" }));
    expect(screen.getByRole("listbox", { name: "Phone country code" })).toHaveClass("overflow-y-auto");
  });

  it("keeps only the newly opened country selector active", () => {
    renderForm();

    fireEvent.click(screen.getByRole("button", { name: "Nationality" }));
    expect(screen.getByRole("listbox", { name: "Nationality" })).toBeInTheDocument();

    fireEvent.click(screen.getByRole("button", { name: "Passport issuing country" }));

    expect(screen.queryByRole("listbox", { name: "Nationality" })).not.toBeInTheDocument();
    expect(screen.getByRole("listbox", { name: "Passport issuing country" })).toBeInTheDocument();

    fireEvent.click(screen.getByRole("button", { name: "Phone country code" }));

    expect(screen.queryByRole("listbox", { name: "Passport issuing country" })).not.toBeInTheDocument();
    expect(screen.getByRole("listbox", { name: "Phone country code" })).toBeInTheDocument();
  });

  it("closes an active selector after selection, Escape, and an outside click", () => {
    renderForm();
    const trigger = screen.getByRole("button", { name: "Nationality" });

    fireEvent.click(trigger);
    fireEvent.click(screen.getByRole("option", { name: /Thailand/ }));
    expect(screen.queryByRole("listbox", { name: "Nationality" })).not.toBeInTheDocument();

    fireEvent.click(trigger);
    fireEvent.keyDown(screen.getByRole("combobox", { name: "Search countries" }), { key: "Escape" });
    expect(screen.queryByRole("listbox", { name: "Nationality" })).not.toBeInTheDocument();

    fireEvent.click(trigger);
    fireEvent.pointerDown(document.body);
    expect(screen.queryByRole("listbox", { name: "Nationality" })).not.toBeInTheDocument();
  });

  it("keeps the search control outside a fixed-height options viewport", () => {
    renderForm();

    fireEvent.click(screen.getByRole("button", { name: "Nationality" }));

    const search = screen.getByRole("combobox", { name: "Search countries" });
    const listbox = screen.getByRole("listbox", { name: "Nationality" });
    const panel = listbox.parentElement;

    expect(panel).toHaveClass("h-[min(26rem,60dvh)]");
    expect(panel).toContainElement(search);
    expect(listbox).toHaveClass("min-h-0", "flex-1", "overflow-y-auto");
    expect(listbox).toHaveAttribute("data-lenis-prevent-wheel");
    expect(search.closest('[role="listbox"]')).toBeNull();
  });

  it("defaults phone code from nationality until the passenger chooses another code", () => {
    const onValuesChange = renderForm();

    fireEvent.click(screen.getByRole("button", { name: "Nationality" }));
    fireEvent.click(screen.getByRole("option", { name: /Thailand/ }));
    expect(onValuesChange).toHaveBeenLastCalledWith([
      expect.objectContaining({ nationalityCode: "TH", phoneCountryCode: "+66" }),
    ]);

  });

  it("preserves a saved phone code when nationality changes", () => {
    const FormHarness = () => {
      const [values, setValues] = useState([passenger({ nationalityCode: "TH", phoneCountryCode: "+1" })]);
      return <PassengerForm errors={[]} onSave={jest.fn()} onValuesChange={setValues} ready={false} recentlySaved={false} saving={false} values={values} />;
    };
    render(<FormHarness />);

    fireEvent.click(screen.getByRole("button", { name: "Nationality" }));
    fireEvent.click(screen.getByRole("option", { name: /Japan/ }));

    expect(screen.getByRole("button", { name: "Phone country code" })).toHaveTextContent("United States +1");
  });

  it("renders localized country labels while preserving calling-code values", () => {
    renderForm(passenger({ nationalityCode: "TH", phoneCountryCode: "+66" }), { locale: "th" });

    fireEvent.click(screen.getByRole("button", { name: "รหัสประเทศของโทรศัพท์" }));
    expect(screen.getByRole("option", { name: /ไทย.*\+66/ })).toBeInTheDocument();
  });

  it("uses the same calling-code selector for emergency contacts", () => {
    renderForm(passenger({
      emergencyContact: {
        name: "Sam Lee",
        phoneCountryCode: "+66",
        phoneNumber: "812345678",
        relationship: "Parent",
      },
    }));

    fireEvent.click(screen.getByRole("button", { name: "Phone country code emergency contact" }));
    expect(screen.getByRole("listbox", { name: "Phone country code emergency contact" })).toBeInTheDocument();
  });
});
