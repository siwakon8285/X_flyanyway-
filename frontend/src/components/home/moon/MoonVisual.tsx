import Image from "next/image";

/**
 * MoonVisual — Luminous photographic full-disc Moon with cinematic moonlight aura.
 *
 * Layer stack (bottom → top):
 *   1. Outer atmospheric bloom (wide, soft cool-blue light scattering into space)
 *   2. Mid moonlight halo (soft silver-white/cool-blue glow around the disc)
 *   3. Tight rim aura (crisp, delicate silver-white light hugging the lunar limb)
 *   4. Orbital trajectory SVG arc (behind the lunar disc)
 *   5. Hover-interactive container (1.04x scale with smooth 400ms transition)
 *      - Hover aura intensifier (subtly enhances moonlight on hover)
 *      - Masked and slightly enlarged photographic Moon (excludes JPEG background pixels)
 *      - Subtle front-illumination sheen (enhances crater highlights without darkening edges)
 */
const MoonVisual = () => (
  <div
    aria-hidden="true"
    className="relative aspect-square w-full max-w-[22rem] sm:max-w-[28rem] lg:-mr-16 lg:max-w-[36rem] xl:-mr-24 xl:max-w-[42rem]"
    data-moon-visual-container
  >
    {/* Layer 1: Outer Atmospheric Bloom (Wide, Soft Cool-Blue Spread) */}
    <div
      className="pointer-events-none absolute -inset-24 rounded-full opacity-75 blur-3xl"
      data-moon-aura="outer"
      style={{
        background:
          "radial-gradient(circle at 50% 50%, rgba(174, 207, 242, 0.18) 0%, rgba(124, 169, 221, 0.10) 44%, rgba(76, 122, 184, 0.035) 68%, transparent 82%)",
      }}
    />

    {/* Layer 2: Mid Moonlight Halo (Soft Cool-White / Silver-Blue Bloom) */}
    <div
      className="pointer-events-none absolute -inset-12 rounded-full opacity-90 blur-2xl"
      data-moon-aura="mid"
      data-moon-glow
      style={{
        background:
          "radial-gradient(circle at 50% 50%, rgba(244, 249, 255, 0.30) 0%, rgba(205, 227, 252, 0.18) 48%, rgba(151, 190, 235, 0.07) 66%, transparent 78%)",
      }}
    />

    {/* Layer 3: Tight Rim Aura (Crisp Silver-White Light hugging the Moon silhouette) */}
    <div
      className="pointer-events-none absolute -inset-3 rounded-full opacity-95 blur-md"
      data-moon-aura="rim"
      style={{
        background:
          "radial-gradient(circle at 50% 50%, rgba(255, 255, 255, 0.42) 0%, rgba(240, 248, 255, 0.34) 72%, rgba(210, 231, 255, 0.18) 82%, transparent 92%)",
      }}
    />

    {/* Layer 4: Orbital Trajectory SVG Arc */}
    <svg
      className="pointer-events-none absolute inset-[-20%] z-[1] h-[140%] w-[140%] select-none"
      data-moon-trajectory-svg
      fill="none"
      style={{ zIndex: 1 }}
      viewBox="0 0 500 500"
    >
      <defs>
        <filter id="moonTrajGlow" x="-20%" y="-20%" width="140%" height="140%">
          <feGaussianBlur in="SourceGraphic" stdDeviation="2.5" result="blur" />
          <feMerge>
            <feMergeNode in="blur" />
            <feMergeNode in="SourceGraphic" />
          </feMerge>
        </filter>
        <linearGradient id="moonTrajGrad" x1="0%" x2="100%" y1="100%" y2="0%">
          <stop offset="0%" stopColor="#FFD400" stopOpacity="0.15" />
          <stop offset="40%" stopColor="#FFD400" stopOpacity="0.8" />
          <stop offset="80%" stopColor="#FFFFFF" stopOpacity="0.9" />
          <stop offset="100%" stopColor="#FFD400" stopOpacity="0.7" />
        </linearGradient>
      </defs>
      <path
        d="M 20 455 C 120 370 230 240 330 160 C 390 115 435 85 480 72"
        data-moon-trajectory
        filter="url(#moonTrajGlow)"
        stroke="url(#moonTrajGrad)"
        strokeDasharray="5 3"
        strokeLinecap="round"
        strokeWidth="1.25"
      />
      <circle cx="20" cy="455" fill="#FFD400" opacity="0.8" r="3" />
      <circle cx="330" cy="160" fill="#FFD400" opacity="0.6" r="2" />
      <circle cx="480" cy="72" fill="#FFFFFF" filter="url(#moonTrajGlow)" r="3.5" />
    </svg>

    {/* Layer 5: Hover-Interactive Wrapper (Subtle 1.04x scale on hover) */}
    <div
      className="group relative z-10 h-full w-full transition-transform duration-[400ms] ease-out will-change-transform motion-safe:hover:scale-[1.04] motion-reduce:transition-none"
      data-moon-sphere
      style={{ zIndex: 10 }}
    >
      {/* Hover Moonlight Intensifier */}
      <div
        className="pointer-events-none absolute -inset-10 rounded-full opacity-0 blur-2xl transition-opacity duration-500 ease-out group-hover:opacity-100 motion-reduce:transition-none"
        style={{
          background:
            "radial-gradient(circle at 50% 50%, rgba(220, 240, 255, 0.25) 0%, rgba(170, 205, 250, 0.12) 46%, transparent 72%)",
        }}
      />

      {/* The 1.09 crop places the JPEG background outside this feathered circular mask. */}
      <div
        className="relative h-full w-full overflow-hidden rounded-full"
        data-moon-surface
        style={{
          WebkitMaskImage:
            "radial-gradient(circle at center, #000 98.9%, rgba(0, 0, 0, 0.96) 99.25%, rgba(0, 0, 0, 0.55) 99.65%, transparent 100%)",
          maskImage:
            "radial-gradient(circle at center, #000 98.9%, rgba(0, 0, 0, 0.96) 99.25%, rgba(0, 0, 0, 0.55) 99.65%, transparent 100%)",
        }}
      >
        <Image
          alt=""
          className="scale-[1.09] object-cover"
          draggable={false}
          fill
          priority
          sizes="(min-width: 1280px) 42rem, (min-width: 1024px) 36rem, (min-width: 640px) 28rem, 22rem"
          src="/images/hero/x-fly-moon-full-v1.jpg"
        />
      </div>

      {/* Layer 5b: Delicate Luminous Front Sheen (highlights crater relief without dark bands) */}
      <div
        aria-hidden="true"
        className="pointer-events-none absolute inset-0 rounded-full"
        style={{
          background:
            "radial-gradient(circle at 45% 42%, rgba(255, 255, 255, 0.07) 0%, rgba(235, 245, 255, 0.02) 48%, transparent 72%)",
        }}
      />
    </div>
  </div>
);

export { MoonVisual };
