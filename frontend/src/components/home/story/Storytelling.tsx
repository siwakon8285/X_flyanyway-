import { AircraftNarrativeStory } from "@/components/home/story/AircraftNarrativeStory";
import { CabinStory } from "@/components/home/story/CabinStory";
import { EditorialServiceStory } from "@/components/home/story/EditorialServiceStory";
import { GlobalReachStory } from "@/components/home/story/GlobalReachStory";
import { JourneyExperienceStory } from "@/components/home/story/JourneyExperienceStory";
import { StorytellingMotion } from "@/components/home/story/StorytellingMotion";

const Storytelling = () => (
  <StorytellingMotion>
    <GlobalReachStory />
    <AircraftNarrativeStory />
    <CabinStory />
    <EditorialServiceStory />
    <JourneyExperienceStory />
  </StorytellingMotion>
);

export { Storytelling };
