import re

with open("frontend/src/components/home/story/StorytellingMotion.tsx", "r") as f:
    content = f.read()

start_marker = r"useGSAP\(\s*\(\) => \{\s*const element = root\.current;\s*if \(!element \|\| reducedMotion\) return;\s*const layeredStory = element\.querySelector<HTMLElement>\(\"\[data-layered-story\]\"\);"
match = re.search(start_marker, content)
if not match:
    print("Could not find start marker")
    exit(1)

start_idx = match.start()
next_usegsap = content.find("useGSAP(", match.end())

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

        // DOM Order: cards[0] is deepest, cards[3] is foreground
        
        // Define positions
        const fgProps = { xPercent: 0, yPercent: 0, scale: 1, opacity: 1, zIndex: 20 };
        const bgProps = { xPercent: 25, yPercent: -15, scale: 0.95, opacity: 0.4, zIndex: 10 };
        const hiddenProps = { xPercent: 25, yPercent: -15, scale: 0.85, opacity: 0, zIndex: 5 };
        const leavingProps = { xPercent: -5, yPercent: 5, scale: 1.05, opacity: 0, zIndex: 30 };

        // Initial setup
        gsap.set(cards, hiddenProps);
        gsap.set(cards[3], fgProps); // Active (Card 1)
        gsap.set(cards[2], bgProps); // Background (Card 2)
        
        gsap.set(copies, { opacity: 0, y: 24 });
        gsap.set(copies[0], { opacity: 1, y: 0 });

        // Transition 1 (Chapter 1 -> 2)
        // Foreground Card 1 leaves
        tl.to(cards[3], { ...leavingProps, duration: 1 }, 0);
        // Copy 1 leaves
        tl.to(copies[0], { opacity: 0, y: -24, duration: 1 }, 0);
        
        // Background Card 2 promotes to Foreground
        tl.to(cards[2], { ...fgProps, duration: 1 }, 0.2);
        // Copy 2 enters
        tl.to(copies[1], { opacity: 1, y: 0, duration: 1 }, 0.4);
        // New Background Card 3 enters
        tl.to(cards[1], { ...bgProps, duration: 1 }, 0.4);

        // Transition 2 (Chapter 2 -> 3)
        // Foreground Card 2 leaves
        tl.to(cards[2], { ...leavingProps, duration: 1 }, 2.0);
        // Copy 2 leaves
        tl.to(copies[1], { opacity: 0, y: -24, duration: 1 }, 2.0);

        // Background Card 3 promotes to Foreground
        tl.to(cards[1], { ...fgProps, duration: 1 }, 2.2);
        // Copy 3 enters
        tl.to(copies[2], { opacity: 1, y: 0, duration: 1 }, 2.4);
        // New Background Card 4 enters
        tl.to(cards[0], { ...bgProps, duration: 1 }, 2.4);

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

print("Patched StorytellingMotion successfully")
