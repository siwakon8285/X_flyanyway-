# X-Fly Anyway — DESIGN.md

> **Design & Frontend Experience Master Plan**
>
> เป้าหมายของเอกสารนี้คือกำหนดทิศทาง Visual Design, Motion System, Page Experience, Component Architecture และแผนการพัฒนาแบบแยก Branch ตั้งแต่เริ่มต้นจนถึง Production-ready สำหรับ X-Fly Anyway
>
> เอกสารนี้เน้นว่า X-Fly Anyway **ต้องไม่ออกมาเป็นเว็บ Booking ธรรมดา** แต่ต้องเป็น Airline Booking Experience ที่มีความ cinematic, premium, editorial, immersive และมี motion ที่สร้างความ “ว้าว” ตั้งแต่หน้าแรกจนถึงตอนออก E-Ticket

---

# 1. Product Vision

X-Fly Anyway คือสายการบินระดับโลกที่ให้บริการผ่าน Web Booking โดยมีภาพลักษณ์:

- Premium
- Cinematic
- Modern
- Editorial
- Fast
- Confident
- Global
- Futuristic
- Yellow-driven brand identity

เว็บไซต์ต้องรู้สึกเหมือนประสบการณ์ของแบรนด์ระดับ luxury aviation มากกว่าเว็บฟอร์มจองตั๋วทั่วไป

ผู้ใช้ควรรู้สึกว่า:

> “กำลังเริ่มต้นการเดินทาง” ไม่ใช่ “กำลังกรอกฟอร์ม”

---

# 2. Core Design Principles

## 2.1 Motion is part of the product

Animation ไม่ใช่ของตกแต่ง แต่ใช้เพื่อ:

- เล่าเรื่อง
- พาผู้ใช้เข้าสู่ booking flow
- เชื่อม section เข้าหากัน
- แสดง hierarchy
- แสดง state change
- สร้าง premium perception
- ทำให้ seat selection และ ticketing มีเอกลักษณ์

## 2.2 Cinematic, not chaotic

Motion ทุกชิ้นต้องมีหน้าที่

ห้าม:

- animation พร้อมกันทุก element
- parallax แรงเกินไป
- scroll hijacking จนควบคุมเว็บยาก
- transition ยาวเกินไป
- effect ที่ทำให้ booking flow ใช้งานยาก

## 2.3 Booking must remain fast

หน้า Landing สามารถ cinematic ได้มาก

แต่เมื่อเข้าสู่:

- Flight Search
- Seat Selection
- Passenger Form
- Payment
- Manage Booking

UX ต้องเร็ว อ่านง่าย และชัดเจน

## 2.4 Premium Yellow

สีเหลืองเป็น Signature Accent ไม่ใช่ใช้ทั้งหน้าจอ

ใช้ Yellow เพื่อ:

- CTA
- Highlight
- Active state
- Seat selection
- Progress indicator
- Focus state
- Important numbers
- Motion accent

พื้นหลักใช้:

- Near Black
- Charcoal
- Off White
- Warm Gray
- Dark Navy
- Yellow Accent

---

# 3. Visual Direction

ภาพอ้างอิงมีลักษณะสำคัญที่เราจะนำมาเป็น direction:

- Full-bleed aircraft photography/video
- Hero typography ขนาดใหญ่
- Dark cinematic backgrounds
- Editorial asymmetric layouts
- Sticky / pinned section
- Large image transitions
- Horizontal content sections
- Split-screen storytelling
- Strong white typography
- Thin yellow/gold accent lines
- Scroll-driven image transitions
- Luxury aviation feeling

เรา **ไม่ clone layout ตรง ๆ** แต่ใช้ visual language แบบเดียวกันเพื่อสร้าง identity ของ X-Fly Anyway

---

# 4. Brand System

## Primary Color

```txt
X-Fly Yellow
#FFD400
```

สามารถปรับหลัง design exploration ได้

## Supporting Colors

```txt
Near Black
#090909

Charcoal
#121212

Dark Surface
#1A1A1A

Warm White
#F5F3EA

Muted Gray
#A5A5A5
```

## Design Treatment

- Black / dark surface เป็น canvas หลัก
- Yellow ใช้เฉพาะจุดที่ต้องการ attention
- White typography contrast สูง
- Images มี warm cinematic grade
- Border บาง 1px
- Glass / blur ใช้แบบ restrained
- Corner radius ไม่ควรกลมเกินไป
- UI มีความ editorial มากกว่า SaaS dashboard

---

# 5. Typography Direction

แนะนำใช้:

## Display

- Geist Sans / Inter / Manrope / Instrument Sans
- หรือ Editorial Sans ที่มี geometry ชัด

## Body

- Geist Sans / Inter

## Typography Behavior

Hero:

```txt
72px – 140px desktop
42px – 72px tablet
36px – 56px mobile
```

ใช้:

- uppercase บางส่วน
- large line-height tight
- letter spacing controlled
- text reveal
- mask animation
- split text
- stagger

---

# 6. Motion Technology Stack

เพื่อให้ motion สวยแต่ architecture ยังควบคุมได้ ให้แบ่งความรับผิดชอบชัดเจน

## 6.1 GSAP

ใช้เป็น motion engine หลัก

Packages:

```txt
gsap
@gsap/react
```

ใช้สำหรับ:

- Hero timelines
- ScrollTrigger
- Pinned sections
- Parallax
- Horizontal scroll
- Image masking
- Timeline choreography
- Counter animation
- Section transitions
- Seat reveal
- Ticket reveal

## 6.2 GSAP ScrollTrigger

ใช้กับ:

- scroll-linked animation
- pinned storytelling
- section progress
- image zoom
- text reveal
- horizontal panels
- timeline sync

## 6.3 GSAP Flip

ใช้สำหรับ:

- Cabin filter → Seat map transition
- Selected flight card → Booking summary
- Seat selection state
- Expanding cards
- Layout morph

## 6.4 Motion / Framer Motion

ใช้เฉพาะ component-level interaction

เช่น:

- modal
- drawer
- dropdown
- page micro-transition
- toast
- tabs
- buttons
- form feedback

ห้ามใช้ซ้อนกับ GSAP ใน animation เดียวกันโดยไม่มีเหตุผล

## 6.5 Lenis

ใช้ smooth scrolling

```txt
lenis
```

GSAP ScrollTrigger ต้อง sync กับ Lenis

ต้องมี fallback สำหรับ:

```txt
prefers-reduced-motion
```

## 6.6 SplitType

ใช้แบ่ง:

- character
- word
- line

สำหรับ text reveal

ต้อง cleanup หลัง animation

## 6.7 CSS Animation

ใช้กับ microinteraction ที่ไม่ต้องใช้ JS

เช่น:

- hover
- focus
- shimmer
- loading
- pulse
- seat state
- border glow
- gradient shift

## 6.8 Optional Visual Tools

ใช้เมื่อมีคุณค่าจริง:

- Rive — logo / icon / small interactive animation
- Lottie — lightweight decorative animation
- WebGL / Three.js — optional only, ไม่ใช่ dependency หลัก
- CSS mask / clip-path — image reveals
- View Transition API — progressive enhancement

---

# 7. Motion Rules

## Motion duration

```txt
Micro interaction: 120–220ms
UI transition: 220–450ms
Section reveal: 500–900ms
Hero cinematic timeline: 1–2.5s
```

## Easing

ใช้ easing ที่ให้ความ premium:

```txt
power2.out
power3.out
expo.out
circ.out
```

หลีกเลี่ยง bounce ที่ดู playful เกิน brand

## Reduced Motion

ถ้า:

```css
prefers-reduced-motion: reduce
```

ระบบต้อง:

- disable parallax
- disable pinned-heavy animation
- reduce transforms
- remove smooth scrolling
- keep content fully usable

---

# 8. Website Experience Map

```txt
Landing
  ↓
Immersive Flight Search
  ↓
Flight Results
  ↓
Flight Detail
  ↓
Cabin Selection
  ↓
Seat Experience
  ↓
Passenger Details
  ↓
Booking Review
  ↓
Mock Payment
  ↓
E-Ticket Reveal
  ↓
Manage Booking
```

Admin:

```txt
Admin Login
  ↓
Analytics Dashboard
  ↓
Flight Management
  ↓
Booking Management
  ↓
Ticket Management
  ↓
External API Clients
```

---

# 9. Landing Page Experience

Landing page ต้องเป็น Showcase ของ project

## Hero

Full-screen aircraft visual

ประกอบด้วย:

- cinematic image/video
- navbar overlay
- giant headline
- subtitle
- primary CTA
- booking search trigger
- subtle grain
- animated gradient
- image zoom
- text mask reveal

ตัวอย่าง copy direction:

```txt
FLY BEYOND.
BOOK THE EXTRAORDINARY.
```

หรือ:

```txt
THE WORLD IS CLOSER
THAN IT LOOKS.
```

## Hero Motion

เมื่อเปิดหน้า:

1. black screen fade
2. yellow brand line expand
3. aircraft image reveal จาก mask
4. headline split reveal
5. subtitle fade + rise
6. search CTA enters
7. subtle hero zoom begins
8. scroll indicator appears

---

# 10. Scroll Storytelling Section

สร้าง pinned cinematic section

โครง:

```txt
Section 1
Global Network

Section 2
156 Countries

Section 3
Choose Your Cabin

Section 4
Every Seat. Your Choice.

Section 5
Next Stop: The Moon
```

ใช้:

- ScrollTrigger
- pinned content
- crossfade image
- background video
- oversized text
- number counters
- line drawing

---

# 11. Horizontal Journey Section

ใช้ horizontal scrolling แบบ controlled

Cards:

```txt
Explore
Choose
Book
Fly
```

แต่ละ card มี:

- full image
- number
- short copy
- motion reveal
- hover interaction

Desktop:

horizontal pin

Mobile:

native vertical stack

---

# 12. Global Destination Section

แสดง network 156 ประเทศด้วย visual ที่ดู dynamic

MVP ไม่ต้องทำ 3D Globe

ใช้:

- animated world map SVG
- yellow route lines
- destination points
- counters
- hover destination
- route filter

Optional Future:

- WebGL globe
- Three.js route visualization

---

# 13. Moon Expansion Section

นี่คือ signature storytelling section

Visual:

```txt
Earth aircraft
↓
Dark transition
↓
Stars / lunar visual
↓
NEXT: THE MOON
```

สามารถใช้:

- parallax star layers
- animated noise
- masked moon reveal
- yellow trajectory line

ไม่ต้องเป็น booking feature จริงใน MVP

เป็น future vision / branding section

---

# 14. Flight Search Experience

Flight Search ต้องไม่เหมือน form ธรรมดา

## Desktop

ใช้ floating search dock:

```txt
FROM
TO
DATE
PASSENGERS
CABIN
SEARCH
```

Interaction:

- airport autocomplete
- swap origin/destination animation
- custom date panel
- cabin selector
- passenger counter
- animated search CTA

เมื่อ Search:

Search bar morph เป็น compact top bar ด้วย GSAP Flip

แล้ว Results slide/reveal ขึ้นมา

---

# 15. Flight Results Design

Layout:

```txt
Sticky Filter Rail
+
Flight Cards
```

Flight Card:

- route
- time
- duration
- cabin availability
- price
- seat count
- flight number

Motion:

- stagger reveal
- card hover lift
- price emphasis
- selection expands card
- selected flight transitions into detail view

---

# 16. Flight Detail

ต้องรู้สึกเหมือน product showcase

ใช้:

- route timeline
- aircraft image
- cabin cards
- amenities
- seat availability
- fare breakdown

Cabin tabs:

```txt
Economy
Premium Economy
Business
First
```

ใช้ GSAP Flip ตอนเปลี่ยน cabin

---

# 17. Seat Selection Experience

Seat Map เป็นหนึ่งใน signature experiences

## Visual Direction

เหมือน:

- cinema seat selection
- aircraft cabin
- premium cockpit/cabin UI

มี:

```txt
Cabin Header
Legend
Seat Map
Mini Flight Summary
Selection Summary
Countdown Hold
```

Seat States:

```txt
Available
Selected
Held
Booked
Unavailable
```

Motion:

- seat hover scale
- selected seat glow
- row stagger reveal
- cabin filter morph
- aisle motion
- selected seat flies/morphs into summary panel

Optional:

- subtle aircraft cabin background
- ambient lighting effect

---

# 18. Seat Hold UX

เมื่อเลือก seat:

```txt
Seat 3A is held for you
09:59
```

Countdown visible

เมื่อเวลาลด:

- progress ring
- subtle urgency
- no aggressive red until near expiry

เมื่อ expire:

- toast
- seat release animation
- user can select again

---

# 19. Passenger Form

หลัง cinematic sections ต้องกลับมาเป็น UX ที่นิ่ง

ใช้:

- large step labels
- floating summary
- clear validation
- progressive disclosure
- animated error feedback
- autosave temporary state in browser where appropriate

Stepper:

```txt
01 Flight
02 Seat
03 Passenger
04 Payment
05 Ticket
```

---

# 20. Booking Review

ใช้ large editorial summary

ซ้าย:

- passenger
- route
- seat
- cabin
- date/time

ขวา:

- fare
- taxes/mock fee
- total
- cancellation policy

CTA:

```txt
CONTINUE TO PAYMENT
```

---

# 21. Mock Payment Experience

3 cards:

```txt
Credit Card
Debit Card
Bitcoin
```

เมื่อเลือก provider:

GSAP Flip / Motion expand card

## Card Mock

มี:

- Cardholder
- Card Number
- Expiry
- CVV

## Bitcoin Mock

มี:

- Mock wallet
- Mock amount
- Simulate Payment

ต้องมี label ชัดเจน:

```txt
DEMO / MOCK PAYMENT
NO REAL CHARGE
```

---

# 22. Payment Success Experience

อย่าใช้แค่ green check

ทำเป็น cinematic transition:

1. processing state
2. yellow line travels across screen
3. confirmation pulse
4. booking reference reveal
5. ticket slides/folds in
6. QR code appears
7. CTA Print / Download / Manage Booking

---

# 23. E-Ticket Design

E-Ticket ต้องเป็น signature visual

ส่วนประกอบ:

- X-Fly branding
- Booking Reference
- Passenger
- Flight
- Route
- Gate placeholder
- Cabin
- Seat
- Date
- QR Code

Motion:

- paper/ticket reveal
- perforation line
- QR fade
- subtle floating effect
- print mode clean

---

# 24. Manage Booking

Search:

```txt
Booking Reference
Last Name
Optional Email
```

หลังเจอ booking:

- animated ticket reveal
- flight status
- seat
- payment
- cancellation eligibility
- refund status

Cancellation CTA ต้องแสดง:

```txt
Free cancellation available until:
DD MMM YYYY HH:mm
```

---

# 25. Cancellation UX

ถ้า >= 24 ชั่วโมง:

```txt
Eligible for 100% refund
```

กดยกเลิก:

1. confirmation dialog
2. booking status transition
3. seat release
4. ticket cancelled
5. mock refund animation
6. receipt / status update

ถ้า < 24 ชั่วโมง:

CTA disabled พร้อม explanation

---

# 26. Admin Design Direction

Admin ไม่ต้อง cinematic เท่าหน้า public

แต่ยังคง brand quality

Style:

- dark professional dashboard
- yellow accent
- dense but readable
- large KPI
- animated charts
- subtle transitions
- strong table UX

---

# 27. Admin Dashboard

KPI:

```txt
Revenue
Bookings
Passengers
Load Factor
Cancellation Rate
Flights Today
```

Analytics:

```txt
Daily
Weekly
Monthly
Custom
```

Charts:

- Revenue Trend
- Booking Trend
- Load Factor
- Most Booked Flights
- Least Booked Flights
- Nationality Distribution
- Route Popularity

ใช้ Recharts

GSAP ใช้ตอน initial reveal เท่านั้น

---

# 28. Flight Management UI

Admin:

```txt
Create Flight
Edit Flight
Cancel Flight
View Flight
```

Fields:

- Flight Number
- Origin
- Destination
- Aircraft
- Departure
- Arrival
- Cabin pricing
- Status

UI:

- wizard หรือ structured drawer
- validation
- preview before save
- destructive confirmation

---

# 29. Booking Management UI

Admin search/filter:

```txt
Booking Reference
Passenger
Flight
Date
Status
Cabin
```

Detail:

- passenger
- flight
- seat
- ticket
- payment
- cancellation
- refund

---

# 30. Ticket Management

Admin สามารถ:

- search ticket
- view QR
- print
- verify ticket
- see cancelled status

---

# 31. External Integration UI

Admin page:

```txt
API Clients
```

สามารถ:

- Create Client
- Revoke Client
- View Client ID
- Regenerate Secret
- Assign Scope
- Disable Client

Scopes:

```txt
flights:read
bookings:read
passengers:read
tickets:read
```

---

# 32. Responsive Strategy

## Desktop

ใช้:

- full cinematic
- pinned sections
- horizontal scroll
- large typography
- split layouts

## Tablet

ลด:

- pin duration
- parallax strength
- section height

## Mobile

ต้องไม่ copy desktop animation ตรง ๆ

เปลี่ยนเป็น:

- vertical storytelling
- swipe card
- reduced sticky
- shorter timeline
- fewer parallax layers
- larger touch targets

---

# 33. Performance Budget

Motion-heavy แต่ต้องเร็ว

Targets:

```txt
LCP < 2.5s target
CLS < 0.1
INP < 200ms target
60fps where possible
```

Rules:

- transform + opacity เป็นหลัก
- avoid layout thrashing
- use requestAnimationFrame-aware libraries
- preload hero asset
- lazy-load below fold
- responsive image
- video poster
- compress media
- unload inactive animations
- cleanup ScrollTrigger
- no animation memory leak

---

# 34. Accessibility

ต้องมี:

- keyboard navigation
- visible focus
- ARIA labels
- seat accessible name
- color-independent seat states
- reduced motion
- semantic HTML
- sufficient contrast
- form error announcement

Seat state ห้ามแยกด้วยสีอย่างเดียว

---

# 35. Frontend Component Architecture

```txt
apps/web/src/

app/
components/
  brand/
  layout/
  navigation/
  motion/
  booking/
  flight/
  seat/
  ticket/
  payment/
  admin/
  analytics/

features/
  flight-search/
  flight-results/
  seat-selection/
  booking/
  payment/
  manage-booking/
  admin-auth/
  admin-flights/
  analytics/

lib/
  api/
  motion/
  validation/
  utils/

hooks/
styles/
types/
```

---

# 36. Motion Utility Architecture

สร้าง central motion layer

```txt
src/lib/motion/
  gsap.ts
  scroll.ts
  easing.ts
  reduced-motion.ts
  transitions.ts
```

และ components:

```txt
components/motion/
  Reveal.tsx
  SplitText.tsx
  ParallaxMedia.tsx
  PinnedSection.tsx
  MagneticButton.tsx
  PageTransition.tsx
  CountUp.tsx
```

ห้ามเขียน GSAP กระจายทุกไฟล์โดยไม่มี lifecycle control

---

# 37. Global Motion Tokens

```txt
duration.fast
duration.normal
duration.slow

ease.standard
ease.enter
ease.exit
ease.cinematic

distance.sm
distance.md
distance.lg
```

เพื่อให้ motion ทั้งเว็บมีบุคลิกเดียวกัน

---

# 38. Asset Strategy

ต้องเตรียม:

```txt
public/
  images/
    hero/
    aircraft/
    destinations/
    cabin/
    moon/
  video/
  icons/
```

Production assets:

- licensed / royalty-free
- optimize WebP/AVIF
- video WebM/MP4
- poster image

ห้ามใช้ hotlink asset จากเว็บอื่นใน final

---

# 39. Main Development Branch Strategy

Branch ทุกอันควรเริ่มจาก branch หลักที่ผ่าน review แล้ว

Naming:

```txt
feat/01-design-foundation
feat/02-motion-foundation
...
```

แต่ละ branch ต้อง:

1. implementation
2. lint
3. typecheck
4. tests
5. responsive review
6. accessibility check
7. performance check
8. merge หลัง review

---

# 40. BRANCH 00 — Design Discovery & Baseline

## Branch

```txt
docs/00-design-discovery
```

## Goal

ล็อก direction ก่อนเขียน UI

## Tasks

- รวบรวม reference
- define brand mood
- define yellow palette
- typography exploration
- spacing system
- border/radius system
- motion mood
- desktop/mobile direction
- define what "wow" means
- define what must remain simple
- create rough page map
- create interaction map
- create animation inventory
- define performance budget
- define reduced motion rules

## Deliverables

```txt
DESIGN.md
docs/design/
docs/motion/
```

## Exit Criteria

ทีม approve:

- Visual direction
- Motion direction
- Tech choices
- Page hierarchy

---

# 41. BRANCH 01 — Frontend Design Foundation

## Branch

```txt
feat/01-design-foundation
```

## Goal

สร้าง design system และ base UI

## Tasks

- Next.js App Router structure
- Tailwind tokens
- CSS variables
- theme
- typography
- spacing
- containers
- responsive breakpoints
- button variants
- input variants
- card surfaces
- glass surfaces
- icon rules
- navigation shell
- footer shell
- loading skeleton
- focus styles

## Components

```txt
Button
IconButton
Input
Select
Badge
Card
Modal
Drawer
Tabs
Tooltip
Container
Section
Heading
```

## Exit Criteria

- Story/demo page แสดงทุก component
- responsive
- dark theme consistent
- yellow accent consistent
- accessibility baseline

---

# 42. BRANCH 02 — Motion Foundation

## Branch

```txt
feat/02-motion-foundation
```

## Goal

สร้าง motion architecture ที่ใช้ต่อได้ทั้งโปรเจกต์

## Tasks

- install GSAP
- @gsap/react
- ScrollTrigger setup
- Flip setup
- Lenis setup
- Motion setup
- SplitType setup
- reduced motion hook
- global easing
- global duration
- route transition strategy
- GSAP context cleanup
- ScrollTrigger cleanup
- mobile motion strategy

## Components

```txt
Reveal
SplitText
ParallaxMedia
PinnedSection
CountUp
MagneticButton
PageTransition
```

## Tests

- navigation leak
- ScrollTrigger duplicate
- reduced motion
- resize behavior

---

# 43. BRANCH 03 — Navigation + Global Shell

## Branch

```txt
feat/03-global-shell
```

## Goal

สร้าง chrome ของเว็บ

## Tasks

- transparent hero navbar
- sticky/scrolled navbar
- logo animation
- nav underline motion
- request/book CTA
- mobile menu animation
- global page background
- footer
- route transition
- scroll progress

## Motion

- navbar backdrop morph
- logo scale
- menu stagger
- page fade/clip transition

---

# 44. BRANCH 04 — Cinematic Hero

## Branch

```txt
feat/04-cinematic-hero
```

## Goal

สร้าง first impression ที่ว้าวที่สุด

## Tasks

- full-screen hero
- aircraft image/video
- hero typography
- brand statement
- CTA
- grain overlay
- gradient overlays
- intro timeline
- hero image slow zoom
- scroll indicator
- responsive hero
- reduced motion

## Motion Timeline

```txt
Brand line
→ image reveal
→ headline reveal
→ subtitle
→ CTA
→ ambient zoom
```

## Exit Criteria

- 60fps target
- mobile fallback
- LCP asset optimized

---

# 45. BRANCH 05 — Scroll Storytelling

## Branch

```txt
feat/05-scroll-storytelling
```

## Goal

สร้าง pinned scroll experience คล้าย reference

## Sections

```txt
156 Countries
Choose Your Cabin
Every Seat Is Yours
Fly Beyond Earth
```

## Tasks

- pinned section
- text changes per scroll stage
- image crossfade
- image zoom
- parallax
- progress markers
- responsive fallback

## Exit Criteria

- no scroll trap
- wheel/touch usable
- mobile does not force heavy pinning

---

# 46. BRANCH 06 — Horizontal Journey

## Branch

```txt
feat/06-horizontal-journey
```

## Goal

สร้าง horizontal section

## Cards

```txt
Search
Choose
Book
Fly
```

## Tasks

- horizontal ScrollTrigger
- large cards
- image reveal
- card numbering
- hover animation
- mobile vertical fallback
- accessibility order

---

# 47. BRANCH 07 — Global Network & Moon Vision

## Branch

```txt
feat/07-global-network-moon
```

## Goal

โชว์ global reach + future vision

## Tasks

- animated map/SVG
- route lines
- destination dots
- 156 country counter
- hover destination
- moon transition section
- star layers
- moon reveal
- trajectory animation

## Optional

Three.js globe only if performance/time budget allows

---

# 48. BRANCH 08 — Flight Search Experience

## Branch

```txt
feat/08-flight-search-ui
```

## Goal

สร้าง search UX ที่ premium

## Tasks

- floating booking dock
- airport autocomplete
- origin/destination swap
- date picker
- passenger selector
- cabin selector
- search button
- validation
- search loading state
- GSAP Flip compact state

## API Integration

เชื่อม public flight search API

---

# 49. BRANCH 09 — Flight Results

## Branch

```txt
feat/09-flight-results-ui
```

## Tasks

- flight cards
- filter rail
- sorting
- empty state
- loading skeleton
- price display
- availability
- route timeline
- card stagger
- selected card expansion

---

# 50. BRANCH 10 — Flight Detail + Cabin Experience

## Branch

```txt
feat/10-flight-detail-cabin
```

## Tasks

- aircraft detail
- route timeline
- fare summary
- cabin selector
- cabin image
- amenities
- seat availability
- animated cabin switching
- GSAP Flip transitions

---

# 51. BRANCH 11 — Seat Map Visual System

## Branch

```txt
feat/11-seat-map-ui
```

## Goal

สร้าง seat UI ก่อนต่อ concurrency

## Tasks

- aircraft seat grid
- aisle
- row number
- cabin separation
- legend
- responsive seat map
- seat hover
- select
- disabled
- booked
- held
- selected summary
- cabin filter

## Accessibility

แต่ละ seat:

```txt
Business Seat 3A, Available
```

---

# 52. BRANCH 12 — Seat Hold & Concurrency UX

## Branch

```txt
feat/12-seat-concurrency-ui
```

## Tasks

- connect hold API
- countdown
- hold refresh state
- conflict response
- seat was taken message
- expired hold
- release seat
- optimistic UI with server confirmation
- polling/revalidation strategy

## Important

Backend remains source of truth

Frontend selection ไม่ถือว่า booked จน server confirm

---

# 53. BRANCH 13 — Passenger Flow

## Branch

```txt
feat/13-passenger-flow
```

## Tasks

- passenger form
- contact form
- validation
- multi-passenger support
- nationality
- passport
- DOB
- error motion
- progress stepper
- booking summary sidebar

---

# 54. BRANCH 14 — Booking Review

## Branch

```txt
feat/14-booking-review
```

## Tasks

- complete itinerary
- passengers
- seat
- cabin
- fare
- cancellation policy
- edit links
- final confirmation
- responsive summary

---

# 55. BRANCH 15 — Mock Payment Experience

## Branch

```txt
feat/15-mock-payment-ui
```

## Tasks

- payment method cards
- credit card mock
- debit card mock
- bitcoin mock
- processing state
- success/failure simulation
- no-real-charge disclosure
- prevent accidental sensitive-data retention

## Motion

- method card Flip
- payment processing
- success line animation

---

# 56. BRANCH 16 — Booking Success + E-Ticket QR

## Branch

```txt
feat/16-ticket-qr-experience
```

## Tasks

- success scene
- booking reference reveal
- e-ticket
- QR code
- ticket verification state
- print CSS
- PDF/download strategy
- copy booking reference
- manage booking CTA

## Motion

- ticket reveal
- QR fade
- confirmation animation

---

# 57. BRANCH 17 — Manage Booking

## Branch

```txt
feat/17-manage-booking-ui
```

## Tasks

- Booking Reference
- Last Name
- optional Email
- lookup result
- ticket preview
- payment state
- flight state
- seat state
- cancellation eligibility

---

# 58. BRANCH 18 — Cancellation & Refund UX

## Branch

```txt
feat/18-cancellation-refund-ui
```

## Tasks

- >= 24 hour validation display
- eligible state
- ineligible state
- confirmation modal
- refund 100%
- ticket cancel state
- seat release state
- receipt/status

---

# 59. BRANCH 19 — Admin Login + Admin Shell

## Branch

```txt
feat/19-admin-shell
```

## Tasks

- admin login
- secure session UI
- protected route UX
- sidebar
- topbar
- admin search
- responsive admin shell
- logout
- expired session state

Admin motion restrained

---

# 60. BRANCH 20 — Admin Flight Management

## Branch

```txt
feat/20-admin-flight-management
```

## Tasks

- list flights
- create flight
- edit flight
- cancel flight
- fare by cabin
- schedule
- route
- aircraft
- seat capacity
- validation
- confirmation
- activity feedback

---

# 61. BRANCH 21 — Admin Booking & Ticket Management

## Branch

```txt
feat/21-admin-booking-ticketing
```

## Tasks

- booking table
- filters
- passenger search
- booking detail
- ticket view
- QR verification
- print
- cancellation status
- refund status

---

# 62. BRANCH 22 — Analytics Dashboard

## Branch

```txt
feat/22-admin-analytics
```

## Required Analytics

- Revenue
- Bookings
- Passengers
- Load Factor
- Cancellation Rate
- Most Booked Flights
- Least Booked Flights
- Nationality
- Daily
- Weekly
- Monthly
- Custom range

## Charts

Recharts

## Motion

- KPI count-up
- chart entrance
- filter transition
- no excessive looping

## Profit

MVP:

```txt
Revenue
```

Profit only if cost data exists

---

# 63. BRANCH 23 — External API Client UI

## Branch

```txt
feat/23-integration-admin-ui
```

## Tasks

- API Client list
- create client
- client ID
- secret reveal-once UX
- scopes
- revoke
- disable
- regenerate
- API docs link

---

# 64. BRANCH 24 — Security & Company Device UX

## Branch

```txt
feat/24-security-hardening-ui
```

## Tasks

- session timeout UX
- unauthorized page
- device/network restriction messaging
- IP restriction support where backend provides
- secure headers verification
- no sensitive client storage
- admin audit visibility if available

Production strategy:

```txt
Admin Login
+
Secure Cookie
+
IP/VPN restriction when deployed
```

---

# 65. BRANCH 25 — Accessibility & Reduced Motion

## Branch

```txt
chore/25-accessibility-motion
```

## Tasks

- keyboard audit
- focus audit
- contrast
- seat labels
- ARIA
- reduced motion
- screen reader forms
- skip navigation
- modal focus trap
- chart text alternatives

---

# 66. BRANCH 26 — Responsive & Mobile Motion

## Branch

```txt
feat/26-responsive-motion
```

## Tasks

- desktop
- laptop
- tablet
- mobile
- small mobile

เปลี่ยน heavy desktop interaction เป็น mobile-native interaction

เช่น:

```txt
Horizontal Pin → Vertical Cards
Heavy Parallax → Fade/Slide
Large Seat Canvas → Zoomable/scrollable seat area
```

---

# 67. BRANCH 27 — Performance Optimization

## Branch

```txt
perf/27-frontend-performance
```

## Tasks

- bundle analysis
- dynamic import
- GSAP scope audit
- ScrollTrigger cleanup
- image optimization
- video optimization
- font loading
- preload critical assets
- lazy load
- reduce JS
- avoid hydration mismatch
- Core Web Vitals audit

---

# 68. BRANCH 28 — Cross-Browser QA

## Branch

```txt
test/28-cross-browser-qa
```

## Browsers

- Chrome
- Safari
- Firefox
- Edge

## Devices

- macOS
- Windows
- iOS Safari
- Android Chrome

## Tests

- scroll
- sticky
- date picker
- seat map
- payment mock
- print ticket
- admin charts

---

# 69. BRANCH 29 — End-to-End Booking Experience

## Branch

```txt
test/29-booking-e2e
```

Playwright scenarios:

```txt
Search Flight
→ Select Flight
→ Select Cabin
→ Hold Seat
→ Passenger
→ Payment
→ Ticket
```

รวม:

- seat conflict
- expired hold
- payment failed
- cancellation
- manage booking

---

# 70. BRANCH 30 — Deployment Preparation

## Branch

```txt
chore/30-deployment-prep
```

## Frontend

Vercel

## Backend

Render Docker

## Tasks

- environment variables
- CORS
- API URL
- production asset URLs
- error pages
- logging
- analytics disabled/enabled decision
- health checks
- production build
- source maps policy

---

# 71. BRANCH 31 — Production Deployment

## Branch

```txt
chore/31-production-deploy
```

## Architecture

```txt
Vercel
  ↓
Next.js

Render
  ↓
Rust + Axum Docker
  ↓
Managed PostgreSQL
```

## Tasks

- deploy frontend
- deploy backend
- production DB
- migrations
- seed
- backup
- smoke test
- verify QR URLs
- admin access restriction
- HTTPS
- domain

---

# 72. BRANCH 32 — Final Polish

## Branch

```txt
feat/32-final-polish
```

## Tasks

- spacing polish
- typography polish
- animation timing
- image color grade
- transitions
- empty states
- error states
- loading states
- microinteraction
- copy cleanup
- responsive final pass

---

# 73. BRANCH 33 — Documentation & Presentation

## Branch

```txt
docs/33-final-documentation
```

## Deliverables

```txt
README.md
REQUIREMENTS.md
DESIGN.md
TECH_STACK.md
ARCHITECTURE.md
FRONTEND.md
BACKEND.md
DATABASE.md
API.md
DEPLOYMENT.md
```

Presentation assets:

- architecture diagram
- booking flow
- seat concurrency diagram
- motion design explanation
- admin analytics
- external API flow
- deployment diagram

---

# 74. Branch Dependency Order

```txt
00 Design Discovery
↓
01 Design Foundation
↓
02 Motion Foundation
↓
03 Global Shell
↓
04 Hero
↓
05 Scroll Storytelling
↓
06 Horizontal Journey
↓
07 Global Network / Moon
↓
08 Flight Search
↓
09 Flight Results
↓
10 Flight Detail
↓
11 Seat Map
↓
12 Seat Concurrency
↓
13 Passenger
↓
14 Review
↓
15 Payment
↓
16 Ticket QR
↓
17 Manage Booking
↓
18 Cancellation
↓
19 Admin Shell
↓
20 Flight Management
↓
21 Booking/Ticket Management
↓
22 Analytics
↓
23 Integration UI
↓
24 Security
↓
25 Accessibility
↓
26 Responsive
↓
27 Performance
↓
28 Browser QA
↓
29 E2E
↓
30 Deployment Prep
↓
31 Deploy
↓
32 Final Polish
↓
33 Documentation
```

---

# 75. Git Merge Strategy

แต่ละ branch:

```txt
main
  ↑
feature branch
```

Workflow:

```txt
git checkout main
git pull
git checkout -b feat/xx-name
```

หลังทำเสร็จ:

```txt
lint
typecheck
test
build
manual QA
review
merge
```

ไม่รวมหลาย feature ใหญ่ใน branch เดียว

---

# 76. Definition of Done — UI Branch

ทุก UI branch ต้องผ่าน:

- Design matches system
- Desktop complete
- Mobile complete
- Keyboard usable
- Reduced motion checked
- No console errors
- No animation leaks
- TypeScript clean
- Lint clean
- Production build passes

---

# 77. Definition of Done — Motion Branch

ต้องตรวจ:

- 60fps target
- no ScrollTrigger duplicate
- no stale GSAP timeline
- resize safe
- mobile safe
- reduced motion safe
- page navigation cleanup
- animation does not block interaction

---

# 78. Important Design Decisions

## Use

```txt
GSAP
ScrollTrigger
Flip
Lenis
Motion
SplitType
Tailwind
shadcn/ui
Recharts
```

## Optional

```txt
Rive
Lottie
Three.js
WebGL
```

## Do Not Force

ไม่ใช้ library เพียงเพราะมันทำ animation ได้

เลือกตาม responsibility

---

# 79. What Makes X-Fly “Wow”

ความว้าวต้องมาจากการรวมกันของ:

1. Cinematic first impression
2. Strong typography
3. Aircraft media
4. Scroll storytelling
5. Smooth section choreography
6. Premium booking transitions
7. Cinema-like seat selection
8. Seat hold countdown
9. Payment completion sequence
10. E-Ticket reveal
11. QR ticket
12. Dark + Yellow identity
13. Analytics ที่ดูเหมือน production dashboard

ไม่ใช่แค่ “ใส่ animation เยอะ”

---

# 80. Final Frontend / Motion Stack

```txt
Next.js
React
TypeScript

Tailwind CSS
shadcn/ui

GSAP
@gsap/react
ScrollTrigger
Flip

Lenis
Motion
SplitType

React Hook Form
Zod
TanStack Query
Recharts
```

Optional:

```txt
Rive
Lottie
Three.js
```

---

# 81. Final System Context

Frontend experience นี้ทำงานอยู่บน architecture หลักของ X-Fly:

```txt
Next.js / React
      ↓
REST API
      ↓
Rust + Axum
      ↓
SQLx
      ↓
PostgreSQL
```

Local:

```txt
Next.js local
Rust local
PostgreSQL Docker
```

Production:

```txt
Vercel
   ↓
Render Rust Docker
   ↓
Managed PostgreSQL
```

---

# 82. Final Recommendation

เป้าหมายสุดท้ายไม่ใช่:

> “สร้างเว็บจองตั๋วเครื่องบินที่ใช้งานได้”

แต่คือ:

> **“สร้าง Digital Airline Experience ที่ใช้ Motion และ Visual Storytelling เป็นส่วนหนึ่งของ Booking Journey”**

หน้า Landing ต้องทำให้ผู้ใช้หยุดดู

หน้า Search ต้องรู้สึกเร็ว

หน้า Seat ต้องรู้สึก interactive

หน้า Payment ต้องมั่นใจ

หน้า Ticket ต้องน่าจดจำ

หน้า Admin ต้องดูเป็นระบบจริง

และ Motion ทุกจุดต้องสนับสนุน UX ไม่ใช่แย่งความสนใจจาก UX

---

# 83. Recommended First Implementation Sequence

หลังทีม approve เอกสารนี้:

```txt
1. docs/00-design-discovery
2. feat/01-design-foundation
3. feat/02-motion-foundation
4. feat/03-global-shell
5. feat/04-cinematic-hero
```

หลัง Branch 04 ควรหยุด Review รอบใหญ่ก่อน

เพราะ 4 branch แรกจะเป็นตัวกำหนดคุณภาพทาง visual ของทั้งระบบ

ถ้า Hero + Motion Foundation ยังไม่ถึงระดับที่ทีมต้องการ **อย่ารีบทำ booking pages ต่อ**

ให้แก้ design language ให้แข็งแรงก่อน

---

# 84. Design Review Checkpoints

## Checkpoint A

หลัง Branch 04

Review:

- Brand
- Typography
- Hero
- Motion quality

## Checkpoint B

หลัง Branch 07

Review:

- Full landing page
- Scroll experience
- Performance

## Checkpoint C

หลัง Branch 12

Review:

- Search
- Flight
- Cabin
- Seat experience

## Checkpoint D

หลัง Branch 18

Review:

- Complete customer booking journey

## Checkpoint E

หลัง Branch 23

Review:

- Admin + integration

## Checkpoint F

หลัง Branch 32

Final production review

---

# 85. Success Criteria

X-Fly Anyway ถือว่าประสบความสำเร็จเมื่อ:

- Landing page มี wow factor
- Motion ลื่นและไม่รบกวน UX
- User จองได้โดยไม่ Login
- Search → Ticket flow ต่อเนื่อง
- Seat concurrency ปลอดภัย
- Payment เป็น mock อย่างชัดเจน
- Ticket มี QR
- Cancellation 24 ชั่วโมงทำงานถูกต้อง
- Admin จัดการ flight ได้
- Dashboard มี analytics ที่ใช้ประโยชน์ได้
- External API มี token/scope
- Mobile ใช้งานได้ดี
- Reduced motion รองรับ
- Production build เร็วและเสถียร
