import re

with open("frontend/src/components/home/story/StorytellingMotion.tsx", "r") as f:
    content = f.read()

# We want to replace the `useGSAP` block that contains `data-journey-path`
# We can find it by looking for `useGSAP(\n    () => {\n      const element = root.current;\n      if (!element || reducedMotion) return;\n\n      const journeyStory`

start_marker = r"useGSAP\(\s*\(\) => \{\s*const element = root\.current;\s*if \(!element \|\| reducedMotion\) return;\s*const journeyStory = element\.querySelector<HTMLElement>\(\s*\"\[data-journey-path\]\",\s*\);"

# We want to replace everything from this start_marker until the next useGSAP
# or the end of the file.

match = re.search(start_marker, content)
if not match:
    print("Could not find start marker")
    exit(1)

start_idx = match.start()

# find the next useGSAP after this one
next_usegsap = content.find("useGSAP(", match.end())

if next_usegsap == -1:
    print("Could not find next useGSAP")
    exit(1)

new_block = """useGSAP(
    () => {
      const element = root.current;
      if (!element || reducedMotion) return;

      const layeredStory = element.querySelector<HTMLElement>("[data-layered-story]");
      if (!layeredStory) return;

      const viewport = layeredStory.querySelector<HTMLElement>("[data-layered-viewport]");
      const copies = gsap.utils.toArray<HTMLElement>("[data-layered-copy]", layeredStory);
      const cards = gsap.utils.toArray<HTMLElement>("[data-layered-card]", layeredStory);

      if (!viewport || copies.length !== 3 || cards.length !== 4) return;

      const mediaQueries = gsap.matchMedia();

      mediaQueries.add(motionMediaQueries.desktop, () => {
        // Desktop pinning and layered crossfade
        const tl = gsap.timeline({
          defaults: { ease: "none" },
          scrollTrigger: {
            trigger: layeredStory,
            start: "top top",
            end: "+=180%",
            pin: true,
            scrub: 0.6,
            invalidateOnRefresh: true,
          }
        });

        // Setup DOM order (4, 3, 2, 1 where 1 is the last child and visually on top)
        // Set initial states
        gsap.set(cards, { opacity: 0, scale: 0.8, y: 32 });
        gsap.set(cards[3], { opacity: 1, scale: 1, y: 0 }); // Active
        gsap.set(cards[2], { opacity: 0.4, scale: 0.9, y: 16 }); // Upcoming
        
        gsap.set(copies, { opacity: 0, y: 24 });
        gsap.set(copies[0], { opacity: 1, y: 0 });

        // Transition 1 (Chapter 1 -> 2)
        tl.to(copies[0], { opacity: 0, y: -24, duration: 1 }, 0);
        tl.to(cards[3], { opacity: 0, scale: 1.1, duration: 1 }, 0);
        
        tl.to(copies[1], { opacity: 1, y: 0, duration: 1 }, 0.5);
        tl.to(cards[2], { opacity: 1, scale: 1, y: 0, duration: 1 }, 0.5);
        tl.to(cards[1], { opacity: 0.4, scale: 0.9, y: 16, duration: 1 }, 0.5);

        // Transition 2 (Chapter 2 -> 3)
        tl.to(copies[1], { opacity: 0, y: -24, duration: 1 }, 2);
        tl.to(cards[2], { opacity: 0, scale: 1.1, duration: 1 }, 2);

        tl.to(copies[2], { opacity: 1, y: 0, duration: 1 }, 2.5);
        tl.to(cards[1], { opacity: 1, scale: 1, y: 0, duration: 1 }, 2.5);
        tl.to(cards[0], { opacity: 0.4, scale: 0.9, y: 16, duration: 1 }, 2.5);

        // Padding at the end
        tl.to({}, { duration: 0.5 });
        
        return undefined;
      });

      return () => mediaQueries.revert();
    },
    {
      dependencies: [reducedMotion],
      revertOnUpdate: true,
      scope: root,
    },
  );

  """

new_content = content[:start_idx] + new_block + content[next_usegsap:]

with open("frontend/src/components/home/story/StorytellingMotion.tsx", "w") as f:
    f.write(new_content)

print("Replaced successfully")
