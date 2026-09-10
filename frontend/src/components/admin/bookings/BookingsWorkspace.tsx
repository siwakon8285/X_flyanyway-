"use client";

import { ArrowRight, Filter, Search } from "lucide-react";
import Link from "next/link";
import { useCallback, useEffect, useState } from "react";

import { cabinKeys, formatBookingDateTime, isLegacyCabin, passengerCountKey, statusKeys } from "./bookingPresentation";
import { DatePickerField } from "@/components/admin/flights/DatePickerField";
import { useLanguage } from "@/i18n/LanguageProvider";
import { formatStaffDate } from "@/i18n/formatters";
import type { BookingListItem, BookingListPage } from "@/lib/admin/bookingTypes";
import "./bookingOperations.css";

const emptyFilters = { bookingReference: "", passengerName: "", flightNumber: "", travelDate: "", bookingStatus: "", cabin: "", origin: "", destination: "" };

const BookingsWorkspace = () => {
  const { locale, t } = useLanguage();
  const [filters, setFilters] = useState(emptyFilters);
  const [applied, setApplied] = useState(emptyFilters);
  const [items, setItems] = useState<BookingListItem[]>([]);
  const [nextOffset, setNextOffset] = useState<number | null>(null);
  const [loading, setLoading] = useState(true);
  const [errorStatus, setErrorStatus] = useState<number | null>(null);
  const [requestVersion, setRequestVersion] = useState(0);

  const requestPage = useCallback(async (offset = 0, signal?: AbortSignal) => {
    const body = Object.fromEntries(Object.entries(applied).filter(([, value]) => value));
    const response = await fetch("/admin/api/bookings/search", {
      method: "POST", credentials: "same-origin", cache: "no-store", signal,
      headers: { "content-type": "application/json", "x-x-fly-csrf": "1" },
      body: JSON.stringify({ ...body, limit: 50, offset }),
    });
    if (!response.ok) throw response.status;
    return response.json() as Promise<BookingListPage>;
  }, [applied]);

  useEffect(() => {
    const controller = new AbortController();
    requestPage(0, controller.signal)
      .then((page) => { setItems(page.items); setNextOffset(page.nextOffset); })
      .catch((cause) => { if (!(cause instanceof DOMException && cause.name === "AbortError")) setErrorStatus(typeof cause === "number" ? cause : 503); })
      .finally(() => setLoading(false));
    return () => controller.abort();
  }, [requestPage, requestVersion]);

  const loadMore = async (offset: number) => {
    setLoading(true); setErrorStatus(null);
    try { const page = await requestPage(offset); setItems((current) => [...current, ...page.items]); setNextOffset(page.nextOffset); }
    catch (cause) { setErrorStatus(typeof cause === "number" ? cause : 503); }
    finally { setLoading(false); }
  };
  const errorMessage = errorStatus === 401 ? t("bookingManagement.sessionExpired") : errorStatus === 403 ? t("bookingManagement.forbidden") : errorStatus === 422 ? t("bookingManagement.invalid") : t("bookingManagement.error");

  return <section aria-labelledby="booking-operations-title" className="xbo-shell">
    <header className="xbo-header">
      <div><p className="xbo-kicker">{t("bookingManagement.eyebrow")}</p><h1 id="booking-operations-title">{t("bookingManagement.title")}</h1><p>{t("bookingManagement.intro")}</p></div>
      <div className="xbo-terminal"><span>XF / OPS—22</span><strong>{t("bookingManagement.terminal")}</strong></div>
    </header>

    <form aria-label={t("bookingManagement.filters")} className="xbo-filters" onSubmit={(event) => { event.preventDefault(); setItems([]); setLoading(true); setErrorStatus(null); setApplied(filters); setRequestVersion((value) => value + 1); }}>
      <div className="xbo-filter-title"><span><Search aria-hidden="true" />{t("bookingManagement.filterTitle")}</span><Filter aria-hidden="true" /></div>
      <div className="xbo-filter-grid">
        {(["bookingReference", "passengerName", "flightNumber", "origin", "destination"] as const).map((field) => <label key={field}>{t(`bookingManagement.${field}`)}<input autoComplete="off" onChange={(event) => setFilters((current) => ({ ...current, [field]: event.target.value }))} value={filters[field]} /></label>)}
        <DatePickerField className="xbo-date-field" label={t("bookingManagement.travelDate")} onChange={(travelDate) => setFilters((current) => ({ ...current, travelDate }))} value={filters.travelDate} />
        <label>{t("bookingManagement.bookingStatus")}<select onChange={(event) => setFilters((current) => ({ ...current, bookingStatus: event.target.value }))} value={filters.bookingStatus}><option value="">{t("bookingManagement.all")}</option><option value="CONFIRMED">{t(statusKeys.CONFIRMED)}</option><option value="CANCELLED">{t(statusKeys.CANCELLED)}</option></select></label>
        <label>{t("bookingManagement.cabin")}<select onChange={(event) => setFilters((current) => ({ ...current, cabin: event.target.value }))} value={filters.cabin}><option value="">{t("bookingManagement.all")}</option>{(["business", "first"] as const).map((cabin) => <option key={cabin} value={cabin}>{t(cabinKeys[cabin])}</option>)}</select></label>
      </div>
      <div className="xbo-filter-actions"><button className="xbo-primary" type="submit">{t("bookingManagement.apply")}</button><button onClick={() => { setFilters(emptyFilters); setApplied(emptyFilters); setItems([]); setLoading(true); setErrorStatus(null); setRequestVersion((value) => value + 1); }} type="button">{t("bookingManagement.clear")}</button></div>
    </form>

    {loading && items.length === 0 ? <p aria-live="polite" className="xbo-message">{t("bookingManagement.loading")}</p> : null}
    {errorStatus ? <div className="xbo-error" role="alert"><p>{errorMessage}</p><button onClick={() => { setLoading(true); setErrorStatus(null); setRequestVersion((value) => value + 1); }} type="button">{t("bookingManagement.retry")}</button></div> : null}
    {!loading && !errorStatus && items.length === 0 ? <div className="xbo-empty"><strong>{t("bookingManagement.empty")}</strong><span>{t("bookingManagement.emptyHelp")}</span></div> : null}
    {items.length ? <div aria-label={t("bookingManagement.title")} className="xbo-table-wrap" role="region" tabIndex={0}><table><caption className="sr-only">{t("bookingManagement.title")}</caption><thead><tr><th scope="col">{t("bookingManagement.bookingReference")}</th><th scope="col">{t("bookingManagement.lead")}</th><th scope="col">{t("bookingManagement.journey")}</th><th scope="col">{t("bookingManagement.cabin")}</th><th scope="col">{t("bookingManagement.bookingStatus")}</th><th scope="col">{t("bookingManagement.payment")}</th><th scope="col">{t("bookingManagement.ticket")}</th><th scope="col">{t("bookingManagement.actions")}</th></tr></thead><tbody>{items.map((booking) => <tr key={booking.bookingReference}>
      <th scope="row"><span className="xbo-reference">{booking.bookingReference}</span><time dateTime={booking.bookedAt}>{formatBookingDateTime(booking.bookedAt, locale)}</time></th>
      <td><strong>{booking.leadPassengerName}</strong><span>{t(passengerCountKey(booking.passengerCount), { count: booking.passengerCount })}</span></td>
      <td><strong>{booking.flightNumber} · {booking.originCode} → {booking.destinationCode}</strong><time dateTime={booking.travelDate}>{formatStaffDate(booking.travelDate, locale)}</time><span className={`xbo-chip ${booking.flightStatus === "CANCELLED" ? "is-alert" : ""}`}>{t(statusKeys[booking.flightStatus])}</span></td>
      <td><span>{t(cabinKeys[booking.cabin])}</span>{isLegacyCabin(booking.cabin) ? <small>{t("bookingManagement.historical")}</small> : null}</td>
      <td><span className={`xbo-chip ${booking.bookingStatus === "CANCELLED" ? "is-muted" : ""}`}>{t(statusKeys[booking.bookingStatus])}</span></td>
      <td><span className="xbo-chip">{t(statusKeys[booking.paymentStatus])}</span></td><td><span className="xbo-chip">{t(statusKeys[booking.ticketStatus])}</span></td>
      <td><Link aria-label={`${t("bookingManagement.open")} ${booking.bookingReference}`} className="xbo-open" href={`/admin/bookings/${booking.bookingReference}`}>{t("bookingManagement.open")}<ArrowRight aria-hidden="true" /></Link></td>
    </tr>)}</tbody></table></div> : null}
    {nextOffset !== null ? <button className="xbo-load-more" disabled={loading} onClick={() => void loadMore(nextOffset)} type="button">{loading ? t("bookingManagement.loading") : t("bookingManagement.loadMore")}</button> : items.length ? <p className="xbo-end">{t("bookingManagement.end")}</p> : null}
  </section>;
};

export { BookingsWorkspace };
