type DemoCardFields = {
  cardholderName: string;
  cardNumber: string;
  cvc: string;
  expiry: string;
};

type DemoCardErrors = Partial<Record<keyof DemoCardFields, "expired" | "invalid" | "required">>;

const validateDemoCard = (fields: DemoCardFields): DemoCardErrors => {
  const errors: DemoCardErrors = {};
  if (!fields.cardholderName.trim()) errors.cardholderName = "required";
  if (!/^\d{16}$/.test(fields.cardNumber.replace(/\s/g, ""))) errors.cardNumber = "invalid";
  if (!/^\d{3,4}$/.test(fields.cvc)) errors.cvc = "invalid";
  const match = /^(\d{2})\/(\d{2})$/.exec(fields.expiry);
  if (!match || Number(match[1]) < 1 || Number(match[1]) > 12) {
    errors.expiry = "invalid";
  } else {
    const expiryEnd = new Date(2000 + Number(match[2]), Number(match[1]), 1);
    if (expiryEnd <= new Date()) errors.expiry = "expired";
  }
  return errors;
};

export { validateDemoCard };
export type { DemoCardErrors, DemoCardFields };
