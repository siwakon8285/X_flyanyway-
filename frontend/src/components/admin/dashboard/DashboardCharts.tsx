"use client";

import { type KeyboardEvent, type PointerEvent, type ReactNode, useId, useState } from "react";

import { useLanguage } from "@/i18n/LanguageProvider";
import { formatDate, formatPrice } from "@/i18n/formatters";
import { dashboardCabins, type DashboardCabin, type DashboardData } from "@/lib/admin/dashboardTypes";

export const cabinKeys = { business: "business", first: "first" } as const;
const cabinColors = ["#FFD400", "#8fa5a3"];
const chartWidth = 680;
const plotStart = 58;
const plotEnd = 650;

const chartGeometry = (points: DashboardData["trends"], metric: "bookings" | "revenue") => {
  const maximum = Math.max(...points.map((point) => point[metric]), 1);
  const coords = points.map((point, index) => ({
    x: points.length === 1 ? 330 : 58 + (index / (points.length - 1)) * 592,
    y: 224 - (point[metric] / maximum) * 176,
  }));
  if (coords.length < 2) return { coords, line: coords.length ? `M${coords[0].x},${coords[0].y}` : "", maximum };

  const slopes = coords.slice(0, -1).map((point, index) => (coords[index + 1].y - point.y) / (coords[index + 1].x - point.x));
  const tangents = coords.map((_, index) => {
    if (index === 0) return slopes[0];
    if (index === coords.length - 1) return slopes.at(-1)!;
    return slopes[index - 1] * slopes[index] <= 0 ? 0 : (slopes[index - 1] + slopes[index]) / 2;
  });
  slopes.forEach((slope, index) => {
    if (slope === 0) {
      tangents[index] = 0;
      tangents[index + 1] = 0;
      return;
    }
    const left = tangents[index] / slope;
    const right = tangents[index + 1] / slope;
    const magnitude = Math.hypot(left, right);
    if (magnitude > 3) {
      const scale = 3 / magnitude;
      tangents[index] = scale * left * slope;
      tangents[index + 1] = scale * right * slope;
    }
  });
  const curves = coords.slice(0, -1).map((point, index) => {
    const next = coords[index + 1];
    const third = (next.x - point.x) / 3;
    return `C${point.x + third},${point.y + tangents[index] * third} ${next.x - third},${next.y - tangents[index + 1] * third} ${next.x},${next.y}`;
  });
  return { coords, line: `M${coords[0].x},${coords[0].y} ${curves.join(" ")}`, maximum };
};

type ChartInspectorProps = {
  children: (activeIndex: number | null) => ReactNode;
  className: string;
  metric: "bookings" | "revenue";
  points: DashboardData["trends"];
  title: string;
};

const ChartInspector = ({ children, className, metric, points, title }: ChartInspectorProps) => {
  const id = useId().replaceAll(":", "");
  const { locale, t } = useLanguage();
  const [activeIndex, setActiveIndex] = useState<number | null>(null);
  const activePoint = activeIndex === null ? undefined : points[activeIndex];
  const geometry = activeIndex === null ? undefined : chartGeometry(points, metric).coords[activeIndex];
  const inspectAtClientX = (event: PointerEvent<SVGSVGElement>) => {
    if (!points.length) return;
    const bounds = event.currentTarget.getBoundingClientRect();
    if (!bounds.width) return;
    const svgX = ((event.clientX - bounds.left) / bounds.width) * chartWidth;
    const ratio = Math.max(0, Math.min(1, (svgX - plotStart) / (plotEnd - plotStart)));
    setActiveIndex(Math.round(ratio * (points.length - 1)));
  };
  const inspectWithKeyboard = (event: KeyboardEvent<SVGSVGElement>) => {
    if (!points.length) return;
    const current = activeIndex ?? points.length - 1;
    let next = current;
    if (event.key === "ArrowLeft") next = Math.max(0, current - 1);
    else if (event.key === "ArrowRight") next = Math.min(points.length - 1, current + 1);
    else if (event.key === "Home") next = 0;
    else if (event.key === "End") next = points.length - 1;
    else if (event.key === "Escape") {
      setActiveIndex(null);
      return;
    } else return;
    event.preventDefault();
    setActiveIndex(next);
  };
  const formattedValue = activePoint
    ? metric === "revenue" ? formatPrice(activePoint.revenue, locale) : new Intl.NumberFormat(locale).format(activePoint.bookings)
    : "";
  const edge = geometry && geometry.x < 190 ? " is-left-edge" : geometry && geometry.x > 510 ? " is-right-edge" : "";
  const vertical = geometry && geometry.y < 108 ? " is-below" : "";

  return <div className="exec-chart-inspector">
    <svg className={`exec-trend ${className}`} viewBox="0 0 680 278" role="img" tabIndex={0} aria-label={title} aria-describedby={`${id}-instructions${activePoint ? ` ${id}-inspection` : ""}`} data-values={points.map((point) => point[metric]).join(",")} onBlur={() => setActiveIndex(null)} onFocus={() => points.length && setActiveIndex((current) => current ?? points.length - 1)} onKeyDown={inspectWithKeyboard} onPointerDown={inspectAtClientX} onPointerMove={inspectAtClientX} onPointerLeave={(event) => { if (event.pointerType === "mouse") setActiveIndex(null); }}>
      <title>{title}</title>
      <desc>{points.map((point) => `${formatDate(point.date, locale)}: ${metric === "revenue" ? formatPrice(point.revenue, locale) : point.bookings}`).join("; ")}</desc>
      {children(activeIndex)}
    </svg>
    <span className="sr-only" id={`${id}-instructions`}>{t("dashboard.chartInspectionInstructions")}</span>
    {activePoint && geometry && <div className={`exec-chart-tooltip${edge}${vertical}`} data-testid={`${metric}-inspection`} id={`${id}-inspection`} role="status" style={{ left: `${(geometry.x / chartWidth) * 100}%`, top: `${(geometry.y / 278) * 100}%` }}>
      <span>{t("dashboard.tooltipDate")}</span><time dateTime={activePoint.date}>{formatDate(activePoint.date, locale)}</time>
      <span>{t(metric === "revenue" ? "dashboard.tooltipRevenue" : "dashboard.tooltipBookings")}</span><strong>{formattedValue}</strong>
    </div>}
  </div>;
};

export function RevenueChart({ points, title }: { points: DashboardData["trends"]; title: string }) {
  const id = useId().replaceAll(":", "");
  const { locale } = useLanguage();
  const number = new Intl.NumberFormat(locale, { notation: "compact", maximumFractionDigits: 1 });
  const { coords, line, maximum } = chartGeometry(points, "revenue");
  const area = coords.length ? `${line} L${coords.at(-1)!.x},224 L${coords[0].x},224 Z` : "";
  const latest = points.at(-1);

  return <ChartInspector className="exec-revenue-chart" metric="revenue" points={points} title={title}>{(activeIndex) => <>
    <defs><linearGradient id={id} x1="0" y1="0" x2="0" y2="1"><stop stopColor="#FFD400" stopOpacity=".28" /><stop offset="1" stopColor="#FFD400" stopOpacity="0" /></linearGradient></defs>
    {[0, 0.25, 0.5, 0.75, 1].map((fraction) => <g key={fraction} className="exec-grid-line"><line x1="58" x2="650" y1={224 - fraction * 176} y2={224 - fraction * 176} /><text x="0" y={228 - fraction * 176}>{number.format(maximum * fraction)}</text></g>)}
    {area && <path data-chart-area className="exec-chart-area" d={area} fill={`url(#${id})`} />}
    {line && <path data-chart-line d={line} fill="none" stroke="#FFD400" strokeWidth="2.6" strokeLinejoin="round" vectorEffect="non-scaling-stroke" />}
    {coords.map((point, index) => <circle key={points[index].date} className={`exec-data-point${activeIndex === index ? " is-active" : ""}`} cx={point.x} cy={point.y} r={activeIndex === index ? 4.5 : 2} />)}
    {activeIndex !== null && coords[activeIndex] && <g className="exec-inspection-mark" aria-hidden="true"><line x1={coords[activeIndex].x} x2={coords[activeIndex].x} y1="40" y2="224" /><circle cx={coords[activeIndex].x} cy={coords[activeIndex].y} r="10" /></g>}
    {coords.length > 0 && <g className="exec-latest-point" transform={`translate(${coords.at(-1)!.x} ${coords.at(-1)!.y})`}><line x1="0" x2="0" y1="-14" y2="14" /><circle r="10" /><circle r="4" /><text x="-9" y={Math.max(-16, 30 - coords.at(-1)!.y)} textAnchor="end">{latest ? number.format(latest.revenue) : ""}</text></g>}
    {points.length > 0 && <><text className="exec-axis-date" x="58" y="264">{formatDate(points[0].date, locale)}</text>{points.length > 1 && <text className="exec-axis-date" x="650" y="264" textAnchor="end">{formatDate(points.at(-1)!.date, locale)}</text>}</>}
  </>}</ChartInspector>;
}

export function DemandChart({ points, title }: { points: DashboardData["trends"]; title: string }) {
  const { locale } = useLanguage();
  const { coords, line, maximum } = chartGeometry(points, "bookings");
  const width = points.length ? Math.max(3, Math.min(15, 470 / points.length)) : 0;

  return <ChartInspector className="exec-demand-chart" metric="bookings" points={points} title={title}>{(activeIndex) => <>
    {[0, 0.5, 1].map((fraction) => <g key={fraction} className="exec-grid-line"><line x1="58" x2="650" y1={224 - fraction * 176} y2={224 - fraction * 176} /><text x="0" y={228 - fraction * 176}>{new Intl.NumberFormat(locale).format(maximum * fraction)}</text></g>)}
    {coords.map((point, index) => <g key={points[index].date} data-demand-pulse className={`exec-demand-pulse${activeIndex === index ? " is-active" : ""}`}><line x1={point.x} x2={point.x} y1={point.y} y2="224" strokeWidth={width} /><circle cx={point.x} cy={point.y} r={activeIndex === index ? 4.5 : 2.7} /></g>)}
    {line && <path data-chart-line className="exec-demand-envelope" d={line} fill="none" />}
    {activeIndex !== null && coords[activeIndex] && <g className="exec-inspection-mark" aria-hidden="true"><line x1={coords[activeIndex].x} x2={coords[activeIndex].x} y1="40" y2="224" /><circle cx={coords[activeIndex].x} cy={coords[activeIndex].y} r="10" /></g>}
    {points.length > 0 && <><text className="exec-axis-date" x="58" y="264">{formatDate(points[0].date, locale)}</text>{points.length > 1 && <text className="exec-axis-date" x="650" y="264" textAnchor="end">{formatDate(points.at(-1)!.date, locale)}</text>}</>}
  </>}</ChartInspector>;
}

export function CabinMix({ cabins }: { cabins: DashboardData["cabins"] }) {
  const { t, locale } = useLanguage();
  const values = dashboardCabins.map((cabin) => cabins.find((item) => item.cabin === cabin)?.bookings ?? 0);
  const total = values.reduce((sum, value) => sum + value, 0);
  const number = new Intl.NumberFormat(locale, { maximumFractionDigits: 1 });

  return <div className="exec-cabin-mix">
    <div className="exec-ring-wrap">
      <svg viewBox="0 0 240 240" role="img" aria-label={t("dashboard.cabinChart")} data-values={values.join(",")}>
        <title>{t("dashboard.cabinChart")}</title>
        <g className="exec-ring-ticks" aria-hidden="true">{Array.from({ length: 24 }, (_, index) => <line key={index} x1="120" y1="10" x2="120" y2="17" transform={`rotate(${index * 15} 120 120)`} />)}</g>
        <circle cx="120" cy="120" r="88" fill="none" stroke="currentColor" opacity=".1" strokeWidth="1" />
        {values.map((value, index) => {
          const share = total ? (value / total) * 100 : 0;
          const offset = total ? (values.slice(0, index).reduce((sum, item) => sum + item, 0) / total) * 100 : 0;
          const radius = 78 - index * 10;
          return share > 0 && <circle data-cabin-segment key={dashboardCabins[index]} cx="120" cy="120" r={radius} fill="none" stroke={cabinColors[index]} strokeWidth="7" pathLength="100" strokeDasharray={`${Math.max(share - 1.2, 0)} ${101.2 - share}`} strokeDashoffset={-offset} strokeLinecap="square" transform="rotate(-90 120 120)" />;
        })}
        <path className="exec-ring-pointer" d="M120 24v19" />
      </svg>
      <div className="exec-ring-center"><span>{t("dashboard.cabinTotal")}</span><strong>{number.format(total)}</strong><small>{t("dashboard.bookingsShort")}</small></div>
    </div>
    <ol className="exec-cabin-legend">{dashboardCabins.map((cabin: DashboardCabin, index) => <li key={cabin}>
      <span className="exec-cabin-index">0{index + 1}</span><span className="exec-cabin-key" style={{ borderColor: cabinColors[index] }} aria-hidden="true" /><span>{t(`dashboard.${cabinKeys[cabin]}`)}</span><strong>{number.format(total ? (values[index] / total) * 100 : 0)}%</strong><small>{number.format(values[index])} {t("dashboard.bookingsShort")}</small>
    </li>)}</ol>
  </div>;
}

export function DailyFigures({ points }: { points: DashboardData["trends"] }) {
  const { t, locale } = useLanguage();
  return <details className="exec-daily"><summary>{t("dashboard.chartTable")}</summary><div className="exec-table-scroll"><table><caption className="sr-only">{t("dashboard.daily")}</caption><thead><tr><th scope="col">{t("dashboard.date")}</th><th scope="col">{t("dashboard.bookingsShort")}</th><th scope="col">{t("dashboard.revenueShort")}</th></tr></thead><tbody>{points.map((point) => <tr key={point.date}><th scope="row">{formatDate(point.date, locale)}</th><td>{new Intl.NumberFormat(locale).format(point.bookings)}</td><td>{formatPrice(point.revenue, locale)}</td></tr>)}</tbody></table></div></details>;
}
