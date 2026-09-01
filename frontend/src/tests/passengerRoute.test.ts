import {
  buildExtrasHandoffHref,
  buildPassengerInformationHref,
} from "@/components/booking/passengers/passengerRoute";

describe("passenger route contracts", () => {
  it("preserves non-sensitive search context but derives passenger authority from holdId", () => {
    expect(
      buildPassengerInformationHref({
        flightId: "xf-201",
        holdId: "hold-123",
        query:
          "from=BKK&to=LHR&departure=2030-05-10&adults=2&children=1&infants=1&cabin=business&trip=one-way",
        selectedCabin: "business",
      }),
    ).toBe(
      "/booking/passengers?from=BKK&to=LHR&departure=2030-05-10&adults=2&children=1&infants=1&cabin=business&trip=one-way&flightId=xf-201&selectedCabin=business&holdId=hold-123",
    );
  });

  it("prepares the future Extras handoff without adding passenger data", () => {
    expect(buildExtrasHandoffHref("hold-123")).toBe("/booking/extras?holdId=hold-123");
  });
});
