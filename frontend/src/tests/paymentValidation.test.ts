import { validateDemoCard } from "@/components/booking/payment/paymentValidation";

describe("demo Card validation", () => {
  it("accepts the documented transient demo values", () => {
    expect(
      validateDemoCard({
        cardholderName: "Demo Traveller",
        cardNumber: "4242 4242 4242 4242",
        cvc: "123",
        expiry: "12/30",
      }),
    ).toEqual({});
  });

  it("returns field-specific errors for incomplete or malformed values", () => {
    expect(
      validateDemoCard({ cardholderName: "", cardNumber: "123", cvc: "x", expiry: "01/20" }),
    ).toEqual({
      cardholderName: "required",
      cardNumber: "invalid",
      cvc: "invalid",
      expiry: "expired",
    });
  });
});
