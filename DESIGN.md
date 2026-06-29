---
name: polygluttony-next
description: Imperial Command Deck for LLM subtitle translation — full-overdrive, honest batched telemetry in Shēng Luó gold on warm obsidian.
colors:
  obsidian: "#0c0a07"
  ink-base: "#100c08"
  surface: "#16110a"
  raised: "#201810"
  hover: "#2a2114"
  border: "#2c2316"
  border-strong: "#3a2f1c"
  text-dim: "#6f6347"
  text-muted: "#9c8b69"
  text-primary: "#ecdfc6"
  text-emphasis: "#fff4dc"
  gold: "#e1a636"
  gold-hi: "#ffd98a"
  gold-deep: "#8f661f"
  jade: "#9bd6a0"
  teal: "#79c0c0"
  aqua: "#6cc7d6"
  amber: "#f0b53e"
  coral: "#ff8a8a"
typography:
  title:
    fontFamily: "Figtree Variable, system-ui, sans-serif"
    fontSize: "1.125rem"
    fontWeight: 600
    lineHeight: 1.2
    letterSpacing: "-0.02em"
  heading:
    fontFamily: "Figtree Variable, system-ui, sans-serif"
    fontSize: "1rem"
    fontWeight: 500
    lineHeight: 1.3
    letterSpacing: "-0.005em"
  body:
    fontFamily: "Figtree Variable, system-ui, sans-serif"
    fontSize: "0.875rem"
    fontWeight: 400
    lineHeight: 1.5
    letterSpacing: "normal"
  label:
    fontFamily: "Figtree Variable, system-ui, sans-serif"
    fontSize: "0.6875rem"
    fontWeight: 500
    lineHeight: 1.2
    letterSpacing: "0.02em"
  mono:
    fontFamily: "ui-monospace, 'Geist Mono', SFMono-Regular, Menlo, monospace"
    fontSize: "0.78rem"
    fontWeight: 400
    lineHeight: 1.5
    fontFeature: "tabular-nums"
rounded:
  sm: "6px"
  md: "8px"
  lg: "10px"
  xl: "14px"
  2xl: "18px"
  4xl: "26px"
  pill: "13px"
spacing:
  1: "4px"
  2: "8px"
  3: "12px"
  4: "16px"
  6: "24px"
components:
  button-primary:
    backgroundColor: "{colors.gold}"
    textColor: "{colors.obsidian}"
    rounded: "{rounded.4xl}"
    height: "36px"
    padding: "0 14px"
  button-outline:
    backgroundColor: "{colors.border-strong}"
    textColor: "{colors.text-primary}"
    rounded: "{rounded.4xl}"
    height: "36px"
    padding: "0 14px"
  input:
    backgroundColor: "{colors.border-strong}"
    textColor: "{colors.text-primary}"
    rounded: "{rounded.4xl}"
    height: "36px"
    padding: "4px 12px"
  card:
    backgroundColor: "{colors.surface}"
    textColor: "{colors.text-primary}"
    rounded: "{rounded.2xl}"
    padding: "24px"
  batch-cell:
    backgroundColor: "{colors.raised}"
    textColor: "{colors.text-primary}"
    rounded: "{rounded.lg}"
    padding: "11px 14px"
  state-chip:
    backgroundColor: "transparent"
    textColor: "{colors.text-primary}"
    rounded: "{rounded.pill}"
    padding: "3px 10px"
  chip-accent:
    backgroundColor: "transparent"
    textColor: "{colors.gold}"
    rounded: "{rounded.pill}"
    padding: "2px 10px"
  nav-item-active:
    backgroundColor: "{colors.raised}"
    textColor: "{colors.gold}"
    rounded: "{rounded.md}"
    padding: "8px 0"
---

# Design System: polygluttony-next

## 1. Overview

**Creative North Star: "Imperial Command Deck"**

polygluttony-next is the command deck of an imperial star-junk hauling a season of subtitles across the dark. Fansubbing has only ever had grey, boxy tools; this is the opposite — a deck that **glows from within**, lit by a single imperial gold (**Shēng Luó 盛螺**, the amber-gold of a Chinese dragon) against warm obsidian, with a living atmosphere of drifting embers and a faint star-chart behind the readouts. You set the coordinates — a folder, a provider, a glossary — and then you *watch the work come back*, **batch by batch**: a chunk dispatched to the model, the suspense of the wait, the snap of ~200 lines landing and passing their marker check, the amber re-fire when drift is caught. The job is the voyage; the deck is the wall of legible, living readouts that makes a 40-minute automated run feel **commanded** — cinematic, and honest.

This is **full overdrive**: a WebGL/Canvas atmosphere, shader-grade gold bloom, telemetry choreographed like a film — but every photon of it is **earned by real work.** The personality is **daring, extravagant, extrovert**, and never childish: no RGB, no neon-gamer glow, no cartoon. Extravagant in *atmosphere and motion*; calm and exact in the *controls*.

One law governs everything: **the spectacle renders the truth.** The unit of telemetry is the **batch**, not the line; counts **jump** when a batch lands; the in-flight shimmer lasts exactly as long as the model takes; drift is a trigger the user only sees as a *consequence*. Decoration that signals nothing — and motion that misrepresents the work — are the two things this system forbids itself.

It explicitly rejects the **generic AI-tool aesthetic** (purple/blue gradients, glow-for-glow's-sake, glassmorphism, sparkles — and the cold AI-blue we deliberately left behind), **childish maximalism** (RGB, gamer-neon, cartoon, bouncy motion), the **consumer-SaaS marketing look** (gradient hero CTAs, hero-metric templates, identical card grids), and the **cluttered enterprise dashboard** (chart-junk, boxy grey panels, no air).

**Key Characteristics:**

- **Dark-only, warm-obsidian and gold** — there is no light theme; the warmth in every neutral is what makes the dark read as *lit* rather than grey.
- **One signal, and it is gold** — Shēng Luó marks actions, focus, the active state, and live work; nothing else, ≤10% of any screen.
- **Telemetry is the hero surface** — the Translate run view (the batch pipeline) and the status strip are the marquee, built to be watched.
- **Honest by construction** — the batch is the unit; counts jump; internals (the drift score) stay internal; verify is a list, never a score.
- **Living atmosphere** — a state-reactive ember/star-chart backdrop that intensifies only while a run is live and idles near-still otherwise.
- **Earned familiarity under a bold skin** — soft-pill maia controls (shadcn) and Phosphor icons behave conventionally; the drama lives in atmosphere, not reinvented widgets.

## 2. Colors

A warm, near-black obsidian foundation lit by a single imperial-gold signal and a small vocabulary of meaningful semantic states.

### Primary — Shēng Luó Gold

- **Gold** (`#e1a636`, ~`oklch(0.78 0.13 78)`): the live signal. Primary buttons, focus rings, progress fill, the active nav item, the in-flight batch, and any "this is happening now" indicator. Imperial amber-gold — the colour of a dragon, never orange. It is the one wavelength the eye is meant to track across the dark.
- **Gold Highlight** (`#ffd98a`): the bloom core, hover state, and landed-spark — gold at its brightest, used for emphasis on already-gold surfaces.
- **Gold Deep** (`#8f661f`): the recessed companion — primary-button base tone, the foot of the progress gradient, selection backgrounds. Presence without shouting.

### Semantic states

The engine's state machine speaks in color (each is *always* paired with a dot/glyph and a label — see The State-Reads-Twice Rule):

- **Translating** → **Gold** `#e1a636` — live, in-flight.
- **Retranslating / drift consequence** → **Amber** `#f0b53e` — a batch re-firing after a failed check; also "Needs a look" warnings.
- **Cleanup** → **Teal** `#79c0c0` — the residual-source cleanup pass.
- **Verifying** → **Aqua** `#6cc7d6` — the self-check pass.
- **Done / clean** → **Jade** `#9bd6a0` — landed, verified, no issues.
- **Failed** → **Coral** `#ff8a8a` — errors, failed runs, destructive hover.
- **Pending / Waiting** → **Muted** `#9c8b69`.

The warm "producing" phases (gold, amber) and the cool "finishing & terminal-good" phases (teal, aqua, jade) read as two families at a glance; coral is the one warm-red that means *bad*.

### Neutral — The Warm-Obsidian Ladder

Depth is a stack of warm near-blacks, deepest at the back; every step carries a hair of gold:

- **Obsidian** (`#0c0a07`): the window field, the nav rail, the status strip — the floor of the deck.
- **Ink Base** (`#100c08`): the deck's base gradient.
- **Surface** (`#16110a`): inputs, tables, cards.
- **Raised** (`#201810`): raised cards, header sections, icon tiles, the active nav tile, batch cells.
- **Hover** (`#2a2114`): interactive hover wash.
- **Border** (`#2c2316`): gold-tinted hairlines. **Border Strong** (`#3a2f1c`): input borders, scrollbar handles.
- **Text Dim** (`#6f6347`): queued items, faint labels. **Text Muted** (`#9c8b69`): secondary/helper text, counts. **Text Primary** (`#ecdfc6`): body. **Text Emphasis** (`#fff4dc`): headings and emphasis.

### Named Rules

**The Signal Rule.** Gold appears on roughly ≤10% of any screen — actions, focus, the current selection, live work. Its rarity against the obsidian is *why* it reads as a signal. Flood the screen with it and it stops meaning "look here."

**The Shēng Luó Rule.** Gold (盛螺) **is** the accent — full stop. This replaces the legacy Signal Blue; the accent migration is now design-canonical and is the next change in code. Canonical value: `#e1a636` / `oklch(0.78 0.13 78)`, deep companion `#8f661f`, highlight `#ffd98a`. Every "live signal" surface — focus ring, progress fill, active nav, primary CTA, the status-strip underglow, the signal bloom — is gold. It was chosen deliberately to escape the generic-AI cold blue and to tie the deck to the xianxia/wuxia worlds the tool translates.

**The Warm-Obsidian Ladder Rule.** Elevation is expressed by stepping *up* the warm-obsidian ladder, never by darkening a shadow (the one exception is the deck frame — see §4). The faint gold in every neutral is load-bearing: pull it out and the dark goes flat and grey, and the console illusion dies.

**The State-Reads-Twice Rule.** Every semantic color is paired with a non-color cue — a dot, a glyph, and/or a text label. State must be legible on a glance with no color memory. Hue alone never carries meaning.

**The Living-Atmosphere Rule.** The ember/star-chart backdrop is the one ambient field that may glow without a discrete event — but it is *still telemetry*: it tracks **run liveness**, brightening while work runs and idling to near-still otherwise. It stays low-contrast behind everything and pauses off-screen; it never competes with the readouts.

## 3. Typography

**Body / UI Font:** Figtree Variable (with `system-ui, sans-serif`)
**Monospace Font:** the system mono stack via `font-mono` — `ui-monospace, Geist Mono, SFMono-Regular, Menlo` (Geist Variable is bundled and is the intended home for a dedicated mono once wired).

**Character:** One humanist sans carries everything — titles, labels, body, buttons. Figtree is friendly without being soft, which lets the *atmosphere* be extravagant while the *text* stays calm and readable. Monospace is reserved for **machine truth**: file paths, line markers (`<0001:D>`), batch labels and line ranges, prompt tokens, and dialogue source/target columns.

### Hierarchy

A tight, **fixed rem** product scale (no fluid clamp headings — a desktop tool views at consistent DPI):

- **Title** (600, `1.125rem`/18px, `tracking-tight`): the page-header `h1`. A touch more present than before — the deck has a head — but it never shouts like a landing page.
- **Heading** (500, `1rem`/16px): card titles and section headers (`font-heading` = Figtree).
- **Body** (400, `0.875rem`/14px, line-height 1.5): primary reading text; prose columns cap at 65–75ch, data tables run denser.
- **Label** (500, `0.6875rem`/11px): chips, status-strip text, nav-rail labels (10px) — the dense telemetry layer.
- **Mono** (400, `~12.5px`, tabular): paths, line markers, batch labels, code, dialogue cells.

### Named Rules

**The Tabular Telemetry Rule.** Every number that updates live or sits in a column — counts, ETAs, file/batch tallies, line totals, percentages, latency timers — uses `font-variant-numeric: tabular-nums`. Because counts **jump by batch** (§5), tabular figures keep the jump clean with no horizontal reflow; a console with dancing, re-laying-out numbers is a broken console.

## 4. Light, Glow & Depth

Depth is **warm tone plus emitted gold light.** Inner surfaces still separate by stepping up the obsidian ladder, and a faint inset ring bounds cards — but in overdrive the deck also *emits* light, and that light is always meaningful.

### Light / Glow Vocabulary

- **Gold focus glow** (`box-shadow: 0 0 0 3px rgba(225,166,54,0.45)` + gold border): the focus treatment on inputs, buttons, and interactive controls — the system's "you are here."
- **Card ring** (`box-shadow: inset 0 0 0 1px rgba(236,223,198,0.08)`): the faint hairline that defines a card without lifting it.
- **Signal bloom** (shipped, not "planned"): a soft gold bloom behind genuinely-live elements — the in-flight batch cell, the reactor fill, the active state dot, the live activity chip. It pulses only while work is happening, and settles flat when idle.
- **The deck frame** — the single permitted cinematic drop-shadow. The whole console floats on a deep outer shadow with an inner gold rim-light, so it reads as a physical deck in the dark. *No other element gets a drop shadow.*
- **The ember atmosphere** — the WebGL/Canvas field behind everything (§5): drifting gold embers, a faint star-chart, a low bottom bloom.

### Named Rules

**The Tone-Plus-Bloom Rule.** Lift an inner surface by tone or the inset ring — never a drop shadow. The only shadow in the system is the deck frame. Light that *appears* (a glow, a bloom) comes from the gold signal and means "live."

**The Meaningful-Glow Rule.** A glow exists only where something is live — focus, the active signal, in-flight work, or the run-reactive atmosphere. A glow that decorates a static element is forbidden; that is precisely the "generic AI-tool" tell this product rejects.

## 5. Motion & Telemetry

Motion here is not polish applied at the end; it **is** the product. It obeys one law above all: **it renders the real work.** The reference implementation of every rule below is the Translate run view.

### The unit is the batch

A file is ~250–300 lines, split into batches of ~100–200 → **1–3 batches per file.** The **batch**, not the line, is the unit of every readout. A file is shown as a short stack of **batch cells**, each moving `queued → in-flight → landed` (or an amber re-fire). Never animate per-line flow — the work is chunky request→response, and the telemetry must say so.

### The in-flight wait

While a batch is out with the model, its return time is **unknown** — so its cell shows an **indeterminate gold shimmer** with a ticking **latency timer** (`awaiting model · 3.6s`), never a fake percentage. The shimmer is the suspense; honesty means we don't pretend to know progress we don't have. (If a provider streams tokens, the shimmer may gain a coarse fill — but the line count still commits only on landing.)

### The landing

When a batch returns and parses, it **snaps**: a ~600 ms marker-check sweep runs across the cell, it settles to **jade with `markers ✓`**, and the file's translated-line count **jumps** by the batch size in one step. This snap is the payoff the wait sets up — the single most important moment of motion in the app.

### Counts jump — they never roll

Because the batch is the unit, the translated-lines count **holds during the wait and jumps ~200 on landing.** A smooth per-line roll is forbidden: it is a lie about how the work happens. Tabular numerals keep the jump clean.

### Drift stays internal; you see the consequence

The 5-signal drift detector is an **internal** retranslation trigger. Its score never appears on screen — verify is an issue list (never a score), and a drift number means nothing to the user. What the user sees is the **consequence**: a batch that fails its check **re-fires in amber** — `batch 2 · drift caught → retranslating` — visible, legible, honest.

### Progress is discrete

The run **reactor** advances by *batches landed / total batches*, drawn with **discrete tick marks** (one per batch) and stepping as each batch lands. No smooth interpolated fill pretending to know sub-batch progress.

### The living atmosphere

Behind the deck, a WebGL/Canvas field of drifting gold embers, a faint star-chart, and a low bottom bloom. It is **state-reactive**: lively only while a run is active, near-still at idle — so even the atmosphere obeys "a glow means something is happening." It sits low-contrast behind everything and **pauses off-screen**.

### Signature moments

- **The hero batch pipeline** (the active file): batch cells with the in-flight → landing choreography. The marquee.
- **The run-integrity ring**: meaningful counts — `7/11 batches · 1 retranslated` — telling the user how hard the model struggled, never an invented metric.
- **The reactor bar**: discrete batch ticks; gold fill with flowing energy and bloom while live, flat when idle.
- **The status strip**: a glowing console line with a gold underglow; the live activity segment (`file 3/5 · batch 2/2`) is clickable straight to the run.
- **View-transition morphs**: rail navigation morphs the focused view (shared-element where it helps); the deck never hard-cuts.

### Timing & ceiling

Controls keep the snappy **150–250 ms** product band (hover, press, focus, open/close). Telemetry has its **own honest rhythm**: the wait lasts as long as the model takes; the landing is a ~600 ms sweep; the atmosphere breathes on a multi-second cycle. The technique ceiling is **full overdrive** — WebGL/Canvas backdrop, shader-grade bloom, Canvas-drawn telemetry, View Transitions — implemented progressively: idle cheap, pause off-screen, lazy-init near viewport, target **60fps** on mid-range hardware. **No `prefers-reduced-motion` gate** — a deliberate product decision (see PRODUCT.md); motion is the point, and performance discipline replaces the a11y fallback.

## 6. Components

Built on shadcn's **maia** style (soft-pill controls) with **Phosphor** icons. Every interactive component ships default, hover, focus-visible, active, and disabled.

### Buttons

- **Shape:** soft pill — `rounded-4xl` (26px) on a 36px-tall button reads as a lozenge, not a rectangle.
- **Primary:** Gold fill, obsidian text, weight 500; hover lifts the fill toward Gold Highlight.
- **Press:** `active:translate-y-px` — the button physically nudges down 1px. Small, tactile, essential to the "real console" feel.
- **Focus:** gold border + 3px gold/45 ring glow.
- **Outline / Ghost / Secondary / Destructive:** outline is a faint `border-strong` on a 30%-input wash; ghost is transparent → `hover` wash; destructive is a tinted coral surface (`coral/10` → `/20`), never a solid red block.

### State Chip — the canonical state primitive

A colored **dot + label** whose color is driven by the engine state. This is the canonical state-color map — reuse it everywhere state is shown:

| State | Color | Note |
|---|---|---|
| Pending / Waiting | Muted `#9c8b69` | static |
| Translating | Gold `#e1a636` | dot pulses; live |
| Retranslating | Amber `#f0b53e` | dot pulses; drift re-fire |
| Cleanup | Teal `#79c0c0` | |
| Verifying | Aqua `#6cc7d6` | |
| Done | Jade `#9bd6a0` | ✓ |
| Needs a look | Amber `#f0b53e` | ⚠ |
| Failed | Coral `#ff8a8a` | |

### Status Chip

Border + label, variants `muted | amber | coral | jade | gold`. Border and text take the semantic color at 40% / full; the fill stays transparent so chips sit quietly on any obsidian layer. Tabular numerals.

### Batch cell — the atomic unit of the run view

A telemetry **row** (not a card — see Cards): `[ BATCH n · line range | meter | status ]` on a `raised` ground, `rounded-lg`. States:

- **queued** — dimmed (text-dim), empty meter.
- **in-flight** — gold border, indeterminate gold shimmer in the meter, status `awaiting model · {t}s`.
- **landing** — a one-shot white marker-check sweep across the cell (~600 ms).
- **done** — jade border, full jade meter, status `landed · markers ✓`.
- **retrans** — amber border, amber shimmer, status `drift caught → retranslating`.

### Run-integrity ring

A gold donut (stroke-dasharray driven), center `n/total` batches, sublabel `k retranslated`. A meaningful run-level readout of how much rework happened — **not** a quality score.

### Reactor bar — the run progress

~10px tall; gold gradient fill (`gold-deep → gold → gold-hi`) with flowing diagonal energy and bloom while live, flat when idle. **Discrete batch tick-marks** overlay the track; the fill advances in steps as batches land.

### Cards / Containers

- **Corner:** `rounded-2xl` (18px) — soft, well short of pill.
- **Background:** `surface`; **no drop shadow** (only the deck frame casts one), bounded by the 1px inset ring.
- **Padding:** 24px default (`sm` size → 16px). Cards are **never nested.** The hero run-card is the one structured container that holds telemetry **rows** (batch cells) — those are primitives, not nested cards.

### Inputs / Fields

- **Shape:** soft pill (`rounded-4xl`, 26px), 36px tall, on a translucent `border-strong` / input wash.
- **Focus:** gold border + 3px gold/45 ring glow — the same focus language as buttons.
- **Disabled:** 50% opacity, no pointer events. **Invalid:** coral border + ring.
- Helper text (`HelpText`) sits one line below, muted, optionally with a leading "ⓘ" in gold.

### Navigation — the icon rail

- A fixed **vertical rail** (64–80px) on `obsidian`, crowned by the **Shēng Luó dragon seal**, split into a workflow group (Project, Glossary, Translate) and a setup group (Connections, Prompts, Help) with a flex spacer between.
- **Item:** stacked Phosphor icon + 10px label in a rounded tile. **Active:** `raised` tile, gold icon with a soft bloom, Phosphor `fill` weight (inactive = `regular`). **Hover:** `hover` wash. **Disabled (gated):** 50%-muted with a tooltip explaining why ("Open a folder first").
- Per-item badges: a count (glossary terms) or a status glyph (✓/⚠ on Connections).

### Status Strip — the marquee telemetry line

A full-width **~32–34px footer** on `obsidian` with a thin gold underglow, pinned across the bottom of the shell. Reads left-to-right like a console status line: folder · file/line counts · live activity chip (`⏳ file 3/5 · batch 2/2`) · world-type · language pair · connection · core version. Tabular numerals throughout; the live activity chip is clickable and routes to the running view. This is the surface the Imperial identity is built to make spectacular.

### App Shell

A CSS grid — `grid-cols-[auto_1fr] grid-rows-[1fr_auto]` — with the rail spanning both rows on the left, scrollable `main` top-right, and the status strip pinned bottom-right. The whole window is the **deck frame** (the one cinematic shadow + inner gold rim-light) floating over the ember atmosphere. Setup panels (Project/Connections editors) center in a ~560–640px reading column; run/data views fill.

## 7. Do's and Don'ts

### Do

- **Do** build depth from the **warm-obsidian ladder** + meaningful **gold bloom**; reserve the system's only drop-shadow for the **deck frame**.
- **Do** treat gold (**Shēng Luó**) as the one signal — actions, focus, current selection, live work — and keep it to ≤10% of any screen.
- **Do** make the **batch** the unit of every readout: `queued → in-flight (latency timer) → landing (snap + marker sweep) → done / amber re-fire`.
- **Do** **jump** counts on a batch landing, with `tabular-nums`; never roll per-line.
- **Do** keep internals internal — surface drift only as a **consequence** (retranslating), never a score; verify is an issue list.
- **Do** make atmosphere and bloom **state-reactive and idle-cheap**; pause off-screen; target 60fps.
- **Do** pair every state color with a **dot/glyph + text label** (The State-Reads-Twice Rule).
- **Do** keep controls conventional: soft-pill maia buttons/inputs, Phosphor icons, standard nav. Drama lives in atmosphere, not reinvented widgets.

### Don't

- **Don't** drift into the **generic AI-tool aesthetic** — no purple/blue decorative gradients, no glassmorphism by default, no sparkles, no `background-clip: text` gradient text — and **don't** fall back to the cold AI-blue we deliberately left.
- **Don't** go **childish** — no RGB, no gamer-neon, no cartoon, no bouncy/elastic easing. Bold ≠ loud-and-immature.
- **Don't** **fake telemetry**: no smooth per-line rolling counts, no interpolated progress pretending to know sub-batch state, no invented metrics, no motion that doesn't track a real event.
- **Don't** put internal machinery on screen as a number (the drift score) — the user sees consequences, not gauges.
- **Don't** animate per-line flow for batched work.
- **Don't** add a `box-shadow` to lift an inner surface (only the deck frame casts one); **don't** emit a glow that signals nothing.
- **Don't** build a light theme — dark-only is the identity.
- **Don't** round cards past ~18px, nest cards, or introduce a display/serif font into UI labels, buttons, or data — one Figtree family carries the interface.
- **Don't** let motion block input, gate content behind a class-triggered reveal, or make the user watch a page-load sequence. Extravagant, never obstructive.
