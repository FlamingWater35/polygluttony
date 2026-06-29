# Product

## Register

product

## Users

Fansubbers, subtitle localizers, and translation hobbyists-to-pros who batch-translate `.ass` subtitle files across many episodes with an LLM. Their context is a focused desktop session — often a long-running automated job they kick off and then **monitor**. They care about three things the naive approach gets wrong: **consistency** (names and terms stable across hundreds of episodes), **correctness** against real LLM failure modes (dropped, merged, reordered, or drifting lines), and **fidelity** (inline override tags, styling, and metadata returned byte-faithfully). They range from a first-time fansubber doing one show to a power user pushing a whole season through in one sitting.

## Product Purpose

polygluttony-next is a cross-platform desktop app (Tauri 2 + a Rust engine + a React 19 UI) that translates ASS subtitles with an LLM while protecting the things that break. It tracks every line with markers and salvages partial failures, detects mid-batch content drift with weighted signals, preserves `{\pos}`/`{\an8}`/font/style tags exactly, keeps a six-category glossary for cross-episode consistency, and verifies the output as an **actionable issue list — never a score**. Success is a translated subtitle set a human would ship with minimal touch-up, produced from a **single window**: pick a folder, connect a provider, optionally build a glossary, and run — watching legible, live, honest telemetry the whole way.

## Brand Personality

**Imperial Command Deck.** Fansubbing has only ever shipped grey, boxy tools — Aegisub, Subtitle Edit, the rest. polygluttony-next is the opposite: a gold-lit sci-fi command deck that glows from within. A single imperial gold — **Shēng Luó (盛螺)**, the amber-gold of a Chinese dragon — burns against warm obsidian, with a living atmosphere of drifting embers behind the readouts. It is **daring, extravagant, extrovert**, and theatrical — but never childish: no RGB, no neon-gamer glow, no cartoon. Extravagant in *atmosphere and motion*; calm and exact in the *controls*. Three words: **spectacular, legible, honest.**

## Anti-references

- **Generic AI-tool aesthetic** — purple/blue gradients, neon-glow-for-its-own-sake, glassmorphism by default, sparkles, gradient text. The 2026 "AI app" cliché. (We chose imperial gold precisely to escape the cold AI-blue everyone else defaults to.)
- **Childish maximalism** — RGB, gamer-neon, cartoon mascots, bouncy/elastic motion, emoji confetti. Bold is not the same as loud-and-immature; the deck is extravagant the way a film is, not the way a toy is.
- **Consumer SaaS marketing look** — big gradient CTAs, hero-metric templates, identical icon-card grids, landing-page polish bolted onto a working tool.
- **Cluttered enterprise dashboard** — chart-junk, boxy grey panels, everything bordered, no breathing room.

## Design Principles

- **Telemetry tells the truth.** The spectacle renders the *real* work and nothing else. The unit is the **batch**, not the line; the translated count **jumps** when a batch lands; the in-flight shimmer lasts exactly as long as the model takes; a retranslation shows itself. No faked smooth fills, no invented metrics, no motion that doesn't track a real event. Dishonest telemetry is as forbidden as decoration that signals nothing.
- **Telemetry as spectacle, never as noise.** Every long-running operation earns vivid, live feedback — progress, state, counts, ETAs — but the feedback serves comprehension first and atmosphere second.
- **Correctness is the product — internals stay internal.** The UI surfaces the engine's hard-won safeguards (line markers, drift detection, tag fidelity, glossary consistency) as understandable *consequences* — "drift caught → retranslating" — not as raw machinery. Verify is an issue list, never a score; drift is a trigger, never a number on screen.
- **Spectacular, never obstructive.** Motion and effects amplify state; they never delay input, gate content, or make the user wait for choreography. The job stays the user's to control.
- **Earned familiarity under a bold skin.** Standard affordances — navigation, forms, tables, dialogs — behave exactly as a user fluent in good tools expects. The extravagance lives in atmosphere, color, and motion, not in reinvented controls.
- **One window, no modes.** Navigation swaps focused views; gating *guides* the user (a tooltip on a disabled rail item) instead of throwing error dialogs.

## Legibility, Motion & Performance

**Dark-only and gold-lit.** The Imperial Command Deck is dark by nature — there is **no light theme**, and the earlier "coming light theme" is dropped from scope. **Legibility is a product value, not a compliance checkbox:** body text stays readable against the warm obsidian, and **state always reads twice** — every status pairs a color with a dot or glyph *and* a text label, so a glance reads it without relying on hue.

**Motion is always on.** This app deliberately does **not** target WCAG and does **not** gate animation behind `prefers-reduced-motion` — accessibility is out of scope here, and the spectacle is the point. Performance discipline replaces the motion fallback: the living atmosphere idles to near-still when no run is active, off-screen canvases pause, heavy resources lazy-init near the viewport, and the deck targets **60fps on mid-range hardware**. Focus stays visible via the gold ring — because that is good tooling, not because of a target.
