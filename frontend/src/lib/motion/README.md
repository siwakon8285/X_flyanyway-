# Motion ownership

- GSAP owns timelines, reveals, counters, parallax, pinning, and layout Flip.
- ScrollTrigger instances must be created inside a scoped `useGSAP()` context.
- Lenis is instantiated only by `SmoothScrollProvider` and uses the GSAP ticker.
- Motion owns small component-level enter/exit behavior only.
- SplitType must be created after hydration and reverted during cleanup.
- Never let GSAP and Motion animate the same property on the same element.
- Reduced-motion content starts visible; smooth scroll, parallax, pinning, and
  positional choreography must become static.
