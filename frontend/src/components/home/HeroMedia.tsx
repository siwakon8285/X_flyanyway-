import Image from "next/image";

const heroImage = "/images/hero/x-fly-aircraft-hero-sharp-v1.png";

const HeroMedia = () => (
  <div
    aria-hidden="true"
    className="absolute inset-0 overflow-hidden bg-surface"
    data-hero-media
  >
    <div
      className="absolute -inset-y-3 inset-x-0"
      data-hero-media-frame
    >
      <Image
        alt=""
        className="object-cover object-[73%_center] sm:object-[69%_center] lg:object-center"
        fill
        preload
        sizes="100vw"
        src={heroImage}
        unoptimized
      />
    </div>
    <div className="absolute inset-0 bg-gradient-to-r from-black/64 via-black/24 to-transparent" />
    <div className="absolute inset-0 bg-gradient-to-t from-black/58 via-black/5 to-transparent" />
    <div className="absolute inset-0 bg-gradient-to-b from-black/28 via-transparent to-transparent" />
  </div>
);

export { HeroMedia, heroImage };
