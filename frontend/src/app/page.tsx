import {
  ArrowRight,
  CircleAlert,
  Info,
  PlaneTakeoff,
} from "lucide-react";

import { BrandMark } from "@/components/brand/BrandMark";
import { BrandWordmark } from "@/components/brand/BrandWordmark";
import { Container } from "@/components/layout/Container";
import { Section } from "@/components/layout/Section";
import { MotionPlayground } from "@/components/motion/MotionPlayground";
import { Badge } from "@/components/ui/Badge";
import { Button, buttonVariants } from "@/components/ui/Button";
import {
  Card,
  CardContent,
  CardDescription,
  CardHeader,
  CardTitle,
} from "@/components/ui/Card";
import {
  Dialog,
  DialogClose,
  DialogContent,
  DialogDescription,
  DialogFooter,
  DialogHeader,
  DialogTitle,
  DialogTrigger,
} from "@/components/ui/Dialog";
import { Heading } from "@/components/ui/Heading";
import { IconButton } from "@/components/ui/IconButton";
import { Input } from "@/components/ui/Input";
import { Label } from "@/components/ui/Label";
import {
  Select,
  SelectContent,
  SelectItem,
  SelectTrigger,
  SelectValue,
} from "@/components/ui/Select";
import { Separator } from "@/components/ui/Separator";
import { Skeleton } from "@/components/ui/Skeleton";
import { Tabs, TabsContent, TabsList, TabsTrigger } from "@/components/ui/Tabs";
import { Tooltip, TooltipProvider } from "@/components/ui/Tooltip";

const colorTokens = [
  { className: "bg-brand text-brand-foreground", hex: "#FFD400", name: "Brand" },
  {
    className: "bg-background text-foreground ring-1 ring-border",
    hex: "#090909",
    name: "Background",
  },
  {
    className: "bg-surface text-foreground ring-1 ring-border",
    hex: "#121212",
    name: "Surface",
  },
  {
    className: "bg-surface-elevated text-foreground ring-1 ring-border",
    hex: "#1A1A1A",
    name: "Elevated",
  },
  { className: "bg-foreground text-background", hex: "#F5F3EA", name: "Foreground" },
  { className: "bg-muted-foreground text-background", hex: "#A5A5A5", name: "Muted" },
] as const;

const Home = () => (
  <main className="flex-1" id="top">
    <Section className="border-b border-border/80" spacing="lg">
      <Container>
        <div className="grid items-end gap-12 lg:grid-cols-[minmax(0,1fr)_22rem]">
          <div>
            <Badge variant="brand">Temporary Motion Foundation Preview</Badge>
            <p className="mt-8 text-label text-muted-foreground">X-FLY ANYWAY</p>
            <Heading as="h1" className="mt-4 max-w-5xl" size="display">
              Design + Motion
              <br />
              Foundation
            </Heading>
          </div>
          <div className="space-y-6 border-l border-brand/50 pl-6">
            <p className="text-body-lg text-muted-foreground">
              A practical preview of the visual language, accessible UI
              primitives, and production-safe motion architecture.
            </p>
            <p className="text-caption text-muted-foreground">
              Not the final X-Fly landing page · No final hero or booking flow
            </p>
          </div>
        </div>
      </Container>
    </Section>

    <Section id="brand" spacing="sm">
      <Container>
        <div className="grid gap-card-gap md:grid-cols-2">
          <Card className="flex min-h-64 flex-col justify-between" variant="elevated">
            <BrandMark className="size-14" label="X-Fly brand mark" />
            <CardHeader className="mb-0 mt-12">
              <CardTitle>Confident by design</CardTitle>
              <CardDescription>
                Near-black foundations let the yellow marker signal action and
                priority without overwhelming the interface.
              </CardDescription>
            </CardHeader>
          </Card>
          <Card className="flex min-h-64 items-center justify-center" variant="bordered">
            <BrandWordmark className="text-base sm:text-lg" />
          </Card>
        </div>
      </Container>
    </Section>

    <Section className="border-y border-border/80" id="colors" spacing="md">
      <Container>
        <div className="mb-10 max-w-reading">
          <p className="text-label text-brand">01 · Color system</p>
          <Heading className="mt-3">Yellow leads. Dark tones carry.</Heading>
        </div>
        <div className="grid gap-3 sm:grid-cols-2 lg:grid-cols-3 3xl:grid-cols-6">
          {colorTokens.map((token) => (
            <div
              className={`flex min-h-36 flex-col justify-between rounded-surface p-5 ${token.className}`}
              key={token.name}
            >
              <span className="text-label">{token.name}</span>
              <span className="font-mono text-xs opacity-75">{token.hex}</span>
            </div>
          ))}
        </div>
      </Container>
    </Section>

    <Section id="typography" spacing="md">
      <Container>
        <p className="text-label text-brand">02 · Typography</p>
        <div className="mt-10 divide-y divide-border border-y border-border">
          <div className="grid gap-4 py-8 md:grid-cols-[8rem_1fr]">
            <span className="text-caption text-muted-foreground">Display</span>
            <p className="text-display">GO ANYWAY.</p>
          </div>
          <div className="grid gap-4 py-8 md:grid-cols-[8rem_1fr]">
            <span className="text-caption text-muted-foreground">Heading 1</span>
            <p className="text-h1">A wider point of view.</p>
          </div>
          <div className="grid gap-4 py-8 md:grid-cols-[8rem_1fr]">
            <span className="text-caption text-muted-foreground">Heading 2</span>
            <p className="text-h2">Designed for every horizon.</p>
          </div>
          <div className="grid gap-4 py-8 md:grid-cols-[8rem_1fr]">
            <span className="text-caption text-muted-foreground">Heading 3</span>
            <p className="text-h3">A dependable interface foundation.</p>
          </div>
          <div className="grid gap-4 py-8 md:grid-cols-[8rem_1fr]">
            <span className="text-caption text-muted-foreground">Body scale</span>
            <div className="max-w-reading space-y-4 text-muted-foreground">
              <p className="text-body-lg">
                Body large introduces important editorial context.
              </p>
              <p className="text-body">
                Body copy stays comfortable across forms and detailed workflows.
              </p>
              <p className="text-body-sm">
                Body small supports concise secondary details.
              </p>
              <p className="text-caption">Caption · Global network</p>
              <p className="text-label">Label · Departure city</p>
            </div>
          </div>
        </div>
      </Container>
    </Section>

    <Section className="bg-surface/55" id="components" spacing="md">
      <Container>
        <div className="mb-12 max-w-reading">
          <p className="text-label text-brand">03 · Controls</p>
          <Heading className="mt-3">Clear states. Strong focus.</Heading>
          <p className="mt-5 text-body-lg text-muted-foreground">
            Native controls and Radix behavior provide keyboard access while
            semantic tokens keep every state consistent.
          </p>
        </div>

        <div className="grid gap-card-gap xl:grid-cols-2">
          <Card variant="elevated">
            <CardHeader>
              <CardTitle>Buttons</CardTitle>
              <CardDescription>Five variants across shared sizes and states.</CardDescription>
            </CardHeader>
            <CardContent className="flex flex-wrap items-center gap-3">
              <Button>Primary</Button>
              <Button variant="secondary">Secondary</Button>
              <Button variant="outline">Outline</Button>
              <Button variant="ghost">Ghost</Button>
              <Button variant="destructive">Destructive</Button>
              <Button disabled>Disabled</Button>
              <Button loading>Loading</Button>
              <Button size="sm" variant="outline">
                Small
              </Button>
              <Button size="lg">Large</Button>
            </CardContent>
          </Card>

          <Card variant="elevated">
            <CardHeader>
              <CardTitle>Form fields</CardTitle>
              <CardDescription>Labels and errors remain readable without color alone.</CardDescription>
            </CardHeader>
            <CardContent className="grid gap-6 sm:grid-cols-2">
              <div className="space-y-2">
                <Label htmlFor="departure-city">Departure city</Label>
                <Input id="departure-city" placeholder="Bangkok" />
              </div>
              <div className="space-y-2">
                <Label htmlFor="cabin-preview">Cabin</Label>
                <Select defaultValue="business">
                  <SelectTrigger id="cabin-preview">
                    <SelectValue placeholder="Select cabin" />
                  </SelectTrigger>
                  <SelectContent>
                    <SelectItem value="economy">Economy</SelectItem>
                    <SelectItem value="business">Business</SelectItem>
                    <SelectItem value="first">First</SelectItem>
                  </SelectContent>
                </Select>
              </div>
              <div className="space-y-2 sm:col-span-2">
                <Label htmlFor="invalid-reference">Booking reference</Label>
                <Input
                  aria-describedby="reference-error"
                  aria-invalid="true"
                  defaultValue="XF-"
                  id="invalid-reference"
                />
                <p
                  className="flex items-center gap-2 text-body-sm text-destructive"
                  id="reference-error"
                >
                  <CircleAlert aria-hidden="true" className="size-4" />
                  Enter the six-character booking reference.
                </p>
              </div>
              <div className="space-y-2 sm:col-span-2">
                <Label htmlFor="disabled-field">Unavailable field</Label>
                <Input
                  disabled
                  id="disabled-field"
                  readOnly
                  value="Disabled state"
                />
              </div>
            </CardContent>
          </Card>

          <Card variant="elevated">
            <CardHeader>
              <CardTitle>Interactive primitives</CardTitle>
              <CardDescription>
                Focus-managed overlays and keyboard-friendly segmented views.
              </CardDescription>
            </CardHeader>
            <CardContent className="space-y-8">
              <Tabs defaultValue="public">
                <TabsList aria-label="Interface context">
                  <TabsTrigger value="public">Public</TabsTrigger>
                  <TabsTrigger value="operations">Operations</TabsTrigger>
                </TabsList>
                <TabsContent value="public">
                  <p className="text-muted-foreground">
                    Spacious editorial layouts support the future passenger journey.
                  </p>
                </TabsContent>
                <TabsContent value="operations">
                  <p className="text-muted-foreground">
                    Dense, readable surfaces can support future operational tools.
                  </p>
                </TabsContent>
              </Tabs>
              <div className="flex flex-wrap items-center gap-3">
                <Dialog>
                  <DialogTrigger className="inline-flex h-11 items-center justify-center rounded-control border border-border px-5 text-sm font-medium outline-none hover:bg-muted focus-visible:ring-2 focus-visible:ring-focus">
                    Inspect dialog
                  </DialogTrigger>
                  <DialogContent>
                    <DialogHeader>
                      <DialogTitle>Accessible overlay foundation</DialogTitle>
                      <DialogDescription>
                        Focus management, Escape handling, title, and description
                        are provided without animation choreography.
                      </DialogDescription>
                    </DialogHeader>
                    <DialogFooter>
                      <DialogClose className="inline-flex h-11 items-center justify-center rounded-control bg-brand px-5 text-sm font-medium text-brand-foreground outline-none focus-visible:ring-2 focus-visible:ring-focus">
                        Close preview
                      </DialogClose>
                    </DialogFooter>
                  </DialogContent>
                </Dialog>
                <TooltipProvider delayDuration={200}>
                  <Tooltip content="Interface icons require an accessible name.">
                    <IconButton label="Icon usage guidance" variant="outline">
                      <Info aria-hidden="true" />
                    </IconButton>
                  </Tooltip>
                </TooltipProvider>
              </div>
            </CardContent>
          </Card>

          <Card variant="elevated">
            <CardHeader>
              <CardTitle>Loading foundation</CardTitle>
              <CardDescription>
                Lightweight skeletons reserve layout while content is pending.
              </CardDescription>
            </CardHeader>
            <CardContent className="space-y-4">
              <Skeleton className="h-5 w-32" />
              <Skeleton className="h-12 w-full" />
              <div className="grid grid-cols-3 gap-3">
                <Skeleton className="h-20" />
                <Skeleton className="h-20" />
                <Skeleton className="h-20" />
              </div>
            </CardContent>
          </Card>
        </div>
      </Container>
    </Section>

    <Section id="surfaces" spacing="md">
      <Container>
        <div className="mb-12 flex flex-col gap-6 lg:flex-row lg:items-end lg:justify-between">
          <div className="max-w-reading">
            <p className="text-label text-brand">04 · Surfaces</p>
            <Heading className="mt-3">Depth, with restraint.</Heading>
          </div>
          <Badge variant="outline">Glass is an accent, not a default</Badge>
        </div>
        <div className="grid gap-card-gap md:grid-cols-2 xl:grid-cols-4">
          {(["surface", "elevated", "bordered", "glass"] as const).map(
            (variant) => (
              <Card className="min-h-64" key={variant} variant={variant}>
                <CardHeader>
                  <PlaneTakeoff aria-hidden="true" className="size-6 text-brand" />
                  <CardTitle className="pt-6 capitalize">{variant}</CardTitle>
                </CardHeader>
                <CardContent className="text-muted-foreground">
                  A reusable surface for focused content and future workflows.
                </CardContent>
              </Card>
            ),
          )}
        </div>
      </Container>
    </Section>

    <MotionPlayground />

    <Section className="border-t border-border/80" spacing="sm">
      <Container>
        <div className="flex flex-col gap-8 lg:flex-row lg:items-center lg:justify-between">
          <div className="max-w-reading">
            <p className="text-label text-brand">Foundation ready for review</p>
            <Heading className="mt-3" size="h3">
              Design and motion foundations are ready for review.
            </Heading>
          </div>
          <a className={buttonVariants({ size: "lg" })} href="#top">
            Review from top
            <ArrowRight aria-hidden="true" />
          </a>
        </div>
        <Separator className="my-10" />
        <p className="text-body-sm text-muted-foreground">
          This page validates design and motion primitives only. It intentionally
          does not represent the final X-Fly landing experience.
        </p>
      </Container>
    </Section>
  </main>
);

export default Home;
