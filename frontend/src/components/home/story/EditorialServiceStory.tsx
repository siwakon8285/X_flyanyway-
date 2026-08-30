"use client";

import Image from "next/image";

import { Container } from "@/components/layout/Container";
import { useLanguage } from "@/i18n/LanguageProvider";

const interiorImage = "/images/hero/x-fly-interior-premium-v1.png";
const serviceImage = "/images/hero/x-fly-service-dining-v1.png";

const EditorialServiceStory = () => {
  const { t } = useLanguage();

  return (
  <section
    aria-labelledby="service-story-heading"
    className="relative isolate overflow-hidden bg-[#eee5d6] py-[clamp(11rem,14vw,16rem)] text-[#171611]"
    data-service-story
    id="service-story"
  >
    <div
      aria-hidden="true"
      className="absolute inset-x-0 top-0 h-48 bg-gradient-to-b from-[#080a0d] via-[#8b806f]/40 to-transparent"
      data-service-transition-entry
    />
    <div
      aria-hidden="true"
      className="pointer-events-none absolute inset-0 bg-[radial-gradient(ellipse_at_30%_42%,rgba(255,249,235,0.8),transparent_42%),radial-gradient(ellipse_at_78%_64%,rgba(211,185,116,0.11),transparent_34%)]"
      data-service-atmosphere
    />
    <div
      aria-hidden="true"
      className="absolute inset-x-0 bottom-0 h-48 bg-gradient-to-b from-transparent via-[#a89d8b]/15 to-[#080a0d]"
      data-service-transition-exit
    />

    <div className="relative" data-service-layout>
      <Container className="relative z-10">
        <div
          className="grid gap-8 border-t border-black/15 pt-7 md:grid-cols-12 md:items-end md:gap-8 lg:gap-12"
          data-service-heading
        >
          <div className="md:col-span-7 md:col-start-2 lg:col-span-6">
            <div
              className="flex items-center gap-4"
              data-service-eyebrow
            >
              <span
                aria-hidden="true"
                className="h-px w-10 origin-left bg-[#ad8b00]"
                data-service-eyebrow-line
              />
              <p className="text-label text-[#8d7100]">{t("home.service.label")}</p>
            </div>
            <h2
              className="mt-4 max-w-[12ch] text-h1 uppercase text-balance"
              id="service-story-heading"
            >
              <span className="block overflow-hidden">
                <span className="block" data-service-heading-line>
                  {t("home.service.headingFirst")}
                </span>
              </span>
              <span className="block overflow-hidden">
                <span className="block" data-service-heading-line>
                  {t("home.service.headingSecond")}
                </span>
              </span>
            </h2>
          </div>
          <p
            className="max-w-lg text-body-lg text-[#5f594f] md:col-span-4 md:col-start-8 md:justify-self-end"
            data-service-intro
          >
            {t("home.service.body")}
          </p>
        </div>

        <div className="mt-14 grid gap-12 md:mt-20 md:grid-cols-12 md:gap-8 lg:mt-24 lg:gap-12">
          <figure
            className="md:col-span-7 md:col-start-1 md:row-start-1 lg:col-start-2"
            data-service-panel="primary"
          >
            <div
              className="relative aspect-[5/4] overflow-hidden bg-[#d9cfbf] sm:aspect-[16/10]"
              data-service-image-frame="primary"
            >
              <div
                className="absolute inset-[-3%]"
                data-service-image-parallax="primary"
              >
                <Image
                  alt={t("home.service.cabinAlt")}
                  className="object-cover object-center"
                  data-service-image="primary"
                  fill
                  sizes="(min-width: 1536px) 54rem, (min-width: 768px) 58vw, 92vw"
                  src={interiorImage}
                />
              </div>
              <div className="absolute inset-0 bg-gradient-to-t from-black/18 via-transparent to-white/8" />
              <span className="absolute left-5 top-5 text-caption text-[#3c352a] sm:left-7 sm:top-7">
                {t("home.service.cabinLabel")}
              </span>
            </div>
            <figcaption
              className="relative mt-5 grid gap-3 pt-4 sm:grid-cols-[1fr_auto] sm:items-start sm:gap-8"
              data-service-caption
            >
              <span
                aria-hidden="true"
                className="absolute inset-x-0 top-0 h-px origin-left bg-gradient-to-r from-[#a88700] via-black/20 to-transparent"
                data-service-caption-rule
              />
              <span className="text-h3">{t("home.service.cabinHeading")}</span>
              <span className="max-w-60 text-body-sm text-[#655f55] sm:text-right">
                {t("home.service.cabinBody")}
              </span>
            </figcaption>
          </figure>

          <figure
            className="md:col-span-4 md:col-start-9 md:row-start-1 md:mt-24 lg:mt-36"
            data-service-panel="secondary"
          >
            <div
              className="relative aspect-[4/5] overflow-hidden bg-[#d9cfbf]"
              data-service-image-frame="secondary"
            >
              <div
                className="absolute inset-[-3%]"
                data-service-image-parallax="secondary"
              >
                <Image
                  alt={t("home.service.serviceAlt")}
                  className="object-cover object-[50%_42%]"
                  data-service-image="secondary"
                  fill
                  sizes="(min-width: 1536px) 28rem, (min-width: 768px) 32vw, 88vw"
                  src={serviceImage}
                />
              </div>
              <div className="absolute inset-0 bg-gradient-to-t from-black/25 via-transparent to-white/8" />
              <span className="absolute left-5 top-5 text-caption text-white/90 sm:left-7 sm:top-7">
                {t("home.service.serviceLabel")}
              </span>
            </div>
            <figcaption
              className="relative mt-5 pt-4"
              data-service-caption
            >
              <span
                aria-hidden="true"
                className="absolute inset-x-0 top-0 h-px origin-left bg-gradient-to-r from-[#a88700] via-black/20 to-transparent"
                data-service-caption-rule
              />
              <span className="text-h3">{t("home.service.serviceHeading")}</span>
              <p className="mt-3 max-w-sm text-body-sm text-[#655f55]">
                {t("home.service.serviceBody")}
              </p>
            </figcaption>
          </figure>
        </div>
      </Container>
    </div>
  </section>
  );
};

export { EditorialServiceStory, interiorImage, serviceImage };
