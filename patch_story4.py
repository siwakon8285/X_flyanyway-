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
      
      const leftImgs = gsap.utils.toArray<HTMLElement>("[data-layered-left-img]", layeredStory);
      const rightImgs = gsap.utils.toArray<HTMLElement>("[data-layered-right-img]", layeredStory);

      // We expect 3 text states, 3 left images, 3 right images
      if (!viewport || copies.length !== 3 || leftImgs.length !== 3 || rightImgs.length !== 3) return;

      const mediaQueries = gsap.matchMedia();

      mediaQueries.add(motionMediaQueries.desktop, () => {
        const tl = gsap.timeline({
          defaults: { ease: "none" },
          scrollTrigger: {
            trigger: layeredStory,
            start: "top top",
            end: "+=150%", // reduced duration since we transition pairs together
            pin: true,
            scrub: 0.6,
            invalidateOnRefresh: true,
          }
        });

        // DOM Order: index 0 = state 3 (top in DOM but deepest z), index 2 = state 1 (bottom in DOM, top z)
        const l3 = leftImgs[0];
        const l2 = leftImgs[1];
        const l1 = leftImgs[2];

        const r3 = rightImgs[0];
        const r2 = rightImgs[1];
        const r1 = rightImgs[2];

        // Initial setup
        gsap.set([l3, l2, r3, r2], { opacity: 0, scale: 1.05 });
        gsap.set([l1, r1], { opacity: 1, scale: 1 });
        
        gsap.set(copies, { opacity: 0, y: 16 });
        gsap.set(copies[0], { opacity: 1, y: 0 });

        // State Transitions
        
        // --- Transition 1: BEGIN -> MOVE ---
        // Left & Right fade out
        tl.to([l1, r1], { opacity: 0, duration: 1 }, 0);
        // Left & Right fade in
        tl.to([l2, r2], { opacity: 1, scale: 1, duration: 1 }, 0);
        
        // Copy: Begin -> Move
        tl.to(copies[0], { opacity: 0, y: -16, duration: 0.8 }, 0.1);
        tl.to(copies[1], { opacity: 1, y: 0, duration: 0.8 }, 0.3);

        // --- Transition 2: MOVE -> ARRIVE ---
        // Left & Right fade out
        tl.to([l2, r2], { opacity: 0, duration: 1 }, 1.5);
        // Left & Right fade in
        tl.to([l3, r3], { opacity: 1, scale: 1, duration: 1 }, 1.5);
        
        // Copy: Move -> Arrive
        tl.to(copies[1], { opacity: 0, y: -16, duration: 0.8 }, 1.6);
        tl.to(copies[2], { opacity: 1, y: 0, duration: 0.8 }, 1.8);

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
