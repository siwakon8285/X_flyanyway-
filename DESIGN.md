# X-Fly Anyway — DESIGN.md

> **Design & Frontend Experience Master Plan**
>
> เป้าหมายของเอกสารนี้คือกำหนดทิศทาง Visual Design, Motion System, Page Experience, Component Architecture และแผนการพัฒนาแบบแยก Branch ตั้งแต่เริ่มต้นจนถึง Production-ready สำหรับ X-Fly Anyway
>
> เอกสารนี้เน้นว่า X-Fly Anyway **ต้องไม่ออกมาเป็นเว็บ Booking ธรรมดา** แต่ต้องเป็น Airline Booking Experience ที่มีความ cinematic, premium, editorial, immersive และมี motion ที่สร้างความ “ว้าว” ตั้งแต่หน้าแรกจนถึงตอนออก E-Ticket

---

# 1. Product Vision

X-Fly Anyway คือ **Airline Booking & Ticketing Platform** สำหรับสายการบินระดับโลกที่ให้บริการจองผ่านเว็บไซต์เท่านั้น โดยมุ่งเน้นลูกค้าที่มีกำลังซื้อสูงและคาดหวังประสบการณ์ที่เร็ว พรีเมียม และมั่นใจได้.

Brand / business direction:

- Airline: **X-Fly Anyway**
- Signature color: **Yellow / `#FFD400`** ตามโจทย์แบรนด์ของบริษัทและ stakeholder brand rationale
- Current network story: **156 countries worldwide**
- Future brand vision: **Moon route next year** — ใช้เป็น marketing / storytelling vision จนกว่าจะมี requirement ให้เป็น sellable inventory จริง
- Target customer: premium / high-purchasing-power travelers
- Customer booking channel: **Web only**
- Customer account: **No registration/login required**
- Booking frequency: no account-based booking limit; one customer may create multiple independent bookings
- Customer recovery / post-booking access: **Booking Reference + Last Name**
- Primary payment: **Card**; current academic implementation uses **Stripe Test Mode Card**
- Alternative demonstration: **Mock Bitcoin** only; no blockchain/live crypto settlement
- Cancellation: eligible when departure is at least 24 hours away, **0 fee / 100% refund**
- Flight change: cancel the existing eligible booking and create a new booking; no direct rebooking/change-flight engine in the MVP
- E-ticket: accessible through the website after Manage Booking verification
- External systems: consume authorized X-Fly data through scoped API tokens

## 1.1 Latest Stakeholder Requirement Override — 07 Sep 2026

The following requirements are the **current authoritative customer-product scope** and override older historical branch descriptions elsewhere in this document when they conflict:

- **Sellable cabins for all new customer bookings: Business and First only.**
- `Economy` and `Premium Economy` are legacy/historical values only. Existing records may remain readable for audit/history, but customer search, availability, fare, seat-hold, review, and new-booking APIs must not offer or accept them for new bookings.
- Passenger gender is **required** and new bookings accept **Male / Female only**. Do not offer `Unspecified` in the customer booking flow.
- The active booking contact is **phone-only**: phone country code + phone number. Customer email is not required or collected for new bookings unless a later stakeholder requirement explicitly restores it.
- X-Fly must **not send customer booking-confirmation email** in the active product scope. Booking success remains authoritative without email, and customers return through the website `Manage Booking` entry using **Booking Reference + Last Name**.
- The standalone **Travel Extras** step/page is removed from the active customer flow. Do not sell or select optional baggage, meal, assistance, or ancillary extras in this MVP. Cabin-included baggage/service benefits may still be displayed as included benefits where truthful.
- Existing historical database rows, legacy enum values, and prior completed branch history must not be destructively rewritten merely to match the new UI scope.
- Implementation belongs in **Branch 20A — Customer Requirements Alignment**, after Branch 20 closes and before Branch 21 begins.

This override exists to keep the roadmap and future coding-agent work aligned with the latest stakeholder direction without falsifying earlier implementation history.

Product boundary:

> X-Fly owns the **booking, payment, ticket, customer retrieval, cancellation/refund, internal management, reporting, and external-data API** domains.

X-Fly does **not** implement airport operational systems such as customer online check-in, gate processing, staff QR-scanning workflow, baggage loading workflow, or boarding-pass lifecycle. Those are downstream systems that may consume authorized X-Fly booking/ticket data.

The website must feel like a premium global airline experience rather than a generic booking form. The public journey can be cinematic, but transactional pages must prioritize clarity, response speed, accessibility, and trust.

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

## 8.1 Customer Booking Lifecycle

```txt
Landing
  ↓
Flight Search
  ↓
Flight Results
  ↓
Flight Detail + Cabin + Fare Conditions
  ↓
Cinema-style Seat Selection
  ↓
Passenger Details + Travel Documents + Phone Contact
  ↓
Booking Review
  ↓
Payment — Stripe Test Mode Card / Mock Bitcoin
  ↓
Booking Confirmed
  ↓
Final Booking Summary
  ↓
END OF PRIMARY BOOKING FLOW
```

The post-payment success page is a **final booking summary**, not the long-term E-ticket management surface. It must not require the customer to continue through additional lifecycle pages.

## 8.2 Returning Customer / E-Ticket Lifecycle

```txt
Website Navigation → Manage Booking
                         ↓
                  /manage-booking
                         ↓
                    Booking Reference
                         + Last Name
                         ↓
                    verified access
                         ↓
              /manage-booking/details
                         ↓
           Booking Details / E-Ticket
                         ↓
              Verification QR available
```

The active product does **not** depend on customer email delivery. The website navigation must expose a clear **Manage Booking** entry, and the customer must retain the Booking Reference shown on the final Booking Confirmed summary.

## 8.3 Cancellation Lifecycle

```txt
Manage Booking Details
  ↓
Cancellation eligibility
  ├── >= 24h before departure → eligible
  │       ↓
  │    Cancel booking
  │       ↓
  │    100% refund / 0 fee
  │
  └── < 24h before departure → unavailable
```

If a customer wants another flight, the project rule is:

```txt
eligible cancellation
  ↓
cancel old booking
  ↓
new booking flow
```

No direct change-flight/rebooking engine is required.

## 8.4 Internal X-Fly Lifecycle

```txt
Employee Authentication
  ↓
RBAC / Company-device policy
  ↓
Role-specific X-Fly workspace
  ├── Executive analytics / reports
  ├── Flight management
  ├── Booking management
  ├── Ticket / passenger operations
  └── API client / token administration
```

A generic administrator does not automatically receive permission to create or edit flights. Flight mutation belongs to the responsible **Flight Manager** role.

## 8.5 External System Lifecycle

```txt
External System
  ↓
API Client Credential / Token
  ↓
Scope authorization
  ↓
X-Fly REST API
  ↓
Only approved booking / flight / passenger / ticket data
```

Examples include baggage, ticketing, operational, or marketing-analysis systems. External systems must never obtain direct database access.

## 8.6 Operational Boundary — Airport Check-in / Boarding

Airport check-in, staff QR-scanning applications, gate operations, baggage-loading workflow, and boarding-pass generation are **outside the X-Fly implementation scope**.

The project presentation may explain the downstream scenario:

```txt
Customer opens X-Fly E-Ticket / QR
  ↓
Airport / airline operational staff verify it using their own system
  ↓
Their operational system records check-in / boarding state
```

X-Fly only provides authoritative booking/ticket data and authorized integration interfaces. It does not implement that downstream workflow.

## 8.7 Realism Boundary

X-Fly models the parts required by the booking/ticketing requirement:

- passenger identity and travel-document details
- phone contact information
- cabin-included baggage/service allowance
- Business/First cabin and cinema-style seat selection
- fare conditions
- cancellation/refund rules
- fare / taxes / fees breakdown
- separate booking, payment, and ticket concepts
- separate Booking Reference and Ticket Number
- E-ticket access and signed verification QR
- internal analytics / reports
- RBAC
- scoped external API access

Do not implement:

- real GDS
- real DCS / airport check-in
- staff QR scanner
- gate / boarding workflow
- boarding-pass lifecycle
- government APIS transmission
- visa / Timatic engine
- real baggage handling
- chatbot / customer-support module
- loyalty / points
- promotion/campaign builder
- live cryptocurrency
- live Stripe charging for the university/demo environment

Marketing systems may receive authorized aggregate/required booking data through the external API, but campaign creation belongs outside X-Fly.

## 8.8 Booking / Payment / Ticket Lifecycle

These concepts remain separate and must not be collapsed:

```txt
Booking
PENDING / CONFIRMED / CANCELLED

Payment
PENDING / PAID / FAILED / REFUNDED

Ticket
NOT_ISSUED / ISSUED / CANCELLED
```

Conceptually:

```txt
Booking created
  ↓
Payment processing
  ↓
Payment confirmed
  ↓
Ticket issued
  ↓
Booking confirmation shown
```

A failed payment must never imply an issued ticket. Cancellation/refund must update each domain according to its own authoritative state.

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

ต้องรู้สึกเหมือน product showcase และเป็นจุดที่ผู้ใช้เข้าใจว่า “กำลังซื้ออะไร” ก่อนเลือกที่นั่ง

ใช้:

- route timeline
- aircraft identity
- cabin imagery / cabin selector
- amenities
- seat availability context
- cabin price
- baggage allowance summary
- fare conditions summary
- cancellation/refund policy summary
- fare breakdown preview where useful

Active sellable cabin tabs:

```txt
Business
First
```

`Economy` and `Premium Economy` may remain readable in historical records, but they are not selectable or accepted for new customer bookings.

Fare context ต่อ cabin ควรอธิบายอย่างน้อย:

- sample fare per passenger
- checked baggage allowance
- cabin baggage allowance
- seat-selection inclusion / availability
- changeability
- refundability / cancellation policy

ห้ามสร้าง fare engine จริงใน frontend; ใช้ typed fixture/presentation metadata จนกว่า backend จะเป็น source of truth.

Cabin switching ใช้ Motion/GSAP ตาม architecture เดิม แต่ booking UX ต้องเร็วและ user-controlled.

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
```

Branch 11 local UI states:

```txt
Available
Selected
Booked
Unavailable
```

`Held` belongs to Branch 12 and must not be simulated in Branch 11.

Motion:

- seat hover scale / lift
- selected seat glow
- short pop + settle
- row stagger reveal
- selected-seat summary transition
- no playful bounce or random shake

Seat geometry must differ visibly between the two active sellable cabins, not only by color:

- Business — large pod/privacy-shell seat with low density
- First — private-suite visual with the largest footprint and fewest seats

Legacy Economy/Premium Economy geometry may remain only where required to render historical records; it must not appear in the active new-booking seat-selection flow.

Optional:

- subtle aircraft cabin background
- ambient lighting effect

Accessibility:

- native semantic seat buttons preferred
- visible seat code / row number
- available / selected / booked / unavailable conveyed without color alone
- keyboard Enter/Space support
- visible focus ring
- clear accessible seat names

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

หลัง cinematic sections ต้องกลับมาเป็น UX ที่นิ่ง ชัด และเหมือน airline passenger-information flow จริง

ใช้:

- large step labels
- floating / sticky booking summary where appropriate
- clear validation
- progressive disclosure
- animated error feedbackแบบ restrained
- temporary browser state only where safe and appropriate

## Passenger Identity

สำหรับผู้โดยสารแต่ละคน รองรับอย่างน้อย:

- Title: Mr / Ms / Mrs / Mx
- Given name
- Middle name (optional)
- Family name / Surname
- Date of birth
- Gender — **Male / Female only; required for new bookings**
- Nationality

ต้องมีข้อความชัดเจน:

```txt
Passenger name must match the travel document exactly.
```

## Travel Document / Passport

สำหรับ international booking ให้รองรับ:

- Passport number
- Issuing country
- Expiry date
- Issue date optional if useful

ระบบ academic MVP สามารถกำหนดให้ passport information ถูกกรอกใน Passenger Details เพื่อให้ booking flow สมบูรณ์และตรวจสอบได้ แม้สายการบินจริงบางแห่งจะอนุญาตให้เพิ่มเอกสารภายหลัง.

Do not implement government APIS transmission, visa checking, or Timatic.

## Contact Details

Booking/contact details for new customer bookings:

- Phone country code
- Phone number

Email is not collected in the active customer flow because customer confirmation email has been removed from scope and Manage Booking uses Booking Reference + Last Name. Existing historical email values may remain stored/readable where required; do not invent placeholder email addresses to satisfy legacy schema constraints.

Optional Emergency Contact:

- Name
- Relationship
- Phone country code
- Phone number

## Multi-passenger

- รองรับผู้โดยสารหลายคน
- seat assignment ต้อง map ไปยัง passenger ได้ใน later booking state
- infant seat rules ต้องเป็น business rule ที่ชัดเจน; current UI may treat under-2 infants as lap infants only where explicitly approved

## Stepper

```txt
01 Flight
02 Seat
03 Passenger
04 Review
05 Payment
06 Booking Confirmed
```

## Validation

- required fields
- date consistency
- passport expiry validity against travel date at the level defined by project rules
- duplicate/malformed passenger data
- accessible error summary / field messages

Do not store raw sensitive travel-document data in insecure client persistence.

---

# 19A. Travel Extras — Removed from Active Scope

The latest stakeholder requirement removes the standalone Travel Extras step/page from the customer booking flow.

Active rules:

- no optional extra-baggage purchase/selection
- no meal-preference selection
- no special-assistance selection in this MVP flow
- no ancillary pricing fixture or extras total
- `/booking/extras` must not remain a required navigation step
- old/direct extras URLs should fail or redirect safely according to the Branch 20A implementation plan
- Business/First cabin-included benefits such as baggage allowance or included meal/service may still be displayed as **included**, not as optional paid extras

The historical Branch 13A implementation may remain in Git history, but active customer state, Review, Payment, E-Ticket, and future documentation must not require selected extras.

---

# 20. Booking Review

ใช้ large editorial summary และทำหน้าที่เป็นจุดตรวจสอบครั้งสุดท้ายก่อน payment

ซ้าย:

- itinerary / route
- date / time
- aircraft
- cabin
- passengers
- passenger travel-document completeness/status summary
- seats
- cabin-included baggage/service benefits

ขวา:

- base fare
- taxes
- airport / passenger charges (fixture)
- seat fee if any
- total
- cancellation policy
- fare conditions

ตัวอย่าง fare breakdown:

```txt
Fare                         THB 52,000
Airport / passenger charges  THB 4,800
Taxes                        THB 6,400
Seat                         Included
Baggage                      Included
--------------------------------------
TOTAL                        THB 63,200
```

ข้อมูล tax/fee ใน MVP ต้องระบุว่าเป็น demo fixture หากไม่ได้มาจาก backend/live pricing.

## Fare Conditions

ต้องมี summary อย่างน้อย:

- refundable / cancellation eligibility
- change allowed or not
- change fee if modeled
- baggage allowance
- seat-selection inclusion

## Final Acknowledgements

ก่อน payment ให้ผู้ใช้ยืนยันอย่างน้อย:

```txt
[ ] I confirm that passenger names match their travel documents.
[ ] I have reviewed the fare conditions and cancellation policy.
```

CTA:

```txt
CONTINUE TO PAYMENT
```

ต้องไม่อนุญาตให้ไป payment หาก acknowledgement ที่ required ยังไม่ครบ.

---

# 21. Payment Experience — Stripe Test Mode + Mock Bitcoin

X-Fly ใช้ payment architecture แบบ **academic/demo only** โดยไม่รับเงินจริง.

Current supported methods:

```txt
Card — Stripe Test Mode via Stripe Payment Element
Bitcoin — X-Fly Mock Payment
```

## Card — Stripe Test Mode

Card payment ใช้ Stripe Test Mode จริงแทน raw mock-card form เดิม.

Frontend:

- ใช้ `@stripe/stripe-js`
- ใช้ `@stripe/react-stripe-js`
- Stripe Payment Element เป็นผู้ครอบครองช่องกรอกข้อมูลบัตร
- X-Fly frontend ไม่มี raw PAN / CVC / expiry fields
- ไม่ persist `client_secret`
- ไม่แสดง raw Stripe status/error ให้ customer

Backend:

- Rust + Axum เป็นผู้สร้าง/retrieve/cancel Stripe PaymentIntent
- จำนวนเงินและ currency มาจาก authoritative X-Fly Review/payment snapshot ฝั่ง server
- Stripe amount ใช้ THB whole-baht → minor unit ×100 โดยไม่ใช้ floating point
- local payment state เป็น source of truth สำหรับ X-Fly UI
- Stripe webhook + reconciliation สามารถ finalize payment ได้แบบ idempotent
- successful payment finalization เป็นผู้ BOOK seat และ consume hold แบบ atomic
- Ticket branch ต้องไม่ finalize inventory ซ้ำ

Current payment statuses:

```txt
CREATED
PROCESSING
AWAITING_PAYMENT
SUCCEEDED
FAILED
CANCELLED
```

Stable customer-safe failure codes include:

```txt
CARD_DECLINED
AUTHENTICATION_FAILED
PROCESSING_ERROR
PAYMENT_CANCELLED
```

Manual QA ที่ยืนยันแล้ว:

- normal Stripe Test card success
- declined card
- 3DS / requires-action COMPLETE
- Stripe Dashboard correlation
- real test webhook delivery
- webhook persistence/replay safety
- PaymentIntent retrieval
- seat BOOKED / hold consumed on success
- no seat booking / no hold consumption on declined payment

## Bitcoin Mock

Bitcoin ยังคงเป็น mock-only flow:

- Mock wallet / address
- Mock amount / demo conversion
- Simulate receive / fail / cancel
- ไม่มี blockchain จริง
- ไม่มี Stripe crypto
- ไม่มีเงินจริง

ต้องมี disclosure ชัดเจนว่าเป็น demo/mock.

## Local Stripe Development

เวลาทดสอบ Stripe webhook ใน Local ให้เปิด 3 process:

```txt
Frontend
npm run dev

Backend
cargo run

Stripe CLI
stripe listen --forward-to localhost:8080/api/v1/payments/stripe/webhook
```

Stripe CLI, frontend `pk_test_...`, และ backend `sk_test_...` ต้องอยู่ใน **Stripe sandbox/account context เดียวกัน**.

`STRIPE_WEBHOOK_SECRET` ใน Local มาจาก `stripe listen`.

## Self-Hosted Demo Server

งานนี้ใช้ **Stripe Test Mode เท่านั้น** แม้ deploy ขึ้น Ubuntu Server.

บน server:

- ใช้ `pk_test_...`
- ใช้ `sk_test_...`
- ไม่ใช้ live keys
- ไม่ต้องรัน `stripe listen`
- สร้าง Stripe Test/Sandbox webhook endpoint ให้ชี้ public HTTPS URL ของ X-Fly เช่น:

```txt
https://<public-demo-host>/api/v1/payments/stripe/webhook
```

จากนั้นใช้ `whsec_...` ของ webhook endpoint ฝั่ง server เป็น `STRIPE_WEBHOOK_SECRET`.

Security rules:

- never commit Stripe secrets
- backend secrets อยู่ backend/server environment เท่านั้น
- `NEXT_PUBLIC_STRIPE_PUBLISHABLE_KEY` เป็น publishable Test Mode key เท่านั้น
- never persist raw card data
- never log Stripe secrets / client_secret / payment PII
- no live payment scope for university/demo deployment

---

# 22. Booking Confirmation Experience

The page currently reached after successful payment/ticket issuance is the **end of the primary booking flow**.

Recommended customer-facing title:

```txt
Booking Confirmed
```

Alternative brand copy may be:

```txt
Your Journey is Confirmed
```

Do not present this page as the long-term Manage Booking/E-Ticket portal.

## Required Summary

Show only the final booking essentials:

- booking confirmed state
- flight number
- origin / destination
- departure / arrival
- passenger name(s)
- cabin
- seat(s)
- Booking Reference
- Ticket Number
- amount paid / currency
- payment successful state

Recommended actions:

```txt
Print Summary
Return Home
```

A visible Manage Booking entry may remain in the global navigation, but the post-payment flow must not force the user into another long page.

## QR Boundary

**Do not show the ticket verification QR on the Booking Confirmed summary page.**

The verification QR belongs to the authenticated/retrieved **E-Ticket / Manage Booking Details** experience. This keeps the final purchase confirmation concise and separates booking completion from later ticket retrieval.

## No Customer Confirmation Email

The latest stakeholder requirement removes customer booking-confirmation email from the active product scope.

After successful payment/ticket issuance:

- payment success remains final according to authoritative backend state
- booked inventory / consumed hold / issued ticket remain unchanged
- the customer receives the Booking Reference and Ticket Number on the Booking Confirmed summary
- later retrieval uses Website → Manage Booking → Booking Reference + Last Name
- no new customer confirmation-email delivery should be required or triggered for new bookings

Historical Branch 17A email/outbox/provider code and persisted delivery records must be retired or disabled safely in Branch 20A without destructive DEV-data cleanup.

## State Separation

Do not imply that payment success and ticket issuance are the same state. The UI may summarize them together after both are authoritative, while the backend keeps their lifecycles separate.

# 23. E-Ticket / Online Ticket Design

The E-Ticket is the **returning-customer ticket surface**, reached after Manage Booking verification. It is not the immediate post-payment summary page.

E-Ticket / Booking Details should include:

- X-Fly branding
- Booking Reference
- Ticket Number
- Ticket Status
- Passenger name(s)
- Flight
- Route
- Cabin
- booked seat(s)
- departure / arrival date-time
- cabin-included baggage/service summary
- payment summary/status
- cancellation status/eligibility
- signed verification QR

Booking Reference and Ticket Number are separate customer identifiers.

Current canonical Booking Reference:

```txt
XF + 8 unambiguous uppercase characters
example shape: ^XF[A-Z2-9]{8}$
```

Do not rely on older six-character examples.

Ticket Number must remain a separate server-generated customer-facing identifier. Do not expose Stripe PaymentIntent IDs or database UUIDs as the customer-facing ticket number.

## QR

QR should encode the signed verification URL/token created by the backend.

Requirements:

- no raw passenger PII in QR payload
- no passport/email/phone/payment secrets
- tamper-resistant signature
- public verification returns only minimal safe ticket facts
- QR is an **E-Ticket verification artifact**, not a Boarding Pass

## Print

The E-Ticket / booking details view should print cleanly from a browser where useful.

X-Fly does not implement the downstream airport Boarding Pass workflow. If an operational system later needs to print a Boarding Pass, it must obtain authorized data through an approved integration boundary rather than treating this E-Ticket QR as a boarding-pass barcode.

# 24. Manage Booking

Manage Booking is the secure returning-customer entry point.

## Public Entry

Route:

```txt
/manage-booking
```

This route must always present a lookup experience rather than silently showing a booking from an old authorization cookie.

Required fields:

```txt
Booking Reference
Last Name
```

Email is **not** a lookup credential in the latest product flow.

Lookup credentials must be sent in a POST body, not placed in URLs.

Generic lookup failure:

```txt
We couldn't find a booking with those details.
```

The public error must not disclose whether the reference or surname was incorrect.

## Successful Lookup

On successful verification:

```txt
POST lookup
  ↓
short-lived signed HttpOnly Manage Booking authorization
  ↓
/manage-booking/details
```

The authorization context must be scoped to the exact booking/ticket verified by the latest successful lookup.

### Stale-context invariant

A stale Manage Booking cookie must never cause Booking A to display when the customer just entered the credentials for Booking B.

Required behavior:

- successful lookup replaces/refreshes the scoped authorization
- details are loaded from the exact authorized ticket/booking identity
- `/manage-booking` does not auto-render the previously authorized booking
- direct `/manage-booking/details` without valid authorization is rejected safely
- cross-booking authorization is rejected
- changing bookings requires a new lookup

This invariant is mandatory because manual QA found a stale-context defect where the displayed travel date/seat belonged to a different previous booking.

## Booking Details / Online Ticket

After verification, show the complete customer booking information in a compact, structured E-Ticket/Booking Details experience:

- booking status
- payment status
- ticket status
- flight / route / authoritative date-time
- passenger name(s)
- travel-document completeness only; mask sensitive values
- cabin
- finalized seats
- cabin-included baggage/service benefits
- amount/currency
- Booking Reference
- Ticket Number
- signed E-Ticket verification QR
- cancellation eligibility / status

Do **not** show:

- customer online check-in panel
- boarding-pass CTA
- airport operational state
- raw passport number
- Stripe provider reference
- client secret
- hold/session tokens

## Navigation

Customers must be able to reach Manage Booking through the X-Fly website:

```txt
X-Fly Website Navigation → Manage Booking
```

The flow must not depend on confirmation email. Desktop should use clear text such as `Manage Booking`; do not rely on an ambiguous icon alone.

## Privacy

Do not display full passport numbers or unnecessary contact/payment data. Use `Cache-Control: no-store, private` where appropriate for authenticated booking detail responses.

# 25. Cancellation UX

Cancellation remains a core X-Fly customer feature.

Project rule:

```txt
departure >= 24 hours away
→ cancellation allowed
→ cancellation fee = 0
→ refund = 100%

departure < 24 hours away
→ cancellation unavailable
```

The exact departure instant must use authoritative origin timezone data.

If eligible, Branch 18 will implement:

1. verified Manage Booking authorization for the exact booking
2. server-side eligibility revalidation
3. explicit confirmation
4. booking cancellation
5. ticket cancellation
6. inventory release according to authoritative backend rules
7. full refund state
8. customer-safe receipt/status

Because customers have no accounts, cancellation/refund authorization must rely on the verified Manage Booking context and authoritative booking/payment ownership. Refund must return through the original supported payment flow in the academic model; do not accept an arbitrary new refund destination merely from browser input.

Branch 18 implements this as a final local cancellation followed by an asynchronous full refund. The locked local transaction creates one durable cancellation identity, cancels the ticket, releases only active seat ownership, and preserves successful payment/consumed-hold history. The current time is sampled once only after all authoritative locks are held; equality at the 24-hour cutoff is eligible. External Stripe Test Mode or Mock Bitcoin refund work begins only after commit and cannot undo cancellation.

Refund state is separate from cancellation state: `PENDING`, `IN_FLIGHT`, and `PROCESSING` are customer-facing “Refund processing”; `SUCCEEDED` is “Refund completed”; `REQUIRES_ATTENTION` is a truthful terminal operational state. Stripe responses and signed refund webhooks are validated against PaymentIntent, cancellation metadata, full amount, currency, and refund identity. Database uniqueness and fenced leases are primary correctness controls; Stripe idempotency is defense-in-depth.

If the customer wants a different flight:

```txt
cancel eligible old booking
  ↓
complete a new booking
```

No direct change-flight engine is required.

Cancellation must remain idempotent and must not create double refunds or double seat release.

# 25A. Airport Operations Boundary — Out of Scope

X-Fly stops at booking/ticketing, Manage Booking, and cancellation/refund.

The following are explicitly **not implemented** in the X-Fly customer/admin product:

- customer online check-in
- staff QR-scanner application
- airport Departure Control System
- gate operations
- boarding workflow
- Boarding Pass generation/lifecycle
- baggage loading/handling workflow
- government APIS submission

The presentation may describe that airline/airport staff can verify X-Fly ticket data or consume authorized data in their own operational system, then record check-in/boarding state there.

That downstream system is **not part of this repository**.

X-Fly may expose appropriately scoped REST API data to such a system through API clients/tokens, but must not simulate the downstream operational workflow inside the customer booking application.

# 26. Admin Design Direction & RBAC

The internal system is for X-Fly company operations and management. It must not be a single unrestricted `admin` role.

Recommended roles:

```txt
EXECUTIVE
  → read analytics, reports, revenue/profitability, booking demand, nationality/route trends

FLIGHT_MANAGER
  → create/edit/cancel flight services, schedules, cabin/fare/capacity configuration

BOOKING_OPERATIONS
  → search/view/manage bookings within approved business actions

TICKETING
  → inspect ticket state, print ticket-related documents, booking/passenger data required for ticket operations

BAGGAGE
  → read only the passenger/flight/baggage-related fields required for baggage operations

API_ADMIN
  → create/revoke API clients, tokens, scopes, integration access

SYSTEM_ADMIN
  → identity/security/system administration only
```

A role may be combined for a specific employee only through explicit authorization. `SYSTEM_ADMIN` does not automatically receive flight-edit permission.

## Company-device policy

Employees are expected to access internal X-Fly tools only from company-managed computers/tablets.

Do not implement this by trusting browser User-Agent strings.

Production enforcement belongs to the identity/network/device-trust boundary, for example managed-device posture, enterprise VPN/private network, or equivalent infrastructure controls. The application must still enforce authentication, session security, and RBAC.

Admin visual direction:

- dark professional dashboard
- yellow accent
- dense but readable
- large KPI
- strong table/filter UX
- restrained motion
- accessible on company desktop/tablet devices

# 27. Executive Analytics & Reporting

The executive/owner view must answer business questions such as:

- which flights/routes generate the most revenue
- which flights are under-booked / over-booked
- booking volume by route
- load factor
- passenger nationality distribution
- cancellation/refund rate
- cabin mix
- trends by time period

Required filters:

```txt
Origin
Destination
Flight
Date range
Cabin
Booking status
```

Required report periods:

```txt
Daily
Weekly
Monthly
Custom range
```

Core KPI:

- Revenue
- Bookings
- Passengers
- Seats sold
- Load Factor
- Cancellation Rate
- Refund Amount
- Flights operated/scheduled

## Profit / Loss

The stakeholder wants profit/loss by flight.

Do **not** label revenue as profit.

Profit/loss is valid only after the system has authoritative or approved demo cost data, for example operational cost per flight/instance.

Concept:

```txt
Profit = recognized revenue - modeled/authoritative flight operating cost
```

If cost data has not been implemented, show Revenue and mark Profit/Loss unavailable rather than inventing it.

Charts may use Recharts. Motion should be limited to initial reveal/filter transitions.

# 28. Flight Management UI

Only authorized `FLIGHT_MANAGER` users may create or modify flight inventory/schedules.

Capabilities:

```txt
Create Flight
Edit Flight
Cancel Flight
View Flight
```

Fields may include:

- Flight Number
- Origin
- Destination
- aircraft
- departure / arrival local schedule
- authoritative origin timezone
- cabin availability
- Business/First cabin pricing for new sellable inventory
- Business/First seat capacity/inventory template
- status
- approved operational cost field/fixture when required for profit/loss reporting

Requirements:

- validation
- preview before save
- destructive confirmation
- audit log
- optimistic UI only with server confirmation
- unauthorized employee roles cannot mutate flight data

# 29. Booking Management UI

Authorized booking/ticket operations staff can search/filter authoritative bookings.

Search/filter:

```txt
Booking Reference
Passenger
Flight
Date
Status
Cabin
```

Details may include:

- passenger identity fields appropriate to the staff role
- flight
- seat
- cabin-included baggage/service context
- ticket
- payment status (not raw card details)
- cancellation
- refund

Role-specific field filtering is required. A baggage role should not automatically receive payment or unnecessary passport/contact data.

Admin actions must be server-authorized and auditable.

# 30. Ticket / Passenger Operations

Ticketing/operations users need an internal read/print workflow for authoritative ticket records.

Capabilities:

- search by Booking Reference / Ticket Number
- view issued/cancelled ticket status
- view passenger/flight/seat fields required for the role
- print ticket-related documents where required
- inspect cancellation/refund relation
- verify that a ticket exists and is authoritative

This is **not** an airport check-in or boarding-pass application.

The system must never expose full Stripe card data, client secrets, or unrelated passenger-sensitive fields merely because an employee can access ticket operations.

# 31. External Integration & API Client Model

External-system integration is a **core requirement**, not an optional future extra.

Architecture:

```txt
External System
  ↓
API Client ID / Secret or Token
  ↓
Authentication
  ↓
Scope authorization
  ↓
X-Fly REST API
  ↓
Approved data only
```

Example scopes:

```txt
flights:read
bookings:read
passengers:read
tickets:read
baggage:read
analytics:read
```

Examples:

- baggage system receives only flight/passenger/baggage fields it needs
- ticketing/operations integration receives booking/ticket fields
- marketing analytics system receives approved booking/aggregate data, not campaign-building capability
- airport/downstream operational systems may consume approved ticket data without X-Fly implementing their check-in workflow

API client administration should support:

- create client
- reveal secret/token once
- store only a safe hash where applicable
- assign/revoke scopes
- revoke/disable client
- regenerate/rotate secret
- expiration where appropriate
- rate limiting
- audit log
- last-used visibility

External clients never connect directly to PostgreSQL.

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

# 33. Performance, Scale & Reliability Targets

X-Fly targets a premium customer experience and has aggressive stakeholder non-functional requirements.

## Customer response target

Stakeholder target:

```txt
critical customer actions should respond within ~1 second
```

Treat this as an API/application performance objective that must be measured, not a blanket promise for every network/browser condition.

Frontend experience targets remain:

```txt
LCP < 2.5s target
CLS < 0.1
INP < 200ms target
60fps where possible for motion
```

## Scale target

Stakeholder requirement mentions approximately **100,000 concurrent users/requests per second**.

This is a production-scale architecture target, **not a claim that the university self-hosted server currently sustains 100,000 req/s**.

The system design must therefore remain horizontally scalable:

- stateless backend instances where practical
- Nginx/upstream load balancing
- connection pooling
- database indexing/query budgets
- caching where safe
- rate limiting / backpressure
- observability
- capacity/load testing
- cloud migration path

Branch 28 must benchmark realistic scenarios and report measured throughput/latency honestly.

## Deployment environments

```txt
University / demo:
self-hosted Ubuntu + Docker + Nginx + Named Cloudflare Tunnel

Target large-scale production:
Cloud infrastructure / horizontally scalable deployment
```

The self-hosted deployment proves functional/operational architecture; it does not certify the full stakeholder scale target.

## Data durability / safety

Requirement: booking/ticket data must not be lost.

Design expectations:

- PostgreSQL constraints/transactions
- persistent storage
- automated backups
- tested restore procedure
- controlled migrations
- auditability for privileged actions
- secret management
- HTTPS
- monitoring/alerting strategy

No system can truthfully guarantee absolute zero data loss under every failure mode; documents and presentations should describe the durability/recovery mechanisms and tested recovery objectives instead of making an absolute guarantee.

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
frontend/src/

app/
  manage-booking/
  admin/

components/
  brand/
  layout/
  navigation/
  motion/
  booking/
    confirmation/
    review/
  flight/
  seat/
  ticket/
  payment/
  manage-booking/
  admin/
  analytics/

features/
  flight-search/
  flight-results/
  seat-selection/
  booking/
  payment/
  manage-booking/
  cancellation/
  admin-auth/
  admin-rbac/
  admin-flights/
  admin-bookings/
  admin-tickets/
  analytics/
  api-clients/

lib/
  api/
  motion/
  validation/
  utils/

hooks/
styles/
types/
```

Do not create customer check-in or boarding-pass feature modules in this product scope.

Customer booking-confirmation email is outside the active scope. Legacy email infrastructure may remain temporarily for safe retirement, but the frontend must not depend on email delivery.

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

## Goal

สร้าง passenger/travel-document flow ที่ใกล้ airline international booking จริง โดยยังไม่เชื่อม government systems.

## Tasks

- passenger form
- multi-passenger support
- Title
- Given / Middle / Family name
- DOB
- Gender — required; Male / Female only for new bookings
- Nationality
- Passport number
- Passport issuing country
- Passport expiry
- optional passport issue date
- phone country code + phone
- no customer email field for new bookings
- optional emergency contact
- validation
- travel-document-name warning
- error motion
- progress stepper
- booking summary sidebar
- protect sensitive document data from unsafe browser persistence

## Important

- Passenger names must match travel documents.
- International booking should collect travel-document details in this academic flow.
- No APIS/government transmission.
- No Timatic/visa eligibility engine.

---

# 53A. BRANCH 13A — Travel Extras (Historical / Superseded)

## Branch

```txt
feat/13a-travel-extras
```

## Status

**Completed historically, but superseded by the latest stakeholder requirement.**

Branch 13A remains part of repository history, but its standalone Travel Extras customer step is no longer active product scope.

Branch 20A must remove the active extras route/step/state/pricing dependencies while preserving any non-destructive historical data required by existing records.

Cabin-included Business/First baggage or service benefits may remain visible as included benefits.

---

# 54. BRANCH 14 — Booking Review

## Branch

```txt
feat/14-booking-review
```

## Tasks

- complete itinerary
- passengers
- travel-document completeness summary
- seat assignment
- cabin
- cabin-included baggage/service benefits
- base fare
- taxes fixture
- airport/passenger charges fixture
- total
- fare conditions
- baggage conditions
- cancellation policy
- edit links
- travel-document-name acknowledgement
- fare/cancellation acknowledgement
- final confirmation
- responsive summary

## Important

Do not claim demo taxes/fees are live government/airport values unless they come from a real backend source.

---

# 55. BRANCH 15 — Mock Payment Foundation

## Branch

```txt
feat/15-mock-payment-ui
```

## Status

**Completed / merged.**

Branch 15 established the provider-neutral payment foundation and mock payment behavior before Stripe integration.

Delivered concepts include:

- payment method selection
- payment attempt lifecycle
- server-authoritative Review/payment snapshot boundary
- payment/booking/ticket state separation
- idempotent payment attempts
- atomic successful inventory finalization
- Bitcoin mock gateway
- payment failure/cancellation behavior
- no sensitive card-data persistence

This branch is historical foundation. The Card mock UI was superseded by Branch 15b.

---

# 55A. BRANCH 15B — Stripe Test Payment

## Branch

```txt
feat/15b-stripe-test-payment
```

## Status

**Completed / manually verified / READY TO CLOSE.**

## Goal

Replace the raw mock Card experience with **Stripe Test Mode** while preserving Bitcoin as mock-only.

## Implemented

- Stripe Payment Element
- backend-created PaymentIntent
- provider-neutral payment gateway/reconciliation architecture
- server-authoritative amount/currency
- Stripe Test Mode only
- request + Stripe idempotency
- Stripe webhook signature verification
- webhook replay persistence
- PaymentIntent reconciliation
- safe cancellation
- finalization reservation for in-flight Stripe payment
- inventory protection while external payment outcome is unresolved
- authoritative frontend polling after `stripe.confirmPayment`
- stable customer-safe failure codes
- Card success / decline / 3DS support
- Bitcoin mock regression preserved

## Critical Inventory Rule

Successful payment finalization already owns:

```txt
payment_attempt = SUCCEEDED
        ↓
payment_attempt_seats
        ↓
flight_seats = BOOKED
hold_id = NULL
booked_at = populated
        ↓
seat_holds.consumed_at = populated
```

Later Ticket/QR code must be downstream-only and must not BOOK seats or consume holds again.

## Manual QA Verified

```txt
Success Card                  VERIFIED
Declined Card                 VERIFIED
3DS COMPLETE                  VERIFIED
Webhook delivery              VERIFIED
Webhook persistence           VERIFIED
Stripe Dashboard correlation  VERIFIED
PaymentIntent retrieval       VERIFIED
DB inventory finalization     VERIFIED
Bitcoin Mock                  VERIFIED
```

3DS FAIL was not manually tested at branch closure; automated failure-path coverage exists.

## Local Runtime

```txt
Terminal 1: backend  → cargo run
Terminal 2: frontend → npm run dev
Terminal 3: Stripe   → stripe listen --forward-to localhost:8080/api/v1/payments/stripe/webhook
```

## Server Runtime

University/demo server remains **Stripe Test Mode only**.

```txt
Browser
  ↓
X-Fly Frontend
  ↓
Rust Backend
  ↔ Stripe Test/Sandbox
  ↑
public HTTPS Stripe webhook
```

No `stripe listen` process is required on the deployed server.

---

# 56. BRANCH 16 — Ticket Foundation + Signed QR

## Branch

```txt
feat/16-ticket-qr-experience
```

## Status

**Completed technical foundation / manually verified.**

Branch 16 established the issued-ticket domain, server-generated Booking Reference / Ticket Number, signed verification QR, public zero-PII verification, and private authorized ticket retrieval.

The latest product UX decision in Branch 17 **repositions** these capabilities:

```txt
Immediate post-payment page
→ Booking Confirmed summary
→ NO QR

Returning customer
→ Manage Booking verification
→ Booking Details / E-Ticket
→ signed QR shown here
```

Therefore Branch 16 remains the backend/ticket/QR foundation, while Branch 17 changes where customers see the E-Ticket/QR.

## Core Architecture & Invariants

- **Downstream-only issuance**: ticket issuance/retrieval never finalizes payment or inventory.
- Ticket issues only from authoritative `SUCCEEDED` payment + consumed hold + finalized `BOOKED` seats.
- `UNIQUE(payment_attempt_id)` prevents duplicate logical tickets.
- Booking Reference and Ticket Number are separate identifiers.
- Current Booking Reference shape is `XF` + 8 unambiguous uppercase characters (`^XF[A-Z2-9]{8}$`).
- Ticket Number is a separate server-generated customer-facing identifier.
- Stripe PaymentIntent IDs/database UUIDs are not customer-facing ticket numbers.
- QR signing uses backend-only `TICKET_QR_SIGNING_SECRET`.
- QR verification uses HMAC-SHA256 and constant-time comparison.
- QR/public verify expose no passenger PII.
- QR encodes a frontend verification URL and supports deployed origin configuration.
- Card remains Stripe Test Mode only.
- Bitcoin remains mock-only.

## Manual QA Foundation Verified

- Stripe Test payment → ticket
- Mock Bitcoin → ticket
- refresh/idempotency
- DB payment/hold/seat invariant
- QR valid verification
- tampered QR rejection
- mobile/LAN QR verification
- unauthorized private ticket access blocked

Do not weaken these security/inventory invariants while simplifying the customer UX.

# 57. BRANCH 17 — Manage Booking / E-Ticket Retrieval

## Branch

```txt
feat/17-manage-booking-ui
```

## Current Goal

Refine the returning-customer experience around the latest product boundary.

Required changes before closure:

- fix the stale Manage Booking authorization/context bug found in manual QA
- `/manage-booking` must be lookup-only, not auto-display an old cookie booking
- lookup uses **Booking Reference + Last Name**
- remove optional Email as a lookup credential
- successful lookup establishes the scoped HttpOnly authorization
- navigate to `/manage-booking/details`
- details show authoritative booking/E-Ticket information
- move verification QR to Manage Booking Details / E-Ticket
- remove QR from the immediate Booking Confirmed summary
- remove customer Check-in UI/state/CTA
- preserve cancellation eligibility display only
- add clear global `Manage Booking` navigation
- keep passenger-sensitive fields masked/minimized
- ticket access must reuse the existing issued ticket, never issue a duplicate
- maintain observational-only payment/hold/seat behavior

## Manual-QA Defect — Must Fix

Manual QA observed:

```txt
Ticket / newly booked journey
  ↓
Manage Booking
  ↓
old authorization cookie
  ↓
different previous booking displayed
```

This is a Branch 17 blocker.

The exact latest successful lookup must determine the booking displayed. Stale-cookie state must never substitute another booking.

## Engineering Hygiene Note

A separate pre-existing test-suite lifecycle defect was reproduced on the Branch 16 baseline: repeated full backend test runs can collide with legacy active hold fixtures. Fix that in a dedicated test-hygiene change; do not disguise it as a Manage Booking product defect.

---

# 57A. BRANCH 17A — Booking Confirmation Email (Historical / Superseded)

## Branch

```txt
feat/17a-booking-confirmation-email
```

## Status

**Completed historically, but superseded by the 07 Sep 2026 stakeholder requirement.**

Branch 17A introduced durable confirmation-email delivery after ticket issuance. That implementation remains part of repository/schema history, but customer confirmation email is no longer active product scope.

Branch 20A must:

- stop requiring/collecting customer email for new bookings
- stop creating new booking-confirmation delivery work for new bookings
- remove customer-facing copy that promises an email
- preserve successful payment, inventory finalization, ticket issuance, Booking Confirmed, Manage Booking, and cancellation/refund behavior
- preserve historical email/outbox records without destructive DEV-database cleanup
- remove or disable unused worker/provider paths safely where they are dedicated only to booking confirmation

Manage Booking remains Website → Booking Reference + Last Name.

---

# 58. BRANCH 18 — Cancellation & Full Refund

## Branch

```txt
feat/18-cancellation-refund
```

## Rule

```txt
>= 24 hours before authoritative departure
→ cancel allowed
→ fee 0
→ refund 100%

< 24 hours
→ cancel unavailable
```

## Tasks

- verified Manage Booking authorization for the exact booking
- server-side eligibility revalidation using origin timezone
- confirmation UX
- idempotent booking cancellation
- idempotent ticket cancellation
- authoritative seat release
- refund state
- Stripe Test Mode refund integration where current payment architecture supports it
- Mock Bitcoin refund simulation
- receipt/status
- repeat-request safety
- audit event

No direct flight-change engine. Customer cancels and books again.

---

# 59. BRANCH 19 — Admin Authentication + RBAC

## Branch

```txt
feat/19-admin-auth-rbac
```

## Tasks

- employee login/session
- protected admin shell
- RBAC policy
- roles: EXECUTIVE / FLIGHT_MANAGER / BOOKING_OPERATIONS / TICKET_PASSENGER_OPERATIONS / BAGGAGE_STAFF / API_ADMIN / SYSTEM_ADMIN
- unauthorized states
- session expiry/logout
- role-aware navigation
- server-side authorization tests
- company-device policy boundary documented

Do not grant flight mutation to generic admin by default.

Branch 19 uses PostgreSQL-backed staff identities, many-to-many roles, role permissions, login throttles, and opaque revocable sessions. Effective permissions are the union of the account's current durable role grants and are resolved on every authenticated request. `SYSTEM_ADMIN` manages only the identity/access domain by default and has no implicit superuser behavior.

Staff passwords use Argon2id and are provisioned only through the local `staff_admin bootstrap|create` CLI with a hidden confirmed password prompt. There is no public registration endpoint or default credential. Staff authentication uses the separate `x_fly_staff_session` cookie scoped to `/admin`; it does not reuse or alter customer hold or Manage Booking authorization.

The Branch 19 `/admin` workspace is an authentication, authorization, and visual shell only. Future module links stay omitted until their branches provide real functionality; the shell contains no Executive Dashboard metrics or fabricated analytics. English and Thai use the shared typed locale system.

Company-managed-device access is not proven by application login or browser checks. Real enforcement remains a Branch 27 identity/network control using an auditable mechanism such as managed-device certificates, Zero Trust device posture, MDM identity, or a private/VPN policy.

---

# 60. BRANCH 20 — Executive Dashboard / Owner Analytics

## Branch

```txt
feat/20-executive-analytics-reports
```

## Status

**Implementation complete and manually reviewed. Final read-only branch audit remains before user commit/merge.**

Automated verification reported clean frontend/backend gates, responsive EN/TH behavior, no dashboard console errors during manual QA, and no deployment/commit/push performed by coding agents.

## Implemented boundary

Branch 20 adds one consolidated, read-only executive analytics endpoint and a premium bilingual dashboard inside the Branch 19 Admin Shell. Both backend access and frontend navigation require the effective union of `dashboard:read`, `analytics:read`, and `reports:read`; role names are never treated as authorization and `SYSTEM_ADMIN` remains limited to its explicit grants.

Commercial metrics use one authoritative cohort: `payment_attempts` rows with `status = 'SUCCEEDED'`, `currency_code = 'THB'`, the selected provider, and `succeeded_at` within inclusive Bangkok calendar dates. The existing unique successful-payment-per-hold constraint prevents retry double-counting. Gross Booking Revenue is the stored paid amount before refunds; average booking value is derived from that amount and booking count. Tickets, cancellations, and refund lifecycle values join the existing one-to-one historical records without mutating them.

The dashboard provides gross revenue, booking and ticket counts, average booking value, cancellation rate, completed and pending refund values, route and cabin aggregates, zero-filled daily revenue/demand trends, most-booked and highest-revenue flights, and deterministic observations. Current recorded seat occupancy uses sellable `flight_seats` for departures in the selected dates. Because flight inventory is materialized on demand, the UI explicitly describes this as recorded seat occupancy rather than fleet-wide load factor.

Its visual system is an editorial executive terminal rather than a conventional card dashboard: an abstract non-geographic flight-path opening, asymmetric commercial command deck, distinct revenue and demand instruments, commercial-corridor rankings, instrumented cabin composition, refund-state rail, flight ranking board, recorded-capacity runway, and numbered executive brief. GSAP owns the scoped entrance, first-viewport, path, and data-refresh choreography; CSS owns short control feedback. Reduced-motion users receive stable final content immediately without animation-dependent meaning or interaction.

The final chart refinement uses a point-preserving smoothed revenue curve and a larger demand instrument while keeping the exact daily dataset authoritative. Successful filter refreshes rerun chart reveal/draw choreography. Both charts support premium nearest-day inspection: mouse/pointer, touch, and keyboard (`Left`/`Right`/`Home`/`End`/`Escape`) snap only to real daily API points and show localized date + authoritative value/count; no tooltip value is interpolated from the visual curve.

Filters support inclusive custom dates (maximum 366 days), Today, 7 Days, 30 Days, This Month, route, cabin, and the currently valid stored providers `STRIPE` and `MOCK_BITCOIN`. Queries run in a PostgreSQL read-only repeatable-read transaction. Responses contain aggregates, flight/route identifiers, and dates only; passenger identity, booking references, ticket numbers, provider references, and secrets are excluded.

Because Branch 20 predates the latest Business/First-only stakeholder scope, its read-only analytics may truthfully surface legacy Economy/Premium Economy history already stored in PostgreSQL. Branch 20A must restrict **new bookings**, not falsify or delete historical analytics.

Profit/Loss, passenger nationality, reporting/export workflows, scheduled delivery, and PDF/CSV output are intentionally absent. Profit requires authoritative operating-cost data, nationality is passenger-sensitive and not needed for this first dashboard, and report generation belongs to later approved scope.

Manual QA also found one unrelated customer-storytelling regression: the `X-FLY SERVICE` image referenced a non-existent asset path. The focused fix points the section to an existing tracked cabin image and adds a regression check; this does not change booking/business behavior.

---

# 60A. BRANCH 20A — Customer Requirements Alignment

## Branch

```txt
feat/20a-customer-requirements-alignment
```

## Goal

Align the already-built customer booking lifecycle with the latest stakeholder requirements **before Branch 21 begins**, without weakening the completed payment/ticket/cancellation foundations or destructively rewriting historical data.

## Required changes

### Cabin scope

- new customer bookings offer **Business** and **First** only
- remove Economy/Premium Economy from customer search/detail/seat-selection controls
- backend new-booking/hold/review/payment boundaries must reject non-sellable legacy cabins rather than trusting frontend filtering
- preserve historical Economy/Premium Economy records where they already exist
- update fare fixtures, cabin availability, seat geometry/templates, Review, and tests consistently

### Passenger gender

- gender is required for new bookings
- accepted customer values: **Male / Female only**
- remove `Unspecified` from customer controls and new-booking validation
- do not destructively rewrite historical rows solely to remove a legacy value

### Contact data

- new booking contact is **phone country code + phone number**
- remove required customer email from Passenger/Contact UI
- backend/persistence must not require a fake placeholder email
- if legacy schema requires migration, make email nullable/optional safely while preserving historical values
- Manage Booking remains Booking Reference + Last Name

### Confirmation email retirement

- no new customer booking-confirmation email
- remove UI copy promising email delivery
- stop creating new confirmation-email outbox work for new bookings
- safely disable/remove worker/provider paths that exist only for booking confirmation
- keep historical delivery records/schema unless a later migration can remove them safely
- payment/ticket finalization must remain independent from notification code

### Travel Extras removal

- remove the active Travel Extras step/page from the primary booking lifecycle
- customer flow becomes Seat → Passenger + Phone Contact → Review → Payment
- remove selected-extras state/pricing from Review/payment snapshots where active
- safe redirect/recovery behavior for stale/direct `/booking/extras` URLs
- Business/First cabin-included benefits may still display as included benefits

## Safety / regression boundaries

- do not reset or destroy the DEV PostgreSQL volume
- do not rewrite historical bookings merely to match the new customer UI
- preserve seat-hold concurrency
- preserve Stripe Test Mode + Mock Bitcoin
- preserve payment/ticket lifecycle separation
- preserve Manage Booking / E-Ticket / signed QR
- preserve cancellation/refund rules
- preserve Admin/RBAC/Branch 20 analytics behavior; analytics may truthfully include legacy cabin history until data naturally ages out or a separately approved data migration exists

## Exit criteria

- Business/First-only new booking is enforced frontend + backend
- Male/Female-only required gender is enforced frontend + backend
- new bookings no longer collect customer email
- no booking-confirmation email is created/sent for new bookings
- Travel Extras is no longer in the active booking path
- Review/Payment/Booking Confirmed/Manage Booking remain coherent
- EN/TH, responsive, keyboard, reduced-motion, tests, build, and customer E2E regressions pass

---

# 61. BRANCH 21 — Flight Management

## Branch

```txt
feat/21-flight-management
```

## Tasks

- FLIGHT_MANAGER-only create/edit/cancel
- schedules / timezone
- route
- aircraft
- cabin pricing
- seat capacity/inventory template
- status
- cost data boundary for profit/loss where approved
- validation
- audit trail
- destructive confirmation

---

# 62. BRANCH 22 — Booking Management

## Branch

```txt
feat/22-booking-management
```

## Tasks

- booking search/filter
- passenger search
- flight/date/status/cabin filters
- authoritative booking detail
- payment/ticket/cancellation/refund summary
- role-aware sensitive-field filtering
- auditable approved operations

This is internal booking administration, not airport check-in.

---

# 63. BRANCH 23 — Ticket / Passenger Operations

## Branch

```txt
feat/23-ticket-passenger-operations
```

## Tasks

- ticket search
- Booking Reference / Ticket Number lookup
- ticket status
- passenger/flight/seat view according to role
- print-ready ticket documentation
- cancellation/refund relation
- ticket verification
- TICKETING / BAGGAGE-specific field permissions

No staff QR scanner or Boarding Pass workflow.

---

# 64. BRANCH 24 — API Client Management

## Branch

```txt
feat/24-api-client-management
```

## Tasks

- API client list
- create client
- reveal credential once
- hashed/safe credential persistence
- assign scopes
- revoke / disable
- rotate/regenerate
- expiration
- last-used
- audit visibility

---

# 65. BRANCH 25 — External REST API + Token / Scopes

## Branch

```txt
feat/25-external-rest-api
```

## Tasks

- bearer/API-client authentication
- scope authorization
- flights:read
- bookings:read
- passengers:read
- tickets:read
- baggage:read
- analytics:read where approved
- field-level data minimization
- rate limiting
- audit logs
- API documentation
- token revocation behavior
- contract tests

External clients never receive direct PostgreSQL access.

---

# 66. BRANCH 26 — Security, Audit & Company-Device Boundary

## Branch

```txt
feat/26-security-audit
```

## Tasks

- admin session hardening
- RBAC audit
- audit log for privileged mutations
- API-token audit
- secure headers
- secret review
- sensitive-data exposure review
- company-device/network enforcement integration boundary
- unauthorized/access-denied UX
- security documentation
- legal/compliance requirement matrix

Do not claim application-side User-Agent checks enforce company-owned devices.

---

# 67. BRANCH 27 — Accessibility + Responsive + Cross-Browser QA

## Branch

```txt
test/27-accessibility-responsive
```

## Tasks

- keyboard
- focus
- ARIA
- contrast
- reduced motion
- responsive desktop/tablet/mobile
- iOS Safari
- Android Chrome
- Chrome / Safari / Firefox / Edge
- touch targets
- booking forms
- seat map
- Manage Booking
- Admin company-tablet layout

---

# 68. BRANCH 28 — Performance & Load Testing

## Branch

```txt
perf/28-load-performance
```

## Tasks

Frontend:

- bundle analysis
- media optimization
- animation lifecycle
- Core Web Vitals
- hydration
- lazy loading

Backend/system:

- latency benchmarks
- database query/index review
- connection-pool behavior
- concurrency/load scenarios
- Nginx upstream behavior
- rate limiting/backpressure
- identify horizontal-scaling boundary

Report measured results honestly.

The stakeholder `~100,000 users/requests per second` target is a production architecture objective, not a result to claim unless a representative environment actually proves it.

---

# 69. BRANCH 29 — Production Readiness + End-to-End QA

## Branch

```txt
test/29-production-readiness
```

## Core Customer E2E

```txt
Search
→ Flight
→ Business / First
→ Seat Hold
→ Passenger + Phone Contact
→ Review
→ Stripe Test / Mock Bitcoin
→ Booking Confirmed
→ Manage Booking lookup
→ E-Ticket / QR
→ Cancellation eligibility / cancellation
```

Must include:

- Business/First-only new-booking enforcement
- backend rejection of legacy/non-sellable cabin attempts for new bookings
- required Male/Female gender validation
- phone-only booking contact with no required customer email
- no active Travel Extras step / safe stale extras-route behavior
- no customer booking-confirmation email creation/delivery for new bookings
- seat conflict / hold expiry
- payment decline/failure
- ticket not issued on failed payment
- stale Manage Booking cookie regression
- wrong surname/reference anti-enumeration
- unauthorized details access
- cancellation >=24h / <24h
- refund idempotency
- responsive/customer critical flows
- admin RBAC critical flows
- external API token/scope critical flows
- backup/restore and migration readiness evidence

Explicitly absent from E2E:

- customer online check-in
- Boarding Pass
- staff QR scanner
- gate/baggage operational workflow

---

# 69A. Nginx Deployment Policy

สำหรับ deployment ของ X-Fly ให้ถือว่า **Nginx เป็น Primary Web Edge / Reverse Proxy / TLS Origin / Application Gateway หลักของโปรเจกต์**.

Domain infrastructure กลางของ server คือ:

```txt
siwakondev.win
```

X-Fly ควรใช้ project-specific subdomain เพื่อไม่ผูก root domain กับ product เดียว เช่น:

```txt
https://x-fly.siwakondev.win
```

ชื่อจริงต้องตรวจว่า route/DNS ยังว่างและได้รับ approval ก่อน deploy.

Agent ต้อง:

- ใช้ Nginx เป็น reverse proxy หลัก
- ห้ามเปลี่ยนไปใช้ Caddy หรือ reverse proxy ตัวอื่นโดยพลการ
- ให้ Nginx terminate TLS ที่ origin จริง ไม่ใช่ติด certificate ไว้เฉย ๆ
- ใช้ **Named Cloudflare Tunnel** เป็น public ingress หลักของ X-Fly
- ให้ `cloudflared` ส่ง traffic เข้า Nginx ผ่าน HTTPS origin เช่น `https://127.0.0.1:8443`
- เปิด certificate verification ระหว่าง `cloudflared` → Nginx และกำหนด SNI / `originServerName` ให้ตรงกับ certificate
- ห้ามใช้ `noTLSVerify: true` เป็น production/default configuration
- validate Nginx config ก่อน reload/restart ทุกครั้ง
- ใช้ Docker DNS/service names สำหรับ container upstream เมื่อ deploy
- ไม่ expose PostgreSQL สู่ Internet
- ไม่ expose frontend/backend public host ports โดยไม่จำเป็น
- คง PostgreSQL เป็น private database หลัง backend เท่านั้น
- same-origin routing ควรให้ `/` ไป Frontend และ `/api/*` ไป Rust backend
- รองรับ Nginx `upstream` สำหรับ backend replicas / load balancing เมื่อมีเหตุผลจริง
- ใช้ Nginx serve static assets โดยตรงเฉพาะเมื่อเหมาะกับ production build และไม่ทำให้ Next.js asset semantics เสีย
- ใช้ Nginx เป็น gateway หน้า backend หลาย service เมื่อ architecture แตก service เพิ่ม

Target deployment direction:

```txt
Internet / Professor / Tester
  ↓ HTTPS
Cloudflare Edge + DNS
  ↓ Named Cloudflare Tunnel
cloudflared on safe-host
  ↓ HTTPS (origin TLS, certificate verified)
127.0.0.1:8443
  ↓
Nginx :443 ssl
  ├── /      → Next.js
  └── /api/* → Rust + Axum
                  ↓
             PostgreSQL 18
```

### X-Fly TLS / Certificate Plan

เป้าหมายคือให้ **Nginx ถือ certificate + private key และทำ TLS termination ที่ origin จริง** เพื่อฝึก production-style TLS lifecycle.

Preferred certificate strategy สำหรับ `safe-host`:

```txt
Let's Encrypt
  ↓ DNS-01 challenge via Cloudflare DNS API
siwakondev.win + *.siwakondev.win
  ↓
Nginx certificate files (read-only mount)
```

เหตุผลที่เลือก DNS-01:

- server อยู่หลัง CGNAT / router ที่ควบคุมไม่ได้
- ไม่ต้องเปิด inbound 80/443 เพื่อทำ ACME HTTP challenge
- wildcard certificate ใช้กับหลาย project subdomain ได้
- รองรับ automatic renewal ได้

Security requirements:

- Cloudflare API token สำหรับ ACME ต้องเป็น least-privilege และอยู่บน server เท่านั้น
- certificate private key ห้าม commit Git
- certificate path ต้อง mount เข้า Nginx แบบ read-only
- renewal ต้องมี validation/reload hook ที่ปลอดภัย
- ถ้าใช้ Cloudflare Origin CA แทนในอนาคต ต้องบันทึกเหตุผลและยืนยันว่า origin รับ traffic ผ่าน Cloudflare เท่านั้น

> การเพิ่ม TLS ที่ Nginx มีเป้าหมายหลักด้าน security architecture, operational realism และการเรียนรู้ Nginx TLS; ไม่ควรอ้างว่า HTTPS เพิ่ม performance โดยตัวมันเอง.

Stripe Test Mode webhook บน server ต้องใช้ fixed public HTTPS URL ผ่าน Named Tunnel/Nginx เช่น:

```txt
https://x-fly.siwakondev.win/api/v1/payments/stripe/webhook
```

Quick Tunnel ยังคงใช้ได้เฉพาะ fallback / temporary troubleshooting และไม่ใช่ default deployment path ของ X-Fly หลังมี domain แล้ว.

---

# 70. BRANCH 30 — Deployment Preparation

## Branch

```txt
chore/30-deployment-prep
```

## Deployment Target

สำหรับ **university/demo environment** X-Fly จะ deploy เข้า **self-hosted Ubuntu Server ที่เตรียมไว้แล้ว** แทนการใช้ Vercel / Render เป็น default demo target.

Stakeholder production-scale requirement still prefers Cloud infrastructure; the self-hosted server is the current academic/demo deployment and learning environment, not the final 100,000 req/s production claim.

Server foundation ที่มีอยู่แล้ว:

```txt
Ubuntu Server
├── Docker Engine + Docker Compose
├── External Docker network: web
├── PostgreSQL 18
├── Nginx reverse proxy / application gateway
├── UFW
├── SSH key authentication
├── PostgreSQL daily backup
└── cloudflared
```

หลักการสำคัญ:

> **อย่ารื้อหรือสร้าง server infrastructure ใหม่โดยไม่จำเป็น**
>
> Project ต้องถูก Dockerize และปรับให้เสียบเข้ากับ infrastructure ที่ผ่านการทดสอบแล้ว.

## Target Runtime Architecture

```txt
x-fly.siwakondev.win
        ↓ HTTPS
Cloudflare Edge + DNS
        ↓
Named Cloudflare Tunnel
        ↓
cloudflared on Ubuntu Server
        ↓ HTTPS origin (certificate verified)
127.0.0.1:8443
        ↓
Nginx :443 ssl
        ↓
Docker network: web
   ┌──────┴──────┐
   ↓             ↓
Frontend       Backend
Next.js        Rust + Axum
:3000          :8080
                  ↓
             PostgreSQL 18
```

Nginx ต้องถือ certificate/private key และ terminate TLS ที่ origin จริง. Public edge TLS ของ Cloudflare และ origin TLS ของ Nginx เป็นคนละ TLS hop และทั้งสองต้องถูกใช้งานจริง.

Frontend และ Backend ไม่ควร publish public host ports หาก Nginx สามารถเข้าถึงผ่าน Docker network `web` ได้.

Browser ต้องไม่เชื่อม PostgreSQL โดยตรง.

## PostgreSQL Inspection / pgAdmin Strategy

X-Fly ต้องสามารถตรวจสอบ schema และข้อมูลจริงผ่าน **pgAdmin บน Mac** ได้ทั้ง Local และ Self-Hosted Production โดยใช้ 2 connection แยกจากกัน.

### Local Development — pgAdmin

Local architecture:

```txt
Mac
├── Next.js
├── Rust + Axum
├── pgAdmin
└── Docker
    └── PostgreSQL
```

Backend และ pgAdmin ต้องชี้เข้า PostgreSQL local instance ตัวเดียวกัน.

ตัวอย่าง connection:

```txt
pgAdmin connection name:
X-Fly - LOCAL

Host:
localhost

Port:
<POSTGRES_HOST_PORT>
ตัวอย่าง 5433

Database:
x_fly

Username / Password:
ใช้ค่าจาก local .env
```

Backend local ใช้ host-facing connection เช่น:

```txt
DATABASE_URL=postgres://<user>:<password>@localhost:<POSTGRES_HOST_PORT>/x_fly
```

จุดประสงค์:

- ดู tables / columns / constraints
- ตรวจ migration result
- ตรวจ seed data
- ตรวจ seat hold / booking / passenger / payment / ticket records
- query/debug data ระหว่าง development
- ยืนยันว่า backend เขียนข้อมูลลง database จริง

### Self-Hosted Production — pgAdmin ผ่าน SSH Tunnel

Production PostgreSQL **ห้ามเปิด public Internet เพื่อให้ pgAdmin เข้า**.

Mac มี SSH access ไป Ubuntu Server อยู่แล้ว ดังนั้น production inspection ต้องใช้:

```txt
pgAdmin on Mac
      ↓
SSH Tunnel
      ↓
Ubuntu Server
      ↓
127.0.0.1:5432
      ↓
PostgreSQL Container
```

Server PostgreSQL ยังคง bind เฉพาะ localhost ของ Ubuntu host:

```txt
127.0.0.1:5432
```

pgAdmin production connection ควรตั้งชื่อชัดเจน เช่น:

```txt
X-Fly - PRODUCTION
```

และใช้ SSH Tunnel:

```txt
SSH Host:
safe-host
หรือ hostname/address ที่ SSH config resolve ได้

SSH authentication:
existing SSH key

Database Host after tunnel:
127.0.0.1

Database Port:
5432

Database:
x_fly
```

หลักการ:

- Local DB และ Production DB เป็น **คนละ PostgreSQL instance**
- schema ควรสอดคล้องกันผ่าน migrations
- data ไม่ sync อัตโนมัติ
- Local ใช้สำหรับ development/test data
- Production ใช้สำหรับ deployed/demo data
- ห้าม copy Local database ทั้งก้อนขึ้น Production โดยไม่มี controlled migration/seed plan
- ห้ามเปิด PostgreSQL public port เพื่อความสะดวกของ pgAdmin

ใน pgAdmin ควรเห็นแยกชัด:

```txt
Servers
├── X-Fly - LOCAL
│   └── x_fly
└── X-Fly - PRODUCTION
    └── x_fly
```

ก่อนแก้ไข Production data ผ่าน pgAdmin ต้องตรวจ connection name ให้แน่ใจเพื่อลดความเสี่ยงแก้ผิด environment.

## Frontend Preparation

- production Next.js Dockerfile
- production build (`npm run build`)
- production runtime (`npm run start` หรือ standalone output ตาม implementation ที่อนุมัติ)
- join external Docker network `web`
- responsive/static asset optimization
- environment-variable boundary
- public API calls ใช้ same-origin relative `/api` เมื่อเป็นไปได้
- production error pages
- health/readiness strategy
- source-map policy
- no `npm run dev` on server

## Backend Preparation

- production Rust + Axum Dockerfile
- release build
- bind service ภายใน container ตาม configured port
- join Docker network ที่จำเป็น
- connect PostgreSQL ด้วย Docker DNS/service hostname ไม่ใช่ hard-coded LAN IP
- database migrations
- seed strategy สำหรับ academic/demo data
- structured logging
- health endpoint
- graceful shutdown
- production error handling
- secrets via server `.env`, never committed
- Stripe remains Test Mode only for university/demo deployment
- backend server env contains a Test Mode `STRIPE_SECRET_KEY` and the deployed webhook `STRIPE_WEBHOOK_SECRET`
- frontend contains the matching Test Mode `NEXT_PUBLIC_STRIPE_PUBLISHABLE_KEY`
- all Stripe keys/webhook configuration must belong to the same Stripe Test/Sandbox account context

## Reverse Proxy / Routing Preparation

**Nginx เป็น Primary Web Edge / Reverse Proxy ของ application ภายใน server**

Nginx มีหน้าที่หลักสำหรับ X-Fly deployment:

1. **Reverse Proxy** — รับ request จาก Cloudflare แล้วส่งต่อไป Frontend / Backend
2. **Load Balancer** — รองรับการกระจาย request ไป backend หลาย instance ในอนาคต
3. **Web Server** — รับและตอบ HTTP request และสามารถ serve static content ได้
4. **TLS Termination** — สามารถเป็นจุด terminate TLS ได้เมื่อ architecture ใช้ TLS ที่ Nginx โดยตรง
5. **Serve Static Files** — สามารถส่ง static assets โดยตรงเมื่อเหมาะสม
6. **Gateway หน้า backend หลาย service** — route เช่น `/api/auth`, `/api/booking`, `/api/payment` ไป service ที่เหมาะสม

สำหรับ target architecture ใหม่ Cloudflare Named Tunnel เป็น public ingress และ `cloudflared` ต้องเชื่อม Nginx ผ่าน HTTPS origin บน loopback ของ Ubuntu Server.

Target request flow:

```txt
Internet
  ↓ HTTPS
Cloudflare Edge
  ↓ Named Tunnel
cloudflared
  ↓ HTTPS / verified origin certificate
Nginx :443 ssl
  ↓
Docker network: web
  ├── Frontend
  └── Backend / API services
       ↓
   PostgreSQL
```

Nginx จะเป็น entry point ของ application ภายใน server.

Target routing concept:

```txt
/
↓
frontend:3000

/api/*
↓
backend:8080
```

Config จริงต้องตรวจ route ของ Next.js และ Axum ก่อนใช้ ห้าม copy config แบบ blind.

## Networking & Security Rules

- do not expose PostgreSQL to Internet
- do not hard-code DHCP/LAN server IP
- do not publish unnecessary frontend/backend ports
- use Docker DNS/service names for container-to-container traffic
- `.env` stays on server and is gitignored
- Git contains only `.env.example`
- SSH remains key-only
- UFW remains deny-incoming by default
- public traffic enters through Cloudflare Tunnel → Nginx only
- verify admin/security restrictions before public demo
- configure Stripe Test/Sandbox webhook endpoint to public HTTPS `/api/v1/payments/stripe/webhook`
- do not run `stripe listen` as the deployed webhook transport
- never use live Stripe keys for this university/demo deployment

## Backup / Data Preparation

PostgreSQL infrastructure already has:

- persistent storage
- daily `pg_dump` backup at 02:00
- tested restore workflow

Before major deployment/release:

- run a manual PostgreSQL backup
- verify backup file exists
- run migrations only after backup where existing data could be affected

## Manual Deployment Workflow Preparation

Initial deployment remains manual:

```txt
Developer Mac
    ↓ git push
GitHub
    ↓ git pull
Ubuntu Server
    ↓
docker compose build
    ↓
docker compose up -d
```

CI/CD is optional future work and is not a blocker for the university submission.

## Tasks

- create frontend production Dockerfile
- create backend production Dockerfile
- create/update project `compose.yaml`
- join external Docker network `web`
- define internal backend/database networking
- create `.env.example`
- document real server `.env` values without committing secrets
- configure same-origin `/api` strategy where appropriate
- verify CORS requirements after proxy routing
- prepare Nginx reverse-proxy / gateway routes for X-Fly
- prepare X-Fly `server_name` for the approved `*.siwakondev.win` subdomain
- verify Named Cloudflare Tunnel route for the X-Fly hostname
- verify `cloudflared` runs as a persistent system service
- prepare Nginx origin TLS listener (`listen 443 ssl` inside the container)
- prepare loopback-only HTTPS host mapping such as `127.0.0.1:8443:443`
- prepare Let's Encrypt DNS-01 certificate lifecycle for `siwakondev.win` / `*.siwakondev.win`
- mount certificate material into Nginx read-only
- document automatic certificate renewal + safe Nginx reload procedure
- verify cloudflared → Nginx certificate validation; no permanent `noTLSVerify`
- define migration + seed commands
- add frontend/backend health checks
- verify restart policy
- verify production assets
- verify production builds
- verify logging/error pages
- verify source-map policy
- document manual deploy / rollback steps
- document pre-deploy backup step
- verify no unnecessary public ports
- configure/document `X-Fly - LOCAL` pgAdmin connection
- configure/document `X-Fly - PRODUCTION` pgAdmin connection through SSH Tunnel
- verify Local pgAdmin can inspect X-Fly schema/data
- verify Production pgAdmin can inspect X-Fly schema/data without exposing PostgreSQL publicly
- document clear warning that Local and Production data are separate

## Exit Criteria

- frontend image builds successfully
- backend image builds successfully
- project Compose config validates
- services can join expected Docker networks
- backend can resolve PostgreSQL internally
- Nginx upstream service names, internal ports, and routes are documented
- X-Fly public hostname under `siwakondev.win` is documented
- Named Tunnel route is documented
- Nginx origin TLS certificate strategy and renewal path are documented
- cloudflared → Nginx uses verified HTTPS origin in the target deployment
- `.env.example` contains no secrets
- production build passes
- pgAdmin can inspect Local X-Fly PostgreSQL directly through the local host port
- pgAdmin can inspect Production X-Fly PostgreSQL only through SSH Tunnel
- PostgreSQL remains non-public
- Local and Production databases are clearly separated in pgAdmin
- deployment can proceed without redesigning existing server infrastructure

---

# 71. BRANCH 31 — Self-Hosted Deployment

## Branch

```txt
chore/31-production-deploy
```

## Deployment Architecture

X-Fly university/demo deployment หลังมี domain แล้วให้ใช้ **fixed hostname + Named Cloudflare Tunnel + Nginx origin TLS** เป็น default path.

Preferred hostname:

```txt
https://x-fly.siwakondev.win
```

Target architecture:

```txt
Professor / Tester
        ↓
HTTPS
        ↓
Cloudflare Edge + DNS
        ↓
Named Cloudflare Tunnel
        ↓
cloudflared on Ubuntu Server
        ↓
HTTPS origin (certificate verified)
        ↓
127.0.0.1:8443
        ↓
Nginx :443 ssl
        ↓
Docker network: web
   ┌──────┴──────┐
   ↓             ↓
Frontend       Backend
Next.js        Rust + Axum
:3000          :8080
                  ↓
             PostgreSQL 18
                  ↓
          Persistent Storage
                  ↓
          Daily Backup 02:00
```

Nginx ต้องทำงานจริงใน 6 บทบาทตามความเหมาะสมของ X-Fly:

1. Reverse Proxy
2. Load Balancer (เมื่อมี backend replica > 1)
3. Web Server
4. TLS Termination at origin
5. Serve Static Files (เมื่อเหมาะกับ build/runtime)
6. Gateway หน้า backend/API services

Quick Tunnel เก็บไว้เฉพาะ fallback / temporary troubleshooting. ไม่ใช่ URL หลักสำหรับส่งอาจารย์เมื่อ Named Tunnel พร้อมแล้ว.

## Deployment Workflow

### 1. Server Preflight

```txt
ssh safe-host
docker ps
docker exec postgres pg_isready -U postgres
systemctl status cloudflared --no-pager
docker exec nginx nginx -t
```

ตรวจ:

- Docker daemon
- PostgreSQL
- Nginx
- `cloudflared` Named Tunnel service
- free disk space
- current backups
- expected Docker network `web`
- certificate files exist and are readable by the Nginx container
- certificate expiry / renewal state

### 2. Source Deployment

Project source มาจาก Git/GitHub:

```txt
git pull
```

หรือ clone ใน first deployment.

ไม่พัฒนา source code โดยตรงบน server.

### 3. Environment

- create/update server `.env`
- never commit secrets
- validate required variables
- verify backend database URL uses internal Docker hostname
- verify frontend uses same-origin `/api` where approved
- verify no LAN IP is hard-coded
- set the public application base URL to the approved `https://<x-fly-host>.siwakondev.win`
- keep Stripe in Test/Sandbox mode only

### 4. Database Safety

ก่อน migration สำคัญ:

- run manual `pg_dump`
- verify backup exists

จากนั้น:

- create X-Fly database/schema as planned
- run migrations
- run controlled demo seed if required
- verify migration status
- verify application DB permissions

### 4A. Verify Production Data with pgAdmin over SSH

Production database inspection ใช้ pgAdmin บน Mac ผ่าน SSH Tunnel เท่านั้น.

Concept:

```txt
Mac / pgAdmin
      ↓
SSH Tunnel
      ↓
safe-host
      ↓
127.0.0.1:5432
      ↓
PostgreSQL 18
      ↓
x_fly
```

Verification checklist:

- SSH connection to `safe-host` works
- PostgreSQL remains bound to `127.0.0.1:5432` on Ubuntu
- no PostgreSQL public Internet port is opened
- pgAdmin connection `X-Fly - PRODUCTION` connects successfully through SSH
- `x_fly` schema is visible
- migrations are visible in schema/history tables where applicable
- seed/demo records are visible
- application-created records can be inspected after smoke tests
- Local and Production pgAdmin connections are visually distinguishable

Production pgAdmin access is for controlled inspection/debugging only.

Do not use pgAdmin as an application dependency and do not route application traffic through pgAdmin.

### 5. Build & Start

```txt
docker compose build
docker compose up -d
```

ตรวจ:

```txt
docker compose ps
docker compose logs
```

Application containers ต้องใช้ restart policy ที่เหมาะสม เช่น `unless-stopped`.

### 6. Nginx + TLS Integration

Server-managed Nginx config อยู่ที่:

```txt
/srv/apps/proxy/nginx.conf
```

ต้อง:

- add/update X-Fly `server_name`
- route `/` to the X-Fly Next.js frontend
- route `/api/*` to the Rust backend
- configure `listen 443 ssl`
- reference the approved certificate + private key paths
- preserve proxy headers
- configure static-file handling only when it matches the production build
- configure `upstream` only when replicas actually exist
- validate config with `docker exec nginx nginx -t`
- reload only after validation succeeds

### 7. Origin TLS Verification

ทดสอบ HTTPS ที่ Nginx origin โดยไม่ bypass certificate verification.

Concept:

```txt
cloudflared
  ↓ HTTPS
127.0.0.1:8443
  ↓ SNI / originServerName matches certificate
Nginx :443 ssl
```

ตรวจ:

- certificate hostname/SAN matches the configured origin server name
- certificate is not expired
- private key permissions are restricted
- `cloudflared` does not rely on permanent `noTLSVerify`
- renewal hook validates and reloads Nginx safely

### 8. Named Cloudflare Tunnel

Named Tunnel ต้อง route approved X-Fly hostname ไป HTTPS origin:

```txt
x-fly.siwakondev.win
      ↓
Named Cloudflare Tunnel
      ↓
https://127.0.0.1:8443
```

`cloudflared` ควรรันเป็น persistent system service เพื่อให้ tunnel กลับมาหลัง reboot.

### 9. Local / Origin Smoke Test

ตรวจอย่างน้อย:

- homepage
- flight search
- flight results
- seat flow
- passenger/review/payment flow
- Stripe Test Mode payment flow
- ticket / QR verification route
- manage booking
- admin login/access boundary
- backend health
- database read/write
- static assets
- error pages
- HTTPS origin handshake

### 10. Public Verification

ทดสอบจาก Internet จริง:

- `https://x-fly.siwakondev.win` resolves and loads
- HTTPS works through Cloudflare
- cloudflared → Nginx origin TLS is verified
- Stripe Test/Sandbox webhook reaches `/api/v1/payments/stripe/webhook`
- a Test Mode payment can correlate Stripe Dashboard → webhook → X-Fly DB
- frontend loads from external network
- API requests work through Nginx
- no mixed-content errors
- QR / verification URLs use the fixed public base URL
- admin access restrictions behave as designed
- PostgreSQL is not publicly reachable
- no frontend/backend development ports are exposed unnecessarily
- no secrets appear in client bundles/logs
- responsive/mobile smoke test
- browser console clean

### 11. Demo Handoff

ก่อนส่งอาจารย์:

- keep Ubuntu Server powered on
- ensure Docker containers are healthy
- ensure `cloudflared` Named Tunnel service is healthy
- verify `x-fly.siwakondev.win` is the current fixed URL
- verify Nginx certificate validity
- perform one final external smoke test
- provide the fixed project URL to professor/tester

## Tasks

- deploy frontend container
- deploy Rust + Axum backend container
- connect backend to PostgreSQL 18
- run migrations
- run demo seed if required
- manual pre-release database backup
- integrate Nginx routes
- configure X-Fly `server_name`
- configure Nginx `listen 443 ssl`
- install/mount the approved certificate into Nginx
- configure/verify Let's Encrypt DNS-01 renewal path for the server certificate
- validate Nginx before reload
- verify HTTPS origin locally
- configure Named Tunnel hostname route
- verify `cloudflared` persistent service
- verify cloudflared origin certificate validation
- external Internet smoke test
- verify Stripe Test Mode webhook on the fixed HTTPS hostname
- verify QR/public URLs
- verify admin access/security rules
- verify health checks
- verify container restart behavior
- verify logs
- verify backup still operates
- verify certificate renewal/reload procedure
- verify `X-Fly - PRODUCTION` pgAdmin SSH Tunnel access
- inspect deployed schema and representative application records through pgAdmin
- confirm PostgreSQL is still not publicly exposed
- document the fixed X-Fly demo hostname

## Exit Criteria

Full public path works:

```txt
Internet
↓ HTTPS
Cloudflare Edge
↓ Named Cloudflare Tunnel
cloudflared
↓ HTTPS / verified certificate
Nginx TLS
↓
Frontend / Backend
↓
PostgreSQL
```

และ:

- booking E2E works on deployed environment
- database migrations are applied
- backup exists
- fixed `*.siwakondev.win` project hostname works
- HTTPS works via Cloudflare
- Nginx terminates TLS at origin successfully
- cloudflared verifies the Nginx origin certificate
- certificate renewal procedure is documented/testable
- PostgreSQL remains non-public
- pgAdmin on Mac can inspect Production through SSH Tunnel
- Local and Production data remain isolated
- application recovers correctly after container restart
- Named Tunnel recovers correctly after server reboot/service restart
- final external smoke test passes

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
13 Passenger + Travel Documents
↓
13A Travel Extras (historical; superseded by 20A)
↓
14 Review + Fare Conditions
↓
15 Mock Payment Foundation
↓
15B Stripe Test Payment
↓
16 Booking Success + Ticket Foundation
↓
17 Manage Booking / E-Ticket Retrieval
↓
17A Booking Confirmation Email (historical; superseded by 20A)
↓
18 Cancellation + Full Refund
↓
19 Admin Authentication + RBAC
↓
20 Executive Dashboard + Reports
↓
20A Customer Requirements Alignment
↓
21 Flight Management
↓
22 Booking Management
↓
23 Ticket / Passenger Operations
↓
24 API Client Management
↓
25 External REST API + Token / Scopes
↓
26 Security / Audit / Company Device Boundary
↓
27 Accessibility / Responsive / Cross-Browser
↓
28 Performance / Load Testing
↓
29 Production Readiness / E2E
↓
30 Deployment Preparation
↓
31 Self-Hosted Deployment
↓
32 Final Polish
↓
33 Documentation / Presentation
```

`17A` extends the historical roadmap without renumbering already completed numeric branches. `20A` is the latest stakeholder-alignment branch inserted after Branch 20 and before Branch 21; it supersedes the active product behavior introduced by historical Branches 13A and 17A where noted.

Customer online check-in and Boarding Pass branches are intentionally removed from the implementation roadmap because they belong to downstream airport operations, not the X-Fly booking/ticketing product.

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

The premium effect should come from the combination of:

1. Cinematic first impression
2. Strong typography
3. Aircraft media
4. Global 156-country storytelling
5. Moon future-vision storytelling
6. Smooth section choreography
7. Premium booking transitions
8. Business/First-only cinema-like cabin/seat selection
9. Server-authoritative seat-hold experience
10. Confident Stripe Test payment experience
11. Clear Booking Confirmed finish
12. Direct website Manage Booking recovery without email dependency
13. Secure returning-customer Manage Booking
14. Premium online E-Ticket with signed verification QR
15. Dark + Yellow identity
16. Realistic passenger/travel-document/fare-review flow
17. Executive analytics that answer real planning questions
18. Secure external API/token integration

The “wow” is not created by adding more lifecycle screens. Transactional and returning-customer flows should become simpler as the user gets closer to the booking outcome.

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

Core application architecture:

```txt
Next.js / React
      ↓
REST / HTTPS
      ↓
Rust + Axum
      ↓
SQLx
      ↓
PostgreSQL
```

Product context:

```txt
Customer Web
  ├── Booking
  ├── Payment
  ├── Booking Confirmed
  ├── Manage Booking / E-Ticket
  └── Cancellation / Refund

Employee Web
  ├── RBAC
  ├── Executive Analytics / Reports
  ├── Flight Management
  ├── Booking Management
  ├── Ticket / Passenger Operations
  └── API Client Management

External Systems
      ↓ Token / Scope
X-Fly External REST API
      ↓
Authoritative X-Fly Data
```

Airport check-in, boarding-pass lifecycle, gate processing, and staff QR-scanner applications are downstream external operations and are not implemented by X-Fly.

## Local Development

```txt
Mac
├── Next.js local
├── Rust + Axum local
├── pgAdmin
└── PostgreSQL Docker
```

## University / Demo Deployment

```txt
Developer Mac
      ↓
GitHub
      ↓
Ubuntu Server
      ↓
Docker Compose
      ↓
Named Cloudflare Tunnel
      ↓ HTTPS origin
Nginx TLS
   ┌──┴───────────┐
   ↓              ↓
Frontend        Backend
Next.js         Rust + Axum
                  ↓
             PostgreSQL 18
```

Public host:

```txt
https://x-fly.siwakondev.win
```

PostgreSQL remains private. Production inspection from Mac uses pgAdmin through SSH Tunnel only.

## Production-Scale Direction

The stakeholder production target prefers Cloud infrastructure and aggressive scale. The self-hosted university server is the current demo/learning deployment, not evidence of 100,000 req/s capacity.

The application must retain a migration path toward:

- multiple backend instances
- load balancing
- cloud networking
- managed/scaled data services where appropriate
- observability
- capacity planning

## Stripe

University/demo remains **Stripe Test Mode only**.

Local webhook development may use Stripe CLI.

Deployed demo webhook:

```txt
https://x-fly.siwakondev.win/api/v1/payments/stripe/webhook
```

No live Stripe keys or real charging belong in this academic deployment.

# 82. Final Recommendation

The goal is not to simulate every airline/airport subsystem.

The goal is:

> **Build a premium Airline Booking & Ticketing Platform that owns authoritative booking/ticket data, provides excellent customer booking/retrieval/cancellation experiences, gives X-Fly staff the right business visibility, and exposes secure integration APIs to downstream systems.**

Public experience:

- Landing should feel premium and global
- Business/First-only search and seat selection should be fast and interactive
- Passenger/contact capture should be minimal: required Male/Female gender + phone contact, with no customer email requirement
- Payment should feel trustworthy
- `Booking Confirmed` should clearly end the purchase
- Manage Booking should be concise, secure, and reachable directly from the website without email dependency
- E-Ticket/QR should be available after verification
- Cancellation/refund should follow the 24-hour rule

Internal experience:

- Executive dashboards answer planning/revenue/demand questions
- Flight managers alone control flight creation/editing
- Ticketing/baggage/booking staff receive role-appropriate information
- API admins govern external integrations

Integration:

- external systems authenticate using token/client credentials
- scopes and field minimization control what they may read
- direct database access is forbidden

Do not expand X-Fly into online check-in, boarding passes, airport gate systems, staff QR scanners, chatbot/support, loyalty points, or campaign management unless the stakeholder explicitly changes the requirement.

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

## Checkpoint A — after Branch 04

Review:

- Brand
- Typography
- Hero
- Motion quality

## Checkpoint B — after Branch 07

Review:

- full landing page
- global / Moon storytelling
- performance

## Checkpoint C — after Branch 12

Review:

- Search
- Flight
- Cabin
- Seat experience
- concurrency

## Checkpoint D — after Branch 18

Review complete customer booking/ticketing lifecycle:

- Passenger + Travel Documents + Phone Contact
- Business/First-only active cabin scope
- Male/Female-only required gender
- no active Travel Extras step
- Review + Fare Conditions
- Payment / Ticket separation
- Booking Confirmed summary
- no customer confirmation-email dependency
- Manage Booking
- E-Ticket / QR
- Cancellation / Refund

## Checkpoint E — after Branch 26

Review internal/integration architecture:

- Admin authentication / RBAC
- Executive analytics / reports
- Flight permissions
- Booking/Ticket operations
- API clients / token scopes
- company-device/security boundary
- auditability

## Checkpoint F — after Branch 29

Review:

- accessibility
- responsive/cross-browser
- load/performance evidence
- end-to-end readiness
- backup/recovery readiness
- legal/compliance requirement matrix

## Checkpoint G — after Branch 32

Final production/design review.

# 85. Success Criteria

X-Fly Anyway is successful when:

## Customer / Booking

- Landing has premium wow factor
- Motion is smooth and does not block usability
- Customer can book without registering/logging in
- Flight search and cabin/seat selection work on responsive devices
- new customer booking offers and accepts **Business and First only**
- Cinema-style seat plan clearly differentiates Business and First
- seat concurrency/hold rules are server-authoritative
- passenger/travel-document/contact data flow is realistic and protected; gender is required as Male/Female and new booking contact is phone-only
- Review shows fare conditions and cancellation policy clearly
- Stripe Test Mode Card works without exposing raw card data
- Mock Bitcoin remains clearly demo-only
- successful purchase ends at a concise **Booking Confirmed** summary
- Booking Reference and Ticket Number remain separate
- new bookings do not create/send customer confirmation email
- customer can return through website `Manage Booking` using Booking Reference + Last Name
- Manage Booking uses Booking Reference + Last Name and is anti-enumeration safe
- stale authorization can never display a different booking
- E-Ticket/booking details show authoritative data and signed verification QR
- QR contains no raw PII
- cancellation >=24h yields 0-fee / 100% refund
- cancellation <24h is rejected
- changing flight means cancel eligible old booking and create a new booking

## Product Boundary

- no customer online check-in implementation
- no Boarding Pass lifecycle
- no staff QR-scanner application
- no gate/baggage operational workflow
- no chatbot/customer-support module
- no loyalty/points
- no promotion/campaign builder
- marketing/external systems obtain only approved data through integration APIs

## Internal / Admin

- employee authentication and RBAC are enforced
- generic admin cannot edit flights unless granted FLIGHT_MANAGER permission
- executive can see revenue, bookings, load factor, nationality, route/demand trends
- daily/weekly/monthly/custom reports work
- Profit/Loss is shown only when cost data exists
- booking/ticket/baggage operations see only role-required fields
- privileged mutations are auditable
- company-device access policy has a realistic deployment enforcement boundary

## External Integration

- external API clients use credentials/tokens
- scopes restrict access
- credentials can be revoked/rotated
- tokens/secrets are stored safely
- direct PostgreSQL access is never given to external systems
- API has rate limiting/audit visibility
- field-level data minimization is enforced

## Non-Functional

- critical application/API interactions target ~1 second where measurable
- frontend Core Web Vitals targets remain defined
- responsive desktop/tablet/mobile behavior passes
- accessibility/reduced-motion passes
- load tests produce honest measured results
- architecture retains a cloud/horizontal-scaling path for the stakeholder scale target
- university self-hosted server is not falsely claimed to support 100,000 req/s without evidence
- backup/restore is tested
- PostgreSQL remains private
- legal/compliance requirements are documented per operating region rather than falsely claiming universal certification

## Deployment

- production containers build/run correctly
- Named Cloudflare Tunnel + verified HTTPS origin + Nginx TLS works publicly
- `x-fly.siwakondev.win` is the fixed demo hostname when approved/configured
- frontend/backend are not unnecessarily exposed on public host ports
- Stripe deployed webhook uses fixed public HTTPS and remains Test Mode
- pgAdmin Local and Production access remain correctly separated
- Production pgAdmin access uses SSH Tunnel only

## Coding-Agent Execution Guidance

Model/provider choice is operational tooling, not product architecture.

For future branches:

- use the strongest available reasoning agent for security, payment/refund, RBAC, API-token, concurrency, migration, and deployment work
- use medium reasoning for straightforward UI/forms
- always inspect repository state before handoff
- never allow two coding agents to modify the same working tree simultaneously
- every handoff begins with `git status` + `git diff`
- branch prompts and repository architecture documents override generic third-party skills
