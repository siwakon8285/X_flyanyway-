"use client";

import { type KeyboardEvent, useState } from "react";

import { useLanguage } from "@/i18n/LanguageProvider";
import { formatPrice, formatStaffDate } from "@/i18n/formatters";
import type { DashboardData } from "@/lib/admin/dashboardTypes";
import { CabinMix, cabinKeys } from "./DashboardCharts";

const SectionHeading = ({ number, title, subtitle }: { number: string; title: string; subtitle: string }) => <div className="exec-section-title">
  <span aria-hidden="true">{number}</span><div><h2>{title}</h2><p>{subtitle}</p></div>
</div>;

export function DashboardDetail({ data }: { data: DashboardData }) {
  const { locale, t } = useLanguage();
  const [ranking, setRanking] = useState<"bookings" | "revenue">("bookings");
  const number = (value: number) => new Intl.NumberFormat(locale, { maximumFractionDigits: 1 }).format(value);
  const money = (value: number) => formatPrice(value, locale);
  const percent = (value: number | null) => value === null ? "—" : `${number(value)}%`;
  const summary = data.summary;
  const topRoute = data.routes[0];
  const topCabin = [...data.cabins].sort((a, b) => b.bookings - a.bookings || a.cabin.localeCompare(b.cabin))[0];
  const rankedFlights = ranking === "bookings" ? data.flights : data.revenueFlights;
  const occupancy = Math.max(0, Math.min(data.inventory.occupancyPercent ?? 0, 100));
  const changeRankingWithKeyboard = (event: KeyboardEvent<HTMLButtonElement>) => {
    if (!["ArrowLeft", "ArrowRight", "Home", "End"].includes(event.key)) return;
    event.preventDefault();
    const next = event.key === "ArrowLeft" || event.key === "Home" ? "bookings" : "revenue";
    setRanking(next);
    event.currentTarget.parentElement?.querySelector<HTMLButtonElement>(`#flight-tab-${next}`)?.focus();
  };
  const insights = [
    summary.totalBookings === 0 ? t("dashboard.noInsights") : null,
    topRoute && summary.grossRevenue > 0 ? t("dashboard.leadingRoute", { route: topRoute.route.replace("-", " → "), share: number((topRoute.revenue / summary.grossRevenue) * 100) }) : null,
    topCabin && summary.totalBookings > 0 ? t("dashboard.leadingCabin", { cabin: t(`dashboard.${cabinKeys[topCabin.cabin]}`), share: number((topCabin.bookings / summary.totalBookings) * 100) }) : null,
    summary.attentionRefundCount > 0 ? t("dashboard.refundInsight", { count: number(summary.attentionRefundCount) }) : null,
  ].filter((insight): insight is string => Boolean(insight));

  return <div>
    <div className="exec-editorial-grid">
      <section className="exec-panel exec-routes" data-exec-reveal>
        <SectionHeading number="01 /" title={t("dashboard.routeTitle")} subtitle={t("dashboard.routeSubtitle")} />
        {data.routes.length ? <ol>{data.routes.map((route, index) => <li key={route.route} tabIndex={0} data-exec-follow>
          <div className="exec-route-top"><span className="exec-rank">{String(index + 1).padStart(2, "0")}</span><strong>{route.route.replace("-", " → ")}</strong><span>{money(route.revenue)}</span></div>
          <div className="exec-route-track" aria-hidden="true"><span data-route-line style={{ width: `${topRoute.revenue ? (route.revenue / topRoute.revenue) * 100 : 0}%` }} /><i>›</i></div>
          <small>{number(route.bookings)} {t("dashboard.bookingsShort")}</small>
        </li>)}</ol> : <p className="exec-muted">{t("dashboard.noRows")}</p>}
      </section>

      <section className="exec-panel exec-cabin-panel" data-exec-reveal>
        <SectionHeading number="02 /" title={t("dashboard.cabinTitle")} subtitle={t("dashboard.cabinSubtitle")} />
        <div data-exec-follow><CabinMix cabins={data.cabins} /></div>
      </section>
    </div>

    <section className="exec-recovery exec-panel" data-exec-reveal>
      <SectionHeading number="03 /" title={t("dashboard.recoveryTitle")} subtitle={t("dashboard.recoverySubtitle")} />
      <dl className="exec-recovery-grid">
        <div data-exec-follow><span className="exec-status-code">CXL</span><dt>{t("dashboard.cancelled")}</dt><dd>{number(summary.cancelledBookings)}</dd><small>{t("dashboard.statusRecorded")}</small></div>
        <div data-exec-follow><span className="exec-status-code">RFD</span><dt>{t("dashboard.refundCount")}</dt><dd>{number(summary.refundCount)}</dd><small>{t("dashboard.completed")}</small></div>
        <div className="exec-recovery-value" data-exec-follow><span className="exec-status-code">THB</span><dt>{t("dashboard.refunds")}</dt><dd>{money(summary.refundValue)}</dd><small>{t("dashboard.completed")}</small></div>
        <div data-exec-follow><span className="exec-status-code">PND</span><dt>{t("dashboard.pending")}</dt><dd>{number(summary.pendingRefundCount)}</dd><small>{money(summary.pendingRefundValue)}</small></div>
        <div className={summary.attentionRefundCount ? "exec-attention" : ""} data-exec-follow><span className="exec-status-code">CHK</span><dt>{t("dashboard.attention")}</dt><dd>{number(summary.attentionRefundCount)}</dd><small>{summary.attentionRefundCount ? t("dashboard.reviewState") : t("dashboard.clearState")}</small></div>
      </dl>
    </section>

    <section className="exec-panel exec-flight-panel" data-exec-reveal>
      <div className="exec-flight-heading"><SectionHeading number="04 /" title={t("dashboard.flightTitle")} subtitle={t("dashboard.flightSubtitle")} />
        <div className="exec-ranking-toggle" role="tablist" aria-label={t("dashboard.rankingView")}>
          <button id="flight-tab-bookings" type="button" role="tab" tabIndex={ranking === "bookings" ? 0 : -1} aria-selected={ranking === "bookings"} aria-controls="flight-ranking" onClick={() => setRanking("bookings")} onKeyDown={changeRankingWithKeyboard}>{t("dashboard.mostBooked")}</button>
          <button id="flight-tab-revenue" type="button" role="tab" tabIndex={ranking === "revenue" ? 0 : -1} aria-selected={ranking === "revenue"} aria-controls="flight-ranking" onClick={() => setRanking("revenue")} onKeyDown={changeRankingWithKeyboard}>{t("dashboard.highestRevenue")}</button>
        </div>
      </div>
      <div className="exec-table-scroll exec-ranking-board" id="flight-ranking" tabIndex={0} role="tabpanel" aria-labelledby={`flight-tab-${ranking}`} data-exec-follow>
        <table><thead><tr><th scope="col"><span className="sr-only">{t("dashboard.rank")}</span></th><th scope="col">{t("dashboard.flight")}</th><th scope="col">{t("dashboard.route")}</th><th scope="col">{t("dashboard.bookingsShort")}</th><th scope="col">{t("dashboard.revenueShort")}</th></tr></thead>
          <tbody className="exec-board-body" key={ranking}>{rankedFlights.map((flight, index) => <tr key={`${flight.flightNumber}-${flight.departureDate}`}><td className="exec-board-rank">{String(index + 1).padStart(2, "0")}</td><th scope="row">{flight.flightNumber}<small>{formatStaffDate(flight.departureDate, locale)}</small></th><td className="exec-board-route">{flight.route.replace("-", " → ")}</td><td>{number(flight.bookings)}</td><td>{money(flight.revenue)}</td></tr>)}</tbody>
        </table>
      </div>{!rankedFlights.length && <p className="exec-muted">{t("dashboard.noRows")}</p>}
    </section>

    <section className="exec-inventory exec-panel" data-exec-reveal>
      <SectionHeading number="05 /" title={t("dashboard.inventoryTitle")} subtitle={t("dashboard.inventorySubtitle")} />
      <div className="exec-inventory-instrument" data-exec-follow>
        <div className="exec-inventory-value"><span>{t("dashboard.occupancy")}</span><strong>{percent(data.inventory.occupancyPercent)}</strong></div>
        <div className="exec-capacity-readout"><span>{number(data.inventory.bookedSeats)} {t("dashboard.booked")}</span><span>{number(data.inventory.sellableSeats)} {t("dashboard.sellable")}</span></div>
        <div className="exec-capacity-rail" role="img" aria-label={t("dashboard.occupancyLabel", { percent: percent(data.inventory.occupancyPercent), booked: number(data.inventory.bookedSeats), sellable: number(data.inventory.sellableSeats) })}>
          <span data-occupancy-fill style={{ width: `${occupancy}%` }} /><i aria-hidden="true">{Array.from({ length: 11 }, (_, index) => <b key={index} />)}</i>
        </div>
        <p>{t("dashboard.inventoryNote")}</p>
      </div>
      <h3>{t("dashboard.lowDemand")}</h3><div className="exec-table-scroll exec-occupancy-board" tabIndex={0} role="region" aria-label={t("dashboard.lowDemand")} data-exec-follow><table><thead><tr><th scope="col">{t("dashboard.flight")}</th><th scope="col">{t("dashboard.route")}</th><th scope="col">{t("dashboard.seats")}</th><th scope="col">{t("dashboard.occupancy")}</th></tr></thead><tbody>{data.inventory.flights.map((flight) => <tr key={`${flight.flightNumber}-${flight.departureDate}`}><th scope="row">{flight.flightNumber}<small>{formatStaffDate(flight.departureDate, locale)}</small></th><td>{flight.route.replace("-", " → ")}</td><td>{number(flight.bookedSeats)} / {number(flight.sellableSeats)}</td><td><span className="exec-row-occupancy"><i style={{ width: `${Math.max(0, Math.min(flight.occupancyPercent ?? 0, 100))}%` }} />{percent(flight.occupancyPercent)}</span></td></tr>)}</tbody></table></div>{!data.inventory.flights.length && <p className="exec-muted">{t("dashboard.noRows")}</p>}
    </section>

    <aside className="exec-insights" data-exec-reveal><div><p className="exec-eyebrow">X-FLY / {t("dashboard.briefCode")}</p><h2>{t("dashboard.insights")}</h2><p className="exec-muted">{t("dashboard.insightsNote")}</p></div><ol>{insights.map((insight, index) => <li key={insight} data-exec-follow><span>{String(index + 1).padStart(2, "0")}</span><p>{insight}</p></li>)}</ol></aside>
  </div>;
}
