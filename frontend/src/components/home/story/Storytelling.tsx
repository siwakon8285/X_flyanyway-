import { AircraftNarrativeStory } from "@/components/home/story/AircraftNarrativeStory";
import { CabinStory } from "@/components/home/story/CabinStory";
import { EditorialServiceStory } from "@/components/home/story/EditorialServiceStory";
import { FutureMoonStory } from "@/components/home/moon/FutureMoonStory";
import { GlobalReachStory } from "@/components/home/story/GlobalReachStory";
import { LayeredJourneyStory } from "@/components/home/story/LayeredJourneyStory";
import { JourneyExperienceStory } from "@/components/home/story/JourneyExperienceStory";
import { StorytellingMotion } from "@/components/home/story/StorytellingMotion";

const Storytelling = () => (
  <StorytellingMotion>
    <GlobalReachStory />
    <AircraftNarrativeStory />
    <CabinStory />
    <EditorialServiceStory />
    <LayeredJourneyStory />
    <FutureMoonStory />
    <JourneyExperienceStory />
  </StorytellingMotion>
);

export { Storytelling };
