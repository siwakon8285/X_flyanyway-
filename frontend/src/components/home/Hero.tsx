import { HeroContent } from "@/components/home/HeroContent";
import { HeroMedia } from "@/components/home/HeroMedia";
import { HeroMotion } from "@/components/home/HeroMotion";

const Hero = () => (
  <HeroMotion>
    <HeroMedia />
    <HeroContent />
  </HeroMotion>
);

export { Hero };
