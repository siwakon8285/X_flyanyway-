import { AIRPORT_FIXTURES } from "@/components/booking/search/airportFixtures";
import type { AirportOption } from "@/components/booking/search/searchTypes";
import { Hero } from "@/components/home/Hero";
import { HomePromoCarousel } from "@/components/home/HomePromoCarousel";
import { Storytelling } from "@/components/home/story/Storytelling";

const HomePage = ({
  airports = AIRPORT_FIXTURES,
}: {
  airports?: readonly AirportOption[];
}) => (
  <>
    <Hero airports={airports} />
    <HomePromoCarousel />
    <Storytelling />
  </>
);

export { HomePage };
