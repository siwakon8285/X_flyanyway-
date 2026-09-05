"use client";

import { useRouter } from "next/navigation";
import { useRef, useState, type FormEvent } from "react";

import { BookingApiError } from "@/components/booking/api/bookingApiClient";
import {
  lookupManageBooking,
} from "@/components/manage-booking/manageBookingClient";
import { Button } from "@/components/ui/Button";
import { Card } from "@/components/ui/Card";
import { Input } from "@/components/ui/Input";
import { Label } from "@/components/ui/Label";
import { Container } from "@/components/layout/Container";
import { useLanguage } from "@/i18n/LanguageProvider";
import type { TranslationKey } from "@/i18n/types";

type LookupValues = { bookingReference: string; lastName: string };
type LookupErrors = Partial<Record<keyof LookupValues, TranslationKey>>;

const validate = (values: LookupValues): LookupErrors => {
  const errors: LookupErrors = {};
  if (!values.bookingReference.trim()) errors.bookingReference = "manageBooking.validation.referenceRequired";
  else if (!/^XF[A-Z2-9]{8}$/.test(values.bookingReference.trim().toUpperCase())) errors.bookingReference = "manageBooking.validation.referenceInvalid";
  if (!values.lastName.trim()) errors.lastName = "manageBooking.validation.lastNameRequired";
  return errors;
};

const ManageBookingPage = () => {
  const { t } = useLanguage();
  const router = useRouter();
  const [values, setValues] = useState<LookupValues>({ bookingReference: "", lastName: "" });
  const [errors, setErrors] = useState<LookupErrors>({});
  const [submitting, setSubmitting] = useState(false);
  const [failure, setFailure] = useState<"notFound" | "unavailable" | null>(null);
  const formRef = useRef<HTMLFormElement>(null);

  const submit = async (event: FormEvent<HTMLFormElement>) => {
    event.preventDefault();
    const nextErrors = validate(values);
    setErrors(nextErrors);
    setFailure(null);
    const firstError = Object.keys(nextErrors)[0] as keyof LookupValues | undefined;
    if (firstError) {
      requestAnimationFrame(() => formRef.current?.querySelector<HTMLElement>(`#manage-${firstError}`)?.focus());
      return;
    }
    setSubmitting(true);
    try {
      await lookupManageBooking({
        bookingReference: values.bookingReference.trim().toUpperCase(),
        lastName: values.lastName.trim().replace(/\s+/g, " "),
      });
      router.replace("/manage-booking/details");
    } catch (error) {
      setFailure(error instanceof BookingApiError && error.code === "BOOKING_NOT_FOUND" ? "notFound" : "unavailable");
    } finally {
      setSubmitting(false);
    }
  };


  return (
    <main className="min-h-screen bg-background pb-24 pt-28">
      <Container className="max-w-3xl">
        <p className="text-label text-brand">{t("manageBooking.eyebrow")}</p>
        <h1 className="mt-3 text-display-sm">{t("manageBooking.heading")}</h1>
        <p className="mt-4 max-w-2xl text-body-lg text-muted-foreground">{t("manageBooking.intro")}</p>
        <Card className="mt-9 p-6 sm:p-8">
          {failure ? <div aria-live="polite" className="mb-6 rounded-control border border-destructive/40 bg-destructive/10 p-4 text-sm" role="alert">{t(failure === "notFound" ? "manageBooking.error.notFound" : "manageBooking.error.unavailable")}</div> : null}
          <form noValidate onSubmit={submit} ref={formRef}>
            <div className="grid gap-6 sm:grid-cols-2">
              <div><Label htmlFor="manage-bookingReference">{t("manageBooking.bookingReference")}</Label><Input aria-describedby={errors.bookingReference ? "manage-bookingReference-error" : undefined} aria-invalid={Boolean(errors.bookingReference) || undefined} autoCapitalize="characters" autoComplete="off" className="mt-2 uppercase" id="manage-bookingReference" onChange={(event) => setValues({ ...values, bookingReference: event.target.value })} value={values.bookingReference} />{errors.bookingReference ? <p className="mt-2 text-sm text-destructive" id="manage-bookingReference-error">{t(errors.bookingReference)}</p> : null}</div>
              <div><Label htmlFor="manage-lastName">{t("manageBooking.lastName")}</Label><Input aria-describedby={errors.lastName ? "manage-lastName-error" : undefined} aria-invalid={Boolean(errors.lastName) || undefined} autoComplete="family-name" id="manage-lastName" onChange={(event) => setValues({ ...values, lastName: event.target.value })} value={values.lastName} />{errors.lastName ? <p className="mt-2 text-sm text-destructive" id="manage-lastName-error">{t(errors.lastName)}</p> : null}</div>
            </div>
            <Button className="mt-7 min-h-12 w-full sm:w-auto" disabled={submitting} type="submit">{t(submitting ? "manageBooking.finding" : "manageBooking.find")}</Button>
          </form>
        </Card>
      </Container>
    </main>
  );
};

export { ManageBookingPage };
