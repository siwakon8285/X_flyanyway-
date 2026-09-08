import { fireEvent, screen } from "@testing-library/react";
import { useState } from "react";

import { AirportCombobox } from "@/components/admin/flights/AirportCombobox";
import { DatePickerField } from "@/components/admin/flights/DatePickerField";
import { TimePickerField } from "@/components/admin/flights/TimePickerField";
import type { AirportReference } from "@/lib/admin/flightTypes";
import { render } from "@/tests/renderWithLanguage";

const airports: AirportReference[] = [
  { code:"BKK", name:"Suvarnabhumi Airport", city:"Bangkok", countryCode:"TH", countryName:"Thailand", timeZone:"Asia/Bangkok" },
  { code:"DXB", name:"Dubai International Airport", city:"Dubai", countryCode:"AE", countryName:"United Arab Emirates", timeZone:"Asia/Dubai" },
  { code:"SYD", name:"Sydney Kingsford Smith International Airport", city:"Sydney (Mascot)", countryCode:"AU", countryName:"Australia", timeZone:"Australia/Sydney" },
  { code:"CDG", name:"Charles de Gaulle International Airport", city:"Paris", countryCode:"FR", countryName:"France", timeZone:"Europe/Paris" },
];

const RouteFields = () => {
  const [origin, setOrigin] = useState("SYD");
  const [destination, setDestination] = useState("");
  return (
    <>
      <AirportCombobox airports={airports} label="Origin" onChange={setOrigin} value={origin} />
      <AirportCombobox airports={airports} label="Destination" onChange={setDestination} value={destination} />
    </>
  );
};

describe("Flight Management date picker", () => {
  it("opens the native picker from the whole visible control and shows a calendar icon", () => {
    const onChange = jest.fn();
    const { container } = render(
      <DatePickerField label="Departure date" onChange={onChange} required value="" />,
    );
    const input = screen.getByLabelText("Departure date") as HTMLInputElement;
    const showPicker = jest.fn();
    Object.defineProperty(input, "showPicker", { configurable:true, value:showPicker });

    expect(container.querySelector("[data-calendar-icon]")).toBeInTheDocument();
    expect(input).toHaveClass("xfo-control", "xfo-date-input");
    expect(input.className).not.toContain("pl-3");
    expect(container.querySelector("[data-calendar-icon]")).toHaveClass("pointer-events-none", "left-3");
    fireEvent.click(container.querySelector("[data-date-picker-control]") as HTMLElement);
    expect(input).toHaveFocus();
    expect(showPicker).toHaveBeenCalledTimes(1);
  });

  it("retains native date input, value changes, and unmodified Tab behavior", () => {
    const onChange = jest.fn();
    render(<DatePickerField label="Departure date" onChange={onChange} value="2026-10-08" />);
    const input = screen.getByLabelText("Departure date");
    expect(input).toHaveAttribute("type", "date");
    fireEvent.change(input, { target:{ value:"2026-10-09" } });
    expect(onChange).toHaveBeenCalledWith("2026-10-09");
    expect(fireEvent.keyDown(input, { key:"Tab" })).toBe(true);
  });

  it("falls back to focusing the native control when showPicker is unavailable", () => {
    const { container } = render(
      <DatePickerField label="Departure date" onChange={jest.fn()} value="" />,
    );
    const input = screen.getByLabelText("Departure date") as HTMLInputElement;
    Object.defineProperty(input, "showPicker", { configurable:true, value:undefined });

    expect(() => fireEvent.click(container.querySelector("[data-date-picker-control]") as HTMLElement)).not.toThrow();
    expect(input).toHaveFocus();
  });

  it("keeps the native date input focused when showPicker rejects the request", () => {
    const { container } = render(
      <DatePickerField label="Departure date" onChange={jest.fn()} value="" />,
    );
    const input = screen.getByLabelText("Departure date") as HTMLInputElement;
    const showPicker = jest.fn(() => { throw new DOMException("Picker unavailable", "NotAllowedError"); });
    Object.defineProperty(input, "showPicker", { configurable:true, value:showPicker });

    expect(() => fireEvent.click(container.querySelector("[data-date-picker-control]") as HTMLElement)).not.toThrow();
    expect(showPicker).toHaveBeenCalledTimes(1);
    expect(input).toHaveFocus();
  });
});

describe("Flight Management time picker", () => {
  it.each(["Departure", "Arrival"])(
    "keeps %s as a real required time input and opens from the visible control",
    (label) => {
      const { container } = render(
        <TimePickerField label={label} onChange={jest.fn()} required value="" />,
      );
      const input = screen.getByLabelText(label) as HTMLInputElement;
      const showPicker = jest.fn();
      Object.defineProperty(input, "showPicker", { configurable:true, value:showPicker });

      expect(input).toHaveAttribute("type", "time");
      expect(input).toBeRequired();
      expect(input).toHaveClass("xfo-control", "xfo-time-input");
      expect(container.querySelector("[data-clock-icon]")).toHaveClass("pointer-events-none", "left-3");
      fireEvent.click(container.querySelector("[data-time-picker-control]") as HTMLElement);
      expect(input).toHaveFocus();
      expect(showPicker).toHaveBeenCalledTimes(1);
    },
  );

  it("supports value entry and leaves keyboard navigation untouched", () => {
    const onChange = jest.fn();
    render(<TimePickerField label="Departure" onChange={onChange} value="" />);
    const input = screen.getByLabelText("Departure");
    fireEvent.change(input, { target:{ value:"09:20" } });
    expect(onChange).toHaveBeenCalledWith("09:20");
    expect(fireEvent.keyDown(input, { key:"Tab" })).toBe(true);
  });

  it("falls back to focusing the native time input when showPicker is unavailable", () => {
    const { container } = render(
      <TimePickerField label="Arrival" onChange={jest.fn()} value="" />,
    );
    const input = screen.getByLabelText("Arrival") as HTMLInputElement;
    Object.defineProperty(input, "showPicker", { configurable:true, value:undefined });

    expect(() => fireEvent.click(container.querySelector("[data-time-picker-control]") as HTMLElement)).not.toThrow();
    expect(input).toHaveFocus();
  });
});

describe("Flight Management airport combobox", () => {
  it("opens Origin and Destination as bounded internally scrollable listboxes", () => {
    render(<RouteFields />);
    const origin = screen.getByRole("combobox", { name:"Origin" });
    const destination = screen.getByRole("combobox", { name:"Destination" });

    fireEvent.click(origin);
    expect(origin).toHaveAttribute("aria-expanded", "true");
    expect(origin).toHaveClass("xfo-control", "xfo-airport-input");
    const originList = screen.getByRole("listbox");
    expect(originList).toHaveClass("overflow-y-auto", "overscroll-contain");
    expect(originList.className).toContain("max-h-[min(18rem,45dvh)]");

    fireEvent.click(destination);
    expect(destination).toHaveAttribute("aria-expanded", "true");
  });

  it.each([
    ["IATA", "syd", "SYD"],
    ["airport name", "charles de gaulle", "CDG"],
    ["city", "bangkok", "BKK"],
    ["country", "united arab emirates", "DXB"],
  ])("searches case-insensitively by %s", (_kind, query, code) => {
    render(<RouteFields />);
    const destination = screen.getByRole("combobox", { name:"Destination" });
    fireEvent.change(destination, { target:{ value:query } });
    expect(screen.getByRole("option", { name:new RegExp(code) })).toBeInTheDocument();
    expect(screen.getAllByRole("option")).toHaveLength(1);
  });

  it("supports mouse selection and renders the compact selected state", () => {
    render(<RouteFields />);
    const destination = screen.getByRole("combobox", { name:"Destination" });
    fireEvent.change(destination, { target:{ value:"France" } });
    fireEvent.click(screen.getByRole("option", { name:/CDG.*Paris.*France/ }));
    expect(destination).toHaveValue("CDG · Paris · France");
    expect(destination).toHaveAttribute("aria-expanded", "false");
  });

  it("supports Arrow keys, Enter selection, Escape, and leaves Tab unmodified", () => {
    render(<RouteFields />);
    const destination = screen.getByRole("combobox", { name:"Destination" });
    fireEvent.focus(destination);
    fireEvent.keyDown(destination, { key:"ArrowDown" });
    fireEvent.keyDown(destination, { key:"Enter" });
    expect(destination).toHaveValue("DXB · Dubai · United Arab Emirates");

    fireEvent.focus(destination);
    fireEvent.keyDown(destination, { key:"Escape" });
    expect(destination).toHaveAttribute("aria-expanded", "false");
    expect(fireEvent.keyDown(destination, { key:"Tab" })).toBe(true);
  });

  it("announces an empty localized result", () => {
    render(
      <AirportCombobox
        airports={airports}
        label="ปลายทาง"
        onChange={jest.fn()}
        value=""
      />,
      { locale:"th" },
    );
    const destination = screen.getByRole("combobox", { name:"ปลายทาง" });
    fireEvent.change(destination, { target:{ value:"moon" } });
    expect(screen.getByRole("status")).toHaveTextContent("ไม่พบสนามบิน");
  });
});
