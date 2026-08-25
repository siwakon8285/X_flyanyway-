import Image from "next/image";

const heroImage = "/images/hero/x-fly-aircraft-concept-v2.jpg";

const HeroMedia = () => (
  <div
    aria-hidden="true"
    className="absolute inset-0 overflow-hidden bg-surface"
    data-hero-media
  >
    <div className="absolute -inset-y-3 inset-x-0" data-hero-media-frame>
      <Image
        alt=""
        className="object-cover object-[72%_center] sm:object-[68%_center] lg:object-center"
        fill
        preload
        sizes="100vw"
        src={heroImage}
      />
    </div>
    <div className="absolute inset-0 bg-gradient-to-r from-black/72 via-black/32 to-transparent" />
    <div className="absolute inset-0 bg-gradient-to-t from-black/68 via-black/10 to-transparent" />
    <div className="absolute inset-0 bg-gradient-to-b from-black/32 via-transparent to-transparent" />
  </div>
);

export { HeroMedia, heroImage };
