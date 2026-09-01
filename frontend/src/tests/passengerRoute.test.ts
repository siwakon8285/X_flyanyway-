import {
  buildExtrasHandoffHref,
  buildPassengerInformationHref,
  buildReviewHandoffHref,
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

  it("hands off to Extras with only the public hold and allowlisted recovery context", () => {
    expect(
      buildExtrasHandoffHref({
        holdId: "hold-123",
        query:
          "from=BKK&to=LHR&departure=2030-05-10&flightId=xf-201&selectedCabin=business&seats=20A%2C20B&email=private%40example.com",
      }),
    ).toBe(
      "/booking/extras?from=BKK&to=LHR&departure=2030-05-10&flightId=xf-201&selectedCabin=business&holdId=hold-123",
    );
  });

  it("prepares but does not navigate to the future Review route", () => {
    expect(buildReviewHandoffHref("hold-123")).toBe(
      "/booking/review?holdId=hold-123",
    );
  });
});
