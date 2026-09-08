"use client";

import { Armchair, ArrowRight, PlaneTakeoff } from "lucide-react";

import { useLanguage } from "@/i18n/LanguageProvider";
import type { AirportReference, ManagedFlight } from "@/lib/admin/flightTypes";
import { cn } from "@/lib/utils/cn";

const OperationsSectionLegend = ({ index, title }: { index: string; title: string }) => (
  <legend className="xfo-section-legend">
    <span aria-hidden="true">{index} /</span>
    <span>{title}</span>
  </legend>
);

const FlightStatusBadge = ({ status }: { status: ManagedFlight["status"] }) => {
  const { t } = useLanguage();
  return (
    <span className={cn("xfo-status", status === "CANCELLED" && "is-cancelled")}>
      <i aria-hidden="true" />
      {t(status === "SCHEDULED" ? "flightManagement.scheduled" : "flightManagement.cancelled")}
    </span>
  );
};

type OperationalSummaryProps = {
  aircraftCode: string;
  airports: readonly AirportReference[];
  arrivalTime: string;
  businessCapacity: string;
  departureTime: string;
  destinationCode: string;
  firstCapacity: string;
  flightNumber: string;
  operatingDate: string;
  originCode: string;
  status?: ManagedFlight["status"];
};

const OperationalSummary = ({
  aircraftCode,
  airports,
  arrivalTime,
  businessCapacity,
  departureTime,
  destinationCode,
  firstCapacity,
  flightNumber,
  operatingDate,
  originCode,
  status,
}: OperationalSummaryProps) => {
  const { locale, t } = useLanguage();
  const origin = airports.find((airport) => airport.code === originCode);
  const destination = airports.find((airport) => airport.code === destinationCode);
  const formattedDate = operatingDate
    ? new Intl.DateTimeFormat(locale === "th" ? "th-TH" : "en-GB", {
        day: "2-digit",
        month: "short",
        year: "numeric",
      }).format(new Date(`${operatingDate}T12:00:00Z`))
    : "—";

  return (
    <aside aria-label={t("flightManagement.operationalSummary")} className="xfo-instrument">
      <div className="xfo-instrument-meta">
        <span>{flightNumber.trim() || t("flightManagement.newService")}</span>
        <span>{aircraftCode || t("flightManagement.aircraftPending")}</span>
      </div>
      <div className="xfo-instrument-route">
        <div>
          <strong>{originCode || "—"}</strong>
          <span>{origin?.city || t("flightManagement.routePending")}</span>
        </div>
        <ArrowRight aria-hidden="true" />
        <div>
          <strong>{destinationCode || "—"}</strong>
          <span>{destination?.city || t("flightManagement.routePending")}</span>
        </div>
      </div>
      <div className="xfo-instrument-grid">
        <div>
          <span>{t("flightManagement.departureDate")}</span>
          <strong>{formattedDate}</strong>
        </div>
        <div>
          <span>{t("flightManagement.schedule")}</span>
          <strong>{departureTime || "—"} → {arrivalTime || "—"}</strong>
        </div>
      </div>
      <div className="xfo-instrument-footer">
        <span><Armchair aria-hidden="true" />{t("flightManagement.business")} <strong>{businessCapacity || "—"}</strong></span>
        <span><PlaneTakeoff aria-hidden="true" />{t("flightManagement.first")} <strong>{firstCapacity || "—"}</strong></span>
        {status ? <FlightStatusBadge status={status} /> : null}
      </div>
    </aside>
  );
};

export { FlightStatusBadge, OperationalSummary, OperationsSectionLegend };
