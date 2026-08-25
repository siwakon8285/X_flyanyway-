import { Accessibility, Gauge, Layers3, MoveHorizontal } from "lucide-react";

import { Container } from "@/components/layout/Container";
import { Section } from "@/components/layout/Section";
import { CountUp } from "@/components/motion/CountUp";
import { FlipDemo } from "@/components/motion/FlipDemo";
import { MotionPresenceDemo } from "@/components/motion/MotionPresenceDemo";
import { ParallaxMedia } from "@/components/motion/ParallaxMedia";
import { PinnedSection } from "@/components/motion/PinnedSection";
import { Reveal } from "@/components/motion/Reveal";
import { SplitText } from "@/components/motion/SplitText";
import { Badge } from "@/components/ui/Badge";
import {
  Card,
  CardContent,
  CardDescription,
  CardHeader,
  CardTitle,
} from "@/components/ui/Card";
import { Heading } from "@/components/ui/Heading";

const MotionPlayground = () => (
  <>
    <Section className="border-y border-border/80 bg-surface/35" id="motion">
      <Container>
        <div className="mb-12 max-w-reading">
          <Badge variant="brand">Motion foundation</Badge>
          <Heading className="mt-5">Controlled movement, reusable rules.</Heading>
          <p className="mt-5 text-body-lg text-muted-foreground">
            These are isolated engineering proofs. They validate lifecycle,
            accessibility, and motion ownership—not final page choreography.
          </p>
        </div>

        <Reveal className="grid gap-card-gap md:grid-cols-3" stagger={0.1}>
          <Card variant="elevated">
            <Gauge aria-hidden="true" className="size-6 text-brand" />
            <CardHeader className="mb-0 mt-8">
              <CardTitle>Viewport reveal</CardTitle>
              <CardDescription>Opacity and a modest vertical offset.</CardDescription>
            </CardHeader>
          </Card>
          <Card variant="elevated">
            <Layers3 aria-hidden="true" className="size-6 text-brand" />
            <CardHeader className="mb-0 mt-8">
              <CardTitle>Scoped contexts</CardTitle>
              <CardDescription>Animations leave with their component.</CardDescription>
            </CardHeader>
          </Card>
          <Card variant="elevated">
            <Accessibility aria-hidden="true" className="size-6 text-brand" />
            <CardHeader className="mb-0 mt-8">
              <CardTitle>Static fallback</CardTitle>
              <CardDescription>Content never depends on motion to exist.</CardDescription>
            </CardHeader>
          </Card>
        </Reveal>
      </Container>
    </Section>

    <Section id="split-text">
      <Container>
        <p className="text-label text-brand">05 · Split text</p>
        <SplitText
          as="h2"
          className="mt-5 max-w-5xl text-h1"
          split="words,chars"
          text="Readable first. Cinematic when appropriate."
        />
        <p className="mt-7 max-w-reading text-body-lg text-muted-foreground">
          SplitType mutates only this text node after hydration. Its generated
          units are hidden from assistive technology and reverted on cleanup.
        </p>
      </Container>
    </Section>

    <Section className="border-y border-border/80" id="scroll-motion">
      <Container>
        <div className="grid gap-10 lg:grid-cols-[0.72fr_1.28fr] lg:items-center">
          <div>
            <p className="text-label text-brand">06 · Scroll foundations</p>
            <Heading className="mt-4" size="h3">
              Restrained parallax, placeholder media.
            </Heading>
            <p className="mt-5 text-body text-muted-foreground">
              The effect uses transforms only, switches off below tablet width,
              and remains completely static when reduced motion is requested.
            </p>
          </div>
          <ParallaxMedia
            aria-label="Abstract parallax placeholder"
            className="min-h-80 rounded-surface border border-border bg-surface"
          >
            <div className="flex min-h-96 scale-110 items-center justify-center bg-[radial-gradient(circle_at_28%_28%,rgb(255_212_0_/_22%),transparent_28%),linear-gradient(135deg,#1a1a1a,#090909)]">
              <div className="grid size-28 place-items-center rounded-full border border-brand/40 bg-brand/10">
                <MoveHorizontal aria-hidden="true" className="size-9 text-brand" />
              </div>
            </div>
          </ParallaxMedia>
        </div>
      </Container>
    </Section>

    <Section id="count-up">
      <Container>
        <p className="text-label text-brand">07 · Count up</p>
        <div className="mt-10 grid gap-card-gap sm:grid-cols-2">
          <Card variant="bordered">
            <CardContent>
              <CountUp className="text-display text-brand" end={156} />
              <p className="mt-4 text-body-lg text-muted-foreground">
                Demo destinations
              </p>
            </CardContent>
          </Card>
          <Card variant="bordered">
            <CardContent>
              <CountUp className="text-display text-brand" end={92} suffix="%" />
              <p className="mt-4 text-body-lg text-muted-foreground">
                Demo load factor
              </p>
            </CardContent>
          </Card>
        </div>
      </Container>
    </Section>

    <Section className="border-y border-border/80 bg-surface/35" id="pinned-proof">
      <Container>
        <PinnedSection aria-label="Short pinned-section foundation proof">
          <Card className="min-h-64" variant="elevated">
            <p className="text-label text-brand">08 · Short pin proof</p>
            <Heading className="mt-5" size="h3">
              A brief desktop validation—not a story section.
            </Heading>
            <p className="mt-5 max-w-reading text-body text-muted-foreground">
              Desktop receives a short pin. Tablet, mobile, and reduced-motion
              environments keep this ordinary static section.
            </p>
          </Card>
        </PinnedSection>
      </Container>
    </Section>

    <Section id="layout-motion">
      <Container>
        <div className="mb-12 max-w-reading">
          <p className="text-label text-brand">09 · Layout and UI motion</p>
          <Heading className="mt-4">One tool owns each behavior.</Heading>
        </div>
        <div className="grid gap-card-gap xl:grid-cols-2">
          <Card variant="elevated">
            <CardHeader>
              <CardTitle>GSAP Flip</CardTitle>
              <CardDescription>
                Verifies state-to-state layout movement for future flows.
              </CardDescription>
            </CardHeader>
            <CardContent>
              <FlipDemo />
            </CardContent>
          </Card>
          <Card variant="elevated">
            <CardHeader>
              <CardTitle>Motion presence</CardTitle>
              <CardDescription>
                Motion handles this small component-level enter and exit only.
              </CardDescription>
            </CardHeader>
            <CardContent>
              <MotionPresenceDemo />
            </CardContent>
          </Card>
        </div>
      </Container>
    </Section>

    <Section className="border-t border-border/80" id="motion-accessibility" spacing="sm">
      <Container>
        <div className="grid gap-8 lg:grid-cols-2">
          <div>
            <p className="text-label text-brand">Reduced motion</p>
            <p className="mt-3 text-body text-muted-foreground">
              Smooth scrolling, parallax, pinning, counters, and layout morphs
              become static while all information remains visible.
            </p>
          </div>
          <div>
            <p className="text-label text-brand">Lenis validation</p>
            <p className="mt-3 text-body text-muted-foreground">
              One global instance follows native input and synchronizes with the
              GSAP ticker and ScrollTrigger without React scroll-frame state.
            </p>
          </div>
        </div>
      </Container>
    </Section>
  </>
);

export { MotionPlayground };
