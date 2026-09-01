import { fireEvent, screen, waitFor, within } from "@testing-library/react";

import { PassengerInformationPage } from "@/components/booking/passengers/PassengerInformationPage";
import type {
  Passenger,
  PassengerContext,
} from "@/components/booking/passengers/passengerTypes";
import { render } from "@/tests/renderWithLanguage";

const mockPush = jest.fn();

jest.mock("next/navigation", () => ({
  useRouter: () => ({ push: mockPush }),
}));

const hold = {
  cabin: "economy" as const,
  departureDate: "2027-05-10",
  expiresAt: "2099-05-01T10:10:00Z",
  flightId: "xf-201",
  id: "hold-123",
  passengers: { adults: 1, children: 1, infants: 1 },
  seats: ["20A", "20B"],
  serverTime: "2099-05-01T10:00:00Z",
};

const savedPassenger = (
  ordinal: number,
  passengerType: Passenger["passengerType"],
): Passenger => ({
  dateOfBirth:
    passengerType === "ADULT"
      ? "1990-01-01"
      : passengerType === "CHILD"
        ? "2022-01-01"
        : "2026-01-01",
  email: `traveller${ordinal}@example.com`,
  emergencyContact: null,
  familyName: "Suri",
  gender: "FEMALE",
  givenName: "Nara",
  middleName: null,
  nationalityCode: "TH",
  ordinal,
  passengerType,
  passportIssuingCountryCode: "TH",
  passportNumber: `TH${ordinal}234567`,
  phoneCountryCode: "+66",
  phoneNumber: `81234567${ordinal}`,
  title: "MS",
});

const emptyContext: PassengerContext = {
  expectedPassengers: [
    { ordinal: 1, passengerType: "ADULT" },
    { ordinal: 2, passengerType: "CHILD" },
    { ordinal: 3, passengerType: "INFANT" },
  ],
  hold,
  passengers: [],
  readyToContinue: false,
};

const savedContext: PassengerContext = {
  ...emptyContext,
  passengers: [
    savedPassenger(1, "ADULT"),
    savedPassenger(2, "CHILD"),
    savedPassenger(3, "INFANT"),
  ],
  readyToContinue: true,
};

const setFetch = (...responses: Array<{ body: unknown; ok: boolean; status: number }>) => {
  const fetchMock = jest.fn();
  responses.forEach(({ body, ok, status }) => {
    fetchMock.mockResolvedValueOnce({ json: async () => body, ok, status });
  });
  Object.defineProperty(global, "fetch", {
    configurable: true,
    value: fetchMock,
    writable: true,
  });
  return fetchMock;
};

describe("PassengerInformationPage", () => {
  const originalFetch = global.fetch;

  afterEach(() => {
    mockPush.mockReset();
    if (originalFetch) {
      Object.defineProperty(global, "fetch", {
        configurable: true,
        value: originalFetch,
        writable: true,
      });
    } else {
      Reflect.deleteProperty(global, "fetch");
    }
  });

  it("renders deterministic adult, child, and infant sections from the hold response", async () => {
    setFetch({ body: emptyContext, ok: true, status: 200 });
    render(
      <PassengerInformationPage
        backQuery="from=BKK&to=LHR&departure=2027-05-10&adults=99&children=0&infants=0&cabin=economy&trip=one-way&flightId=xf-201"
        holdId="hold-123"
      />,
    );

    expect(await screen.findByRole("heading", { name: "Passenger information" })).toBeInTheDocument();
    const navigation = await screen.findByRole("navigation", { name: "Passengers" });
    expect(within(navigation).getByRole("button", { name: "Passenger 1 — Adult" })).toBeInTheDocument();
    expect(within(navigation).getByRole("button", { name: "Passenger 2 — Child" })).toBeInTheDocument();
    expect(within(navigation).getByRole("button", { name: "Passenger 3 — Infant" })).toBeInTheDocument();
    expect(screen.getByRole("group", { name: "Passenger 1 — Adult" })).toBeInTheDocument();
    expect(screen.getByText("Name must match passport exactly.")).toBeInTheDocument();
    expect(screen.getByText("20A")).toBeInTheDocument();
    expect(screen.getByText("20B")).toBeInTheDocument();
    expect(screen.getByText("Infant · No seat assigned")).toBeInTheDocument();
    expect(screen.queryByText("99 passengers")).not.toBeInTheDocument();
  });

  it("navigates to the real Extras route only after Passenger save succeeds", async () => {
    const fetchMock = setFetch(
      { body: savedContext, ok: true, status: 200 },
      { body: savedContext, ok: true, status: 200 },
    );
    render(
      <PassengerInformationPage
        backQuery="from=BKK&to=LHR&departure=2027-05-10&adults=1&children=1&infants=1&cabin=economy&trip=one-way&flightId=xf-201"
        holdId="hold-123"
      />,
    );

    expect(await screen.findByLabelText("Given name")).toHaveValue("Nara");
    fireEvent.click(screen.getByRole("button", { name: "Save passenger information" }));

    await waitFor(() => expect(fetchMock).toHaveBeenCalledTimes(2));
    expect(mockPush).toHaveBeenCalledWith(
      "/booking/extras?from=BKK&to=LHR&departure=2027-05-10&adults=1&children=1&infants=1&cabin=economy&trip=one-way&flightId=xf-201&holdId=hold-123",
    );
  });

  it("maps backend field errors without exposing backend messages", async () => {
    setFetch(
      { body: savedContext, ok: true, status: 200 },
      {
        body: {
          error: {
            code: "PASSENGER_VALIDATION_FAILED",
            message: "Passenger information is invalid.",
            fieldErrors: [{ passenger: 1, field: "email", code: "INVALID_EMAIL" }],
          },
        },
        ok: false,
        status: 422,
      },
    );
    render(
      <PassengerInformationPage backQuery="flightId=xf-201" holdId="hold-123" />,
    );
    await screen.findByLabelText("Given name");
    fireEvent.click(screen.getByRole("button", { name: "Save passenger information" }));

    expect(await screen.findByText("Enter a valid email address.")).toBeInTheDocument();
    expect(screen.getByLabelText("Email")).toHaveAttribute("aria-invalid", "true");
    expect(mockPush).not.toHaveBeenCalled();
  });

  it("blocks saving when the server reports an expired hold and guides back to seats", async () => {
    setFetch({
      body: { error: { code: "HOLD_EXPIRED", message: "The seat hold has expired." } },
      ok: false,
      status: 410,
    });
    render(
      <PassengerInformationPage
        backQuery="from=BKK&to=LHR&departure=2027-05-10&adults=1&children=1&infants=1&cabin=economy&trip=one-way&flightId=xf-201"
        holdId="hold-123"
      />,
    );

    expect(await screen.findByRole("alert")).toHaveTextContent(
      "Your seat hold expired. Return to seat selection to choose seats again.",
    );
    expect(screen.queryByRole("button", { name: "Save passenger information" })).not.toBeInTheDocument();
    expect(screen.getByRole("link", { name: "Return to seat selection" })).toHaveAttribute(
      "href",
      expect.stringContaining("/flights/xf-201/seats?"),
    );
  });

  it("renders the Passenger workflow in Thai while preserving passport and seat values", async () => {
    setFetch({ body: savedContext, ok: true, status: 200 });
    render(
      <PassengerInformationPage backQuery="flightId=xf-201" holdId="hold-123" />,
      { locale: "th" },
    );

    expect(await screen.findByText("ชื่อต้องตรงกับหนังสือเดินทางทุกตัวอักษร")).toBeInTheDocument();
    expect(screen.getByDisplayValue("TH1234567")).toBeInTheDocument();
    expect(screen.getByText("20A")).toBeInTheDocument();
  });

  it("connects required errors to controls and focuses the first invalid field", async () => {
    setFetch({ body: emptyContext, ok: true, status: 200 });
    render(
      <PassengerInformationPage backQuery="flightId=xf-201" holdId="hold-123" />,
    );
    await screen.findByRole("form", { name: "Passenger information form" });
    fireEvent.click(screen.getByRole("button", { name: "Save passenger information" }));

    await waitFor(() => expect(screen.getByLabelText("Given name")).toHaveFocus());
    expect(screen.getByLabelText("Given name")).toHaveAttribute("aria-invalid", "true");
    expect(screen.getAllByText("This field is required.").length).toBeGreaterThan(0);
  });
});
