import { formatDate } from "@/i18n/formatters";
import { useLanguage } from "@/i18n/LanguageProvider";
import type { SeatHold } from "@/components/booking/seats/seatHoldClient";
import { FLIGHT_RESULT_FIXTURES } from "@/components/booking/results/flightResultFixtures";

const PassengerSummary = ({
  hold,
  remainingMilliseconds,
}: {
  hold: SeatHold;
  remainingMilliseconds: number;
}) => {
  const { locale, t } = useLanguage();
  const flight = FLIGHT_RESULT_FIXTURES.find((item) => item.id === hold.flightId);
  const minutes = Math.floor(Math.max(0, remainingMilliseconds) / 60_000);
  const seconds = Math.floor((Math.max(0, remainingMilliseconds) % 60_000) / 1_000);

  return (
    <aside className="rounded-surface border border-border bg-surface/80 p-6 lg:sticky lg:top-[calc(var(--header-height)+1.5rem)]">
      <p className="text-caption text-brand">{t("passengerInformation.summary")}</p>
      <p className="mt-4 text-lg font-semibold">
        {flight?.flightNumber ?? hold.flightId.toUpperCase()}
      </p>
      {flight ? (
        <p className="mt-1 text-sm text-muted-foreground">
          {flight.originCode} → {flight.destinationCode}
        </p>
      ) : null}
      <p className="mt-4 text-sm text-muted-foreground">
        {formatDate(hold.departureDate, locale)}
      </p>
      <div className="my-5 h-px bg-border" />
      <p className="text-caption">{t("passengerInformation.seats")}</p>
      <div className="mt-3 flex flex-wrap gap-2">
        {hold.seats.map((seat) => (
          <span
            className="rounded-control border border-brand/45 bg-brand/10 px-3 py-2 text-sm font-semibold text-brand"
            key={seat}
          >
            {seat}
          </span>
        ))}
      </div>
      {hold.passengers.infants > 0 ? (
        <p className="mt-3 text-sm text-muted-foreground">
          {t("passengerInformation.infantNoSeat")}
        </p>
      ) : null}
      <p className="mt-6 text-sm font-semibold tabular-nums text-brand" aria-live="polite">
        {t("seatMap.holdCountdown", {
          time: `${minutes}:${seconds.toString().padStart(2, "0")}`,
        })}
      </p>
    </aside>
  );
};

export { PassengerSummary };
