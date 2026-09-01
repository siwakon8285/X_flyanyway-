"use client";

import { ArrowLeft, ArrowRight, Check, Plus, Trash2 } from "lucide-react";
import { useEffect, useRef, useState, type ComponentProps, type FormEvent, type ReactNode } from "react";

import { CountrySelect } from "@/components/booking/passengers/CountrySelect";
import { countryCallingCode } from "@/components/booking/passengers/countryCallingCodes";
import { passengerTypeKeys, validationKeys } from "@/components/booking/passengers/passengerPresentation";
import type {
  PassengerFieldName,
  PassengerFormValue,
  PassengerValidationError,
} from "@/components/booking/passengers/passengerTypes";
import { Button } from "@/components/ui/Button";
import { Input } from "@/components/ui/Input";
import { Label } from "@/components/ui/Label";
import { useLanguage } from "@/i18n/LanguageProvider";
import { cn } from "@/lib/utils/cn";

type UpdateField = <Field extends keyof PassengerFormValue>(
  field: Field,
  value: PassengerFormValue[Field],
) => void;

const PassengerField = ({
  children,
  error,
  id,
  label,
  optional = false,
  shake = false,
}: {
  children: ReactNode;
  error?: string;
  id: string;
  label: string;
  optional?: boolean;
  shake?: boolean;
}) => {
  const { t } = useLanguage();
  const errorId = `${id}-error`;
  return (
    <div className={cn("min-w-0", shake && "motion-safe:animate-[passenger-field-shake_180ms_ease-out_1] motion-reduce:animate-none")}>
      <div className="mb-2 flex items-baseline justify-between gap-3">
        <Label htmlFor={id}>{label}</Label>
        <span className={cn("text-xs", optional ? "text-muted-foreground" : "text-destructive")}>
          {t(optional ? "passengerInformation.optional" : "passengerInformation.required")}
        </span>
      </div>
      {children}
      {error ? (
        <p className="mt-2 text-xs text-destructive" id={errorId}>
          {error}
        </p>
      ) : null}
    </div>
  );
};

const PassengerDateInput = ({
  onClick,
  ...props
}: ComponentProps<typeof Input>) => (
  <Input
    {...props}
    className={cn("transition-[border-color,box-shadow,transform] duration-150 motion-safe:focus-visible:scale-[1.005] motion-reduce:transition-none", props.className)}
    onClick={(event) => {
      onClick?.(event);
      if (event.defaultPrevented) return;
      event.currentTarget.focus();
      try {
        event.currentTarget.showPicker?.();
      } catch {
        // Some browsers restrict showPicker to a trusted interaction; the native field remains usable.
      }
    }}
    type="date"
  />
);

const NativeSelect = ({
  describedBy,
  error,
  id,
  label,
  onChange,
  options,
  value,
}: {
  describedBy?: string;
  error?: boolean;
  id: string;
  label: string;
  onChange: (value: string) => void;
  options: Array<{ label: string; value: string }>;
  value: string;
}) => (
  <select
    aria-describedby={describedBy}
    aria-invalid={error || undefined}
    aria-label={label}
    className={cn(
      "h-12 w-full rounded-control border border-border bg-surface px-4 text-base text-foreground outline-none focus-visible:border-focus focus-visible:ring-2 focus-visible:ring-focus/25",
      error && "border-destructive ring-2 ring-destructive/25",
    )}
    id={id}
    onChange={(event) => onChange(event.target.value)}
    value={value}
  >
    <option value="" />
    {options.map((option) => (
      <option key={option.value} value={option.value}>
        {option.label}
      </option>
    ))}
  </select>
);

const PassengerForm = ({
  errors,
  onSave,
  onValuesChange,
  ready,
  recentlySaved,
  saving,
  values,
}: {
  errors: PassengerValidationError[];
  onSave: () => void;
  onValuesChange: (values: PassengerFormValue[]) => void;
  ready: boolean;
  recentlySaved: boolean;
  saving: boolean;
  values: PassengerFormValue[];
}) => {
  const { t } = useLanguage();
  const formRef = useRef<HTMLFormElement>(null);
  const manuallyEditedPhoneCodes = useRef(new Set(values.filter((value) => value.phoneCountryCode).map((value) => value.ordinal)));
  const [activeIndex, setActiveIndex] = useState(0);
  const [openCountrySelectId, setOpenCountrySelectId] = useState<string | null>(null);
  const [shakeTarget, setShakeTarget] = useState<string | null>(null);
  const passenger = values[activeIndex];
  const firstErrorPassenger = errors[0]?.passenger;
  const firstErrorField = errors[0]?.field;
  useEffect(() => {
    if (firstErrorPassenger === undefined || !firstErrorField) return;
    const index = values.findIndex((value) => value.ordinal === firstErrorPassenger);
    window.requestAnimationFrame(() => {
      if (index >= 0) setActiveIndex(index);
      setShakeTarget(`passenger-${firstErrorPassenger}-${firstErrorField}`);
      window.requestAnimationFrame(() => {
        const field = firstErrorField === "emergencyContact" ? "emergencyName" : firstErrorField;
        formRef.current
          ?.querySelector<HTMLElement>(`#passenger-${firstErrorPassenger}-${field}`)
          ?.focus();
      });
    });
  }, [firstErrorField, firstErrorPassenger, values]);
  if (!passenger) return null;
  const passengerLabel = (value: PassengerFormValue) =>
    t("passengerInformation.passenger.label", {
      ordinal: value.ordinal,
      type: t(passengerTypeKeys[value.passengerType]),
    });
  const errorFor = (field: PassengerFieldName) =>
    errors.find(
      (error) => error.passenger === passenger.ordinal && error.field === field,
    );
  const translatedError = (field: PassengerFieldName) => {
    const error = errorFor(field);
    return error ? t(validationKeys[error.code]) : undefined;
  };
  const controlProps = (field: PassengerFieldName) => {
    const id = `passenger-${passenger.ordinal}-${field}`;
    const error = Boolean(errorFor(field));
    return {
      "aria-describedby": error ? `${id}-error` : undefined,
      "aria-invalid": error || undefined,
      id,
    };
  };
  const selectControlProps = (field: PassengerFieldName) => {
    const props = controlProps(field);
    return {
      describedBy: props["aria-describedby"],
      error: Boolean(props["aria-invalid"]),
      id: props.id,
    };
  };
  const update: UpdateField = (field, value) => {
    onValuesChange(
      values.map((item, index) =>
        index === activeIndex ? { ...item, [field]: value } : item,
      ),
    );
  };
  const updateNationality = (nationalityCode: string) => {
    const defaultPhoneCode = countryCallingCode(nationalityCode);
    onValuesChange(values.map((item, index) => {
      if (index !== activeIndex) return item;
      return {
        ...item,
        nationalityCode,
        phoneCountryCode: manuallyEditedPhoneCodes.current.has(item.ordinal)
          ? item.phoneCountryCode
          : defaultPhoneCode ?? item.phoneCountryCode,
      };
    }));
  };
  const countrySelectState = (id: string) => ({
    onOpenChange: (open: boolean) => setOpenCountrySelectId(open ? id : null),
    open: openCountrySelectId === id,
  });
  const submit = (event: FormEvent<HTMLFormElement>) => {
    event.preventDefault();
    onSave();
  };

  const titleOptions = [
    { label: t("passengerInformation.title.mr"), value: "MR" },
    { label: t("passengerInformation.title.ms"), value: "MS" },
  ];
  const genderOptions = [
    { label: t("passengerInformation.gender.male"), value: "MALE" },
    { label: t("passengerInformation.gender.female"), value: "FEMALE" },
    { label: t("passengerInformation.gender.unspecified"), value: "UNSPECIFIED" },
  ];

  return (
    <form aria-label={t("passengerInformation.formLabel")} noValidate onSubmit={submit} ref={formRef}>
      <nav aria-label={t("passengerInformation.passengerNav")} className="flex gap-2 overflow-x-auto pb-2">
        {values.map((value, index) => (
          <button
            aria-current={index === activeIndex ? "step" : undefined}
            className={cn(
              "min-h-11 shrink-0 rounded-control border px-4 text-sm font-medium outline-none focus-visible:ring-2 focus-visible:ring-focus",
              index === activeIndex
                ? "border-brand bg-brand/10 text-brand"
                : "border-border bg-surface text-muted-foreground",
            )}
            key={value.ordinal}
            onClick={() => setActiveIndex(index)}
            type="button"
          >
            {passengerLabel(value)}
          </button>
        ))}
      </nav>

      <fieldset
        aria-label={passengerLabel(passenger)}
        className="mt-6 rounded-surface border border-border bg-surface/55 p-5 sm:p-7"
      >
        <legend className="sr-only">{passengerLabel(passenger)}</legend>
        <div className="flex flex-wrap items-end justify-between gap-3 border-b border-border pb-5">
          <div>
            <p className="text-caption text-brand">{passengerLabel(passenger)}</p>
            <h2 className="mt-2 text-h3">{t("passengerInformation.identity")}</h2>
          </div>
          <p className="text-sm font-medium text-brand">{t("passengerInformation.nameWarning")}</p>
        </div>

        <div className="mt-6 grid gap-5 sm:grid-cols-2 lg:grid-cols-3">
          <PassengerField id={controlProps("title").id} label={t("passengerInformation.field.title")} error={translatedError("title")}>
            <NativeSelect {...selectControlProps("title")} label={t("passengerInformation.field.title")} onChange={(value) => update("title", value as PassengerFormValue["title"])} options={titleOptions} value={passenger.title} />
          </PassengerField>
          <PassengerField id={controlProps("givenName").id} label={t("passengerInformation.field.givenName")} error={translatedError("givenName")}>
            <Input {...controlProps("givenName")} autoComplete="given-name" onChange={(event) => update("givenName", event.target.value)} value={passenger.givenName} />
          </PassengerField>
          <PassengerField optional id={controlProps("middleName").id} label={t("passengerInformation.field.middleName")} error={translatedError("middleName")}>
            <Input {...controlProps("middleName")} autoComplete="additional-name" onChange={(event) => update("middleName", event.target.value)} value={passenger.middleName} />
          </PassengerField>
          <PassengerField id={controlProps("familyName").id} label={t("passengerInformation.field.familyName")} error={translatedError("familyName")}>
            <Input {...controlProps("familyName")} autoComplete="family-name" onChange={(event) => update("familyName", event.target.value)} value={passenger.familyName} />
          </PassengerField>
          <PassengerField id={controlProps("dateOfBirth").id} label={t("passengerInformation.field.dateOfBirth")} error={translatedError("dateOfBirth")} shake={shakeTarget === controlProps("dateOfBirth").id}>
            <PassengerDateInput {...controlProps("dateOfBirth")} autoComplete="bday" onChange={(event) => update("dateOfBirth", event.target.value)} value={passenger.dateOfBirth} />
          </PassengerField>
          <PassengerField id={controlProps("gender").id} label={t("passengerInformation.field.gender")} error={translatedError("gender")}>
            <NativeSelect {...selectControlProps("gender")} label={t("passengerInformation.field.gender")} onChange={(value) => update("gender", value as PassengerFormValue["gender"])} options={genderOptions} value={passenger.gender} />
          </PassengerField>
          <PassengerField id={controlProps("nationalityCode").id} label={t("passengerInformation.field.nationality")} error={translatedError("nationalityCode")} shake={shakeTarget === controlProps("nationalityCode").id}>
            <CountrySelect {...selectControlProps("nationalityCode")} {...countrySelectState(controlProps("nationalityCode").id)} label={t("passengerInformation.field.nationality")} onChange={updateNationality} value={passenger.nationalityCode} />
          </PassengerField>
        </div>

        <h3 className="mt-10 border-t border-border pt-7 text-xl font-semibold">{t("passengerInformation.passport")}</h3>
        <div className="mt-5 grid gap-5 sm:grid-cols-2">
          <PassengerField id={controlProps("passportNumber").id} label={t("passengerInformation.field.passportNumber")} error={translatedError("passportNumber")}>
            <Input {...controlProps("passportNumber")} autoCapitalize="characters" onChange={(event) => update("passportNumber", event.target.value)} value={passenger.passportNumber} />
          </PassengerField>
          <PassengerField id={controlProps("passportIssuingCountryCode").id} label={t("passengerInformation.field.passportIssuingCountry")} error={translatedError("passportIssuingCountryCode")}>
            <CountrySelect {...selectControlProps("passportIssuingCountryCode")} {...countrySelectState(controlProps("passportIssuingCountryCode").id)} label={t("passengerInformation.field.passportIssuingCountry")} onChange={(value) => update("passportIssuingCountryCode", value)} value={passenger.passportIssuingCountryCode} />
          </PassengerField>
        </div>

        <h3 className="mt-10 border-t border-border pt-7 text-xl font-semibold">{t("passengerInformation.contact")}</h3>
        <div className="mt-5 grid gap-5 sm:grid-cols-2 lg:grid-cols-[1fr_10rem_1fr]">
          <PassengerField id={controlProps("email").id} label={t("passengerInformation.field.email")} error={translatedError("email")}>
            <Input {...controlProps("email")} autoComplete="email" inputMode="email" onChange={(event) => update("email", event.target.value)} type="email" value={passenger.email} />
          </PassengerField>
          <PassengerField id={controlProps("phoneCountryCode").id} label={t("passengerInformation.field.phoneCountryCode")} error={translatedError("phoneCountryCode")} shake={shakeTarget === controlProps("phoneCountryCode").id}>
            <CountrySelect {...selectControlProps("phoneCountryCode")} {...countrySelectState(controlProps("phoneCountryCode").id)} label={t("passengerInformation.field.phoneCountryCode")} mode="callingCode" onChange={(value) => { manuallyEditedPhoneCodes.current.add(passenger.ordinal); update("phoneCountryCode", value); }} value={passenger.phoneCountryCode} />
          </PassengerField>
          <PassengerField id={controlProps("phoneNumber").id} label={t("passengerInformation.field.phoneNumber")} error={translatedError("phoneNumber")}>
            <Input {...controlProps("phoneNumber")} autoComplete="tel-national" inputMode="tel" onChange={(event) => update("phoneNumber", event.target.value)} value={passenger.phoneNumber} />
          </PassengerField>
        </div>

        <div className="mt-10 border-t border-border pt-7">
          <div className="flex items-center justify-between gap-4">
            <h3 className="text-xl font-semibold">{t("passengerInformation.emergency")}</h3>
            <span className="text-xs text-muted-foreground">{t("passengerInformation.optional")}</span>
          </div>
          {passenger.emergencyContact ? (
            <>
              <div className="mt-5 grid gap-5 sm:grid-cols-2">
                <PassengerField id={`passenger-${passenger.ordinal}-emergencyName`} label={t("passengerInformation.field.emergencyName")} error={translatedError("emergencyContact")}>
                  <Input aria-describedby={errorFor("emergencyContact") ? `passenger-${passenger.ordinal}-emergencyName-error` : undefined} aria-invalid={Boolean(errorFor("emergencyContact")) || undefined} id={`passenger-${passenger.ordinal}-emergencyName`} onChange={(event) => update("emergencyContact", { ...passenger.emergencyContact!, name: event.target.value })} value={passenger.emergencyContact.name} />
                </PassengerField>
                <PassengerField id={`passenger-${passenger.ordinal}-relationship`} label={t("passengerInformation.field.relationship")}>
                  <Input id={`passenger-${passenger.ordinal}-relationship`} onChange={(event) => update("emergencyContact", { ...passenger.emergencyContact!, relationship: event.target.value })} value={passenger.emergencyContact.relationship} />
                </PassengerField>
                <PassengerField id={`passenger-${passenger.ordinal}-emergencyCountryCode`} label={t("passengerInformation.field.phoneCountryCode")}>
                  <CountrySelect {...countrySelectState(`passenger-${passenger.ordinal}-emergencyCountryCode`)} id={`passenger-${passenger.ordinal}-emergencyCountryCode`} label={t("passengerInformation.countrySelector.emergencyPhoneLabel")} mode="callingCode" onChange={(value) => update("emergencyContact", { ...passenger.emergencyContact!, phoneCountryCode: value })} value={passenger.emergencyContact.phoneCountryCode} />
                </PassengerField>
                <PassengerField id={`passenger-${passenger.ordinal}-emergencyPhone`} label={t("passengerInformation.field.phoneNumber")}>
                  <Input id={`passenger-${passenger.ordinal}-emergencyPhone`} inputMode="tel" onChange={(event) => update("emergencyContact", { ...passenger.emergencyContact!, phoneNumber: event.target.value })} value={passenger.emergencyContact.phoneNumber} />
                </PassengerField>
              </div>
              <Button className="mt-5" onClick={() => update("emergencyContact", null)} variant="ghost">
                <Trash2 aria-hidden="true" />{t("passengerInformation.removeEmergency")}
              </Button>
            </>
          ) : (
            <Button className="mt-5" onClick={() => update("emergencyContact", { name: "", relationship: "", phoneCountryCode: "", phoneNumber: "" })} variant="outline">
              <Plus aria-hidden="true" />{t("passengerInformation.addEmergency")}
            </Button>
          )}
        </div>
      </fieldset>

      <div className="mt-6 flex flex-wrap items-center justify-between gap-3">
        <Button disabled={activeIndex === 0} onClick={() => setActiveIndex((index) => Math.max(0, index - 1))} variant="ghost">
          <ArrowLeft aria-hidden="true" />{t("passengerInformation.previousPassenger")}
        </Button>
        {activeIndex < values.length - 1 ? (
          <Button onClick={() => setActiveIndex((index) => Math.min(values.length - 1, index + 1))} variant="outline">
            {t("passengerInformation.nextPassenger")}<ArrowRight aria-hidden="true" />
          </Button>
        ) : null}
      </div>

      {errors.length > 0 ? (
        <p className="mt-5 text-sm text-destructive" role="alert">{t("passengerInformation.validation.review")}</p>
      ) : null}
      {ready ? (
        <p aria-label={t("passengerInformation.savedReady")} className={cn("mt-5 flex items-center gap-2 rounded-control border border-brand/40 bg-brand/10 p-4 text-sm font-medium text-brand", recentlySaved && "motion-safe:animate-[passenger-ready_420ms_ease-out_1] motion-reduce:animate-none")} role="status">
          <Check aria-hidden="true" />{t("passengerInformation.savedReady")}
        </p>
      ) : null}
      <Button className="mt-5 w-full sm:w-auto" loading={saving} size="lg" type="submit">
        {saving ? t("passengerInformation.saving") : t("passengerInformation.save")}
      </Button>
    </form>
  );
};

export { PassengerForm };
