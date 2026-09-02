import {
  buildPaymentHandoffHref,
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
    expect(
      buildReviewHandoffHref({
        holdId: "hold-123",
        query: "flightId=xf-201&departure=2030-05-10&email=private%40example.com&grandTotal=1",
      }),
    ).toBe(
      "/booking/review?flightId=xf-201&departure=2030-05-10&holdId=hold-123",
    );
  });

  it("prepares a future Payment URL without carrying sensitive or monetary state", () => {
    expect(
      buildPaymentHandoffHref({
        holdId: "hold-123",
        query: "flightId=xf-201&passport=private&grandTotal=1",
      }),
    ).toBe("/booking/payment?flightId=xf-201&holdId=hold-123");
  });
});
