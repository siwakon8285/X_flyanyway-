import { FlightSearchSection } from "@/components/booking/search/FlightSearchSection";
import { HeroContent } from "@/components/home/HeroContent";
import { HeroMedia } from "@/components/home/HeroMedia";
import { HeroMotion } from "@/components/home/HeroMotion";
import { Container } from "@/components/layout/Container";

const Hero = () => (
  <HeroMotion>
    <div className="relative min-h-svh overflow-hidden" data-hero-visual>
      <HeroMedia />
      <HeroContent />
    </div>
    <Container
      className="relative z-20 -mt-16 pb-12 sm:-mt-20 sm:pb-16 lg:-mt-24 lg:pb-20"
      data-hero-search
      id="flight-search"
    >
      <div className="rounded-surface border border-border/80 bg-background/90 p-4 shadow-[0_22px_60px_rgb(0_0_0/0.36)] sm:p-6">
        <FlightSearchSection embedded />
      </div>
    </Container>
  </HeroMotion>
);

export { Hero };
