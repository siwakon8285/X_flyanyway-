import { render, screen } from "@testing-library/react";

import FlightDetailLoading from "@/app/flights/[flightId]/loading";
import SeatMapLoading from "@/app/flights/[flightId]/seats/loading";
import FlightResultsLoading from "@/app/flights/loading";
import Home from "@/app/page";
import { FlightDetailPage } from "@/components/booking/detail/FlightDetailPage";
import { resolveFlightDetailRequest } from "@/components/booking/detail/flightDetailUtils";
import { FlightResultsPage } from "@/components/booking/results/FlightResultsPage";
import { SeatMapPage } from "@/components/booking/seats/SeatMapPage";
import { getSeatMapFixture } from "@/components/booking/seats/seatMapFixtures";
import { Header } from "@/components/layout/Header";
import { Footer } from "@/components/layout/Footer";
import { LanguageProvider } from "@/i18n/LanguageProvider";

const query = {
  adults: "2",
  cabin: "business",
  children: "1",
  departure: "2099-05-10",
  from: "BKK",
  infants: "1",
  return: "2099-05-18",
  selectedCabin: "business",
  to: "LHR",
  trip: "round-trip",
} as const;

const renderThai = (ui: React.ReactNode) =>
  render(<LanguageProvider initialLocale="th">{ui}</LanguageProvider>);

describe("translated customer screens", () => {
  it("translates the shell and current homepage while preserving the brand", () => {
    renderThai(
      <>
        <Header />
        <Home />
        <Footer />
      </>,
    );

    expect(screen.getAllByText("X-FLY ANYWAY").length).toBeGreaterThan(0);
    expect(screen.getByRole("button", { name: /ภาษาปัจจุบัน/ })).toHaveTextContent(
      "TH",
    );
    expect(screen.getAllByRole("link", { name: "จองเที่ยวบิน" }).length).toBeGreaterThan(0);
    expect(
      screen.getByRole("heading", { name: "ไปได้ทุกที่ บินในแบบที่แตกต่าง" }),
    ).toBeInTheDocument();
    expect(screen.getByText("เครือข่ายทั่วโลก")).toBeInTheDocument();
    expect(screen.getByText("ค้นหาเที่ยวบิน · เส้นทางบนโลก")).toBeInTheDocument();
    expect(screen.getByText("ออกแบบเพื่อพาคุณไปได้ทุกที่")).toBeInTheDocument();
  });

  it("translates Results and keeps route identifiers intact", () => {
    const request = resolveFlightDetailRequest("xf-201", query);
    if (!request) throw new Error("Expected fixture request");

    renderThai(
      <FlightResultsPage criteria={request.criteria} query={request.query} />,
    );

    expect(screen.getByText("เลือกเที่ยวบินขาออก")).toBeInTheDocument();
    expect(
      screen.getByRole("heading", { level: 1, name: /BKK.*LHR/ }),
    ).toBeInTheDocument();
    expect(screen.getAllByText("XF 201").length).toBeGreaterThan(0);
    expect(screen.getAllByText("ชั้นธุรกิจ").length).toBeGreaterThan(0);
  });

  it("translates Flight Detail while retaining cabin and query state", () => {
    const request = resolveFlightDetailRequest("xf-201", query);
    if (!request) throw new Error("Expected fixture request");

    renderThai(<FlightDetailPage {...request} />);

    expect(screen.getByText("กลับไปยังเที่ยวบิน")).toBeInTheDocument();
    expect(screen.getByText("เลือกประสบการณ์การบินของคุณ")).toBeInTheDocument();
    expect(screen.getAllByText("ชั้นธุรกิจ").length).toBeGreaterThan(0);
    expect(screen.getByText("XF 201")).toBeInTheDocument();
  });

  it("translates Seat Map while preserving seat identifiers", () => {
    const request = resolveFlightDetailRequest("xf-201", query);
    if (!request) throw new Error("Expected fixture request");
    const seatMap = getSeatMapFixture(request.flight.aircraft, "business");
    if (!seatMap) throw new Error("Expected seat map fixture");

    renderThai(
      <SeatMapPage
        request={{ ...request, selectedCabin: "business" }}
        seatMap={seatMap}
      />,
    );

    expect(screen.getByRole("heading", { name: "เลือกที่นั่งของคุณ" })).toBeInTheDocument();
    expect(screen.getByText("ที่นั่งของคุณ")).toBeInTheDocument();
    expect(screen.getAllByRole("button", { name: /ที่นั่ง \d+[A-Z]/ }).length).toBeGreaterThan(0);
  });

  it("translates shared route loading states", () => {
    const { unmount } = renderThai(<FlightResultsLoading />);
    expect(screen.getByRole("status", { name: "กำลังโหลดผลการค้นหาเที่ยวบิน" })).toBeInTheDocument();
    unmount();

    const detail = renderThai(<FlightDetailLoading />);
    expect(screen.getByRole("status", { name: "กำลังโหลดรายละเอียดเที่ยวบิน" })).toBeInTheDocument();
    detail.unmount();

    renderThai(<SeatMapLoading />);
    expect(screen.getByRole("status", { name: "กำลังโหลดผังที่นั่ง" })).toBeInTheDocument();
  });
});
