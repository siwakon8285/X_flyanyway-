import { AircraftNarrativeStory } from "@/components/home/story/AircraftNarrativeStory";
import { CabinStory } from "@/components/home/story/CabinStory";
import { EditorialServiceStory } from "@/components/home/story/EditorialServiceStory";
import { LayeredJourneyStory } from "@/components/home/story/LayeredJourneyStory";
import { JourneyExperienceStory } from "@/components/home/story/JourneyExperienceStory";
import { StorytellingMotion } from "@/components/home/story/StorytellingMotion";

const Storytelling = () => (
  <StorytellingMotion>
    <AircraftNarrativeStory />
    <CabinStory />
    <EditorialServiceStory />
    <LayeredJourneyStory />
    <JourneyExperienceStory />
  </StorytellingMotion>
);

export { Storytelling };
