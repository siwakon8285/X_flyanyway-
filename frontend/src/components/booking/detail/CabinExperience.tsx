"use client";

import { AnimatePresence, motion } from "motion/react";
import { ArrowRight, Check } from "lucide-react";
import Image from "next/image";
import Link from "next/link";
import { useState } from "react";

import {
  CABIN_PRESENTATION_BY_ID,
  CABIN_PRESENTATIONS,
} from "@/components/booking/cabin/cabinPresentation";
import { buildSeatSelectionHref } from "@/components/booking/detail/flightDetailUtils";
import { getCabinPrice } from "@/components/booking/results/flightResultUtils";
import type { FlightResult } from "@/components/booking/results/flightResultTypes";
import type { CabinClass } from "@/components/booking/search/searchTypes";
import { Button, buttonVariants } from "@/components/ui/Button";
import { Tabs, TabsContent, TabsList, TabsTrigger } from "@/components/ui/Tabs";
import { motionDurations } from "@/lib/motion/durations";
import { motionEasings } from "@/lib/motion/easing";
import { useReducedMotion } from "@/lib/motion/reducedMotion";
import { cn } from "@/lib/utils/cn";
import { useLanguage } from "@/i18n/LanguageProvider";
import { formatPrice } from "@/i18n/formatters";

const isCabinClass = (value: string): value is CabinClass =>
  CABIN_PRESENTATIONS.some((cabin) => cabin.id === value);

const CabinExperience = ({
  flight,
  initialCabin,
  query,
  searchedCabin,
}: {
  flight: FlightResult;
  initialCabin: CabinClass;
  query: string;
  searchedCabin: CabinClass;
}) => {
  const [activeCabin, setActiveCabin] = useState<CabinClass>(initialCabin);
  const reducedMotion = useReducedMotion();
  const { locale, t } = useLanguage();
  const cabin = CABIN_PRESENTATION_BY_ID[activeCabin];
  const price = getCabinPrice(flight, activeCabin);
  const seatHref = buildSeatSelectionHref({
    flightId: flight.id,
    query,
    selectedCabin: activeCabin,
  });
  const contentItemVariants = {
    hidden: { opacity: 0, y: reducedMotion ? 0 : 8 },
    visible: {
      opacity: 1,
      transition: {
        duration: reducedMotion ? 0 : motionDurations.micro,
        ease: motionEasings.enter,
      },
      y: 0,
    },
  };

  return (
    <section
      aria-label={t("flightDetail.cabin.experienceLabel")}
      className="relative isolate overflow-hidden py-section-md"
      id="cabin-experience"
    >
      <AnimatePresence initial={false}>
        <motion.div
          animate={{ opacity: 1 }}
          aria-hidden="true"
          className={cn("pointer-events-none absolute inset-0 -z-10", cabin.glowClass)}
          exit={{ opacity: reducedMotion ? 1 : 0 }}
          initial={reducedMotion ? false : { opacity: 0 }}
          key={activeCabin}
          transition={{
            duration: reducedMotion ? 0 : motionDurations.ui,
            ease: motionEasings.enter,
          }}
        />
      </AnimatePresence>
      <div className="max-w-3xl">
        <p className="text-label text-brand">{t("flightDetail.cabin.label")}</p>
        <h2 className="mt-3 text-h2">{t("flightDetail.cabin.heading")}</h2>
        <p className="mt-4 text-body-lg text-muted-foreground">
          {t("flightDetail.cabin.description")}
        </p>
      </div>

      <Tabs
        className="mt-8"
        onValueChange={(value) => {
          if (isCabinClass(value)) setActiveCabin(value);
        }}
        value={activeCabin}
      >
        <TabsList
          aria-label={t("flightSearch.cabinClass")}
          className="grid h-auto w-full grid-cols-2 gap-2 bg-transparent p-0 sm:grid-cols-4"
        >
          {CABIN_PRESENTATIONS.map((option) => {
            const isSearchedCabin = option.id === searchedCabin;
            const isAvailable = getCabinPrice(flight, option.id) !== null;
            const stateLabel = isSearchedCabin
              ? t("flightDetail.cabin.searched")
              : isAvailable
                ? ""
                : t("flightDetail.cabin.unavailableLower");
            const label = t(option.labelKey);

            return (
              <TabsTrigger
                aria-label={`${label}${stateLabel ? `, ${stateLabel}` : ""}`}
                className={cn(
                  "min-h-14 min-w-0 flex-col gap-1 border border-border bg-surface/65 px-3 transition-[border-color,background-color,box-shadow,color,transform] duration-200 hover:bg-surface-elevated motion-safe:hover:-translate-y-0.5 motion-safe:hover:scale-[1.01] motion-safe:active:translate-y-0 motion-safe:active:scale-[0.985] data-[state=active]:border-current data-[state=active]:bg-surface-elevated motion-reduce:transition-none",
                  option.accentClass,
                  option.tabInteractionClass,
                )}
                key={option.id}
                value={option.id}
              >
                <span className="text-sm text-foreground sm:text-base">
                  <span className="sm:hidden">{t(option.shortLabelKey)}</span>
                  <span className="hidden sm:inline">{label}</span>
                </span>
                {isSearchedCabin ? (
                  <span className="text-[0.6rem] font-semibold uppercase tracking-[0.12em] text-muted-foreground">
                    {t("flightDetail.cabin.yourSearch")}
                  </span>
                ) : !isAvailable ? (
                  <span className="text-[0.6rem] font-semibold uppercase tracking-[0.12em] text-muted-foreground">
                    {t("flightDetail.cabin.unavailable")}
                  </span>
                ) : null}
              </TabsTrigger>
            );
          })}
        </TabsList>

        <TabsContent className="mt-8" value={activeCabin}>
          <AnimatePresence initial={false} mode="wait">
            <motion.div
              animate={{ opacity: 1, scale: 1, y: 0 }}
              className={cn(
                "grid overflow-hidden rounded-surface border lg:grid-cols-[minmax(19rem,0.75fr)_minmax(0,1.25fr)]",
                cabin.surfaceClass,
              )}
              exit={
                reducedMotion ? undefined : { opacity: 0, scale: 0.995, y: -6 }
              }
              initial={
                reducedMotion ? false : { opacity: 0, scale: 1.01, y: 8 }
              }
              key={activeCabin}
              transition={{
                duration: reducedMotion ? 0 : motionDurations.ui,
                ease: motionEasings.enter,
              }}
            >
              <div className="flex min-w-0 flex-col justify-between p-6 sm:p-8 lg:p-10">
                <motion.div
                  animate="visible"
                  initial={reducedMotion ? false : "hidden"}
                  variants={{
                    hidden: {},
                    visible: {
                      transition: reducedMotion
                        ? { duration: 0 }
                        : { delayChildren: 0.04, staggerChildren: 0.045 },
                    },
                  }}
                >
                  <motion.p
                    className={cn("text-caption", cabin.accentClass)}
                    variants={contentItemVariants}
                  >
                    {t(cabin.labelKey)}
                  </motion.p>
                  <motion.h3
                    className="mt-3 text-h2"
                    variants={contentItemVariants}
                  >
                    {t(cabin.labelKey)}
                  </motion.h3>
                  <motion.p
                    className="mt-4 max-w-md text-body text-muted-foreground"
                    variants={contentItemVariants}
                  >
                    {t(cabin.descriptionKey)}
                  </motion.p>
                  <ul className="mt-7 space-y-3">
                    {cabin.featureKeys.map((featureKey) => (
                      <motion.li
                        className="group/feature flex items-center gap-3 border-l border-transparent text-body-sm transition-[border-color,color,transform] duration-200 hover:border-current motion-safe:hover:translate-x-1 motion-reduce:transition-none"
                        key={featureKey}
                        variants={contentItemVariants}
                      >
                        <Check
                          aria-hidden="true"
                          className={cn(
                            "size-4 transition-[color,filter] duration-200 group-hover/feature:drop-shadow-[0_0_6px_currentColor] motion-reduce:transition-none",
                            cabin.accentClass,
                          )}
                        />
                        <span>{t(featureKey)}</span>
                      </motion.li>
                    ))}
                  </ul>
                </motion.div>

                <motion.div
                  animate={{ opacity: 1 }}
                  className="mt-10 border-t border-current/15 pt-6"
                  initial={reducedMotion ? false : { opacity: 0 }}
                  transition={{
                    delay: reducedMotion ? 0 : 0.12,
                    duration: reducedMotion ? 0 : motionDurations.micro,
                  }}
                >
                  {price === null ? (
                    <>
                      <p className="text-lg font-semibold">{t("flightDetail.cabin.notAvailable")}</p>
                      <p className="mt-2 text-body-sm text-muted-foreground">
                        {t("flightDetail.cabin.previewAnother")}
                      </p>
                      <Button
                        aria-label={t("flightDetail.chooseSeatUnavailableAria", { cabin: t(cabin.labelKey), flight: flight.flightNumber })}
                        className="mt-6 w-full sm:w-auto"
                        disabled
                        size="lg"
                      >
                        {t("flightDetail.cabin.chooseSeat")}
                        <ArrowRight aria-hidden="true" />
                      </Button>
                    </>
                  ) : (
                    <>
                      <p className="text-2xl font-semibold tracking-[-0.035em]">
                        {formatPrice(price, locale)}
                      </p>
                      <p className="mt-1 text-xs text-muted-foreground">
                        {t("common.farePerPassenger")}
                      </p>
                      <Link
                        aria-label={t("flightDetail.chooseSeatAria", { cabin: t(cabin.labelKey), flight: flight.flightNumber })}
                        className={cn(
                          buttonVariants({ size: "lg" }),
                          "mt-6 w-full transition-[background-color,box-shadow,transform] duration-200 hover:shadow-[0_12px_30px_rgb(255_212_0/0.22)] motion-safe:hover:-translate-y-0.5 motion-safe:hover:scale-[1.02] motion-safe:active:translate-y-0 motion-safe:active:scale-[0.985] sm:w-auto motion-reduce:transition-none [&_svg]:transition-transform [&_svg]:duration-200 motion-safe:hover:[&_svg]:translate-x-1.5",
                        )}
                        href={seatHref}
                      >
                        {t("flightDetail.cabin.chooseSeat")}
                        <ArrowRight aria-hidden="true" />
                      </Link>
                    </>
                  )}
                </motion.div>
              </div>

              <motion.div
                animate={{ opacity: 1, scale: 1 }}
                className="group/image relative order-first aspect-[16/10] min-h-72 overflow-hidden bg-background lg:order-last lg:aspect-auto lg:min-h-[38rem]"
                initial={reducedMotion ? false : { opacity: 0, scale: 1.015 }}
                transition={{
                  duration: reducedMotion ? 0 : 0.45,
                  ease: motionEasings.enter,
                }}
              >
                <Image
                  alt={t(cabin.altKey)}
                  className={cn(
                    "object-cover transition-transform duration-[650ms] ease-[cubic-bezier(0.4,0,0.2,1)] motion-reduce:transition-none motion-safe:[@media(hover:hover)_and_(pointer:fine)]:group-hover/image:scale-[1.025] motion-safe:[@media(hover:hover)_and_(pointer:fine)]:group-hover/image:duration-[550ms] motion-safe:[@media(hover:hover)_and_(pointer:fine)]:group-hover/image:ease-[cubic-bezier(0.22,1,0.36,1)]",
                    cabin.imagePosition,
                  )}
                  fill
                  priority={activeCabin === initialCabin}
                  sizes="(min-width: 1024px) 58vw, 100vw"
                  src={cabin.image}
                />
                <div
                  aria-hidden="true"
                  className="pointer-events-none absolute inset-0 bg-[linear-gradient(to_top,rgba(9,9,9,0.34),transparent_35%)] opacity-100 transition-opacity duration-500 motion-reduce:transition-none [@media(hover:hover)_and_(pointer:fine)]:group-hover/image:opacity-80"
                />
              </motion.div>
            </motion.div>
          </AnimatePresence>
        </TabsContent>
      </Tabs>
    </section>
  );
};

export { CabinExperience };
