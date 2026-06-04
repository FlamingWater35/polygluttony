# Step 2 — Welcome screen + Project view (folder pickup)

**Status:** Approved design (brainstorming) — ready for implementation plan
**Date:** 2026-06-04
**Project:** polygluttony (Python `subs-translate` → Tauri 2 / Rust + React rewrite)
**Builds on:** Step 1 (app shell + Connections). This step adds the **first two workflow
views** — the **Welcome** landing screen and the **Project** (per-folder setup) view —
and the read-side engine they need (ASS reader, world detection, languages, discovery,
per-folder persistence).

---

## 1. Goal & deliverable

From a usable app (a connection exists), the user lands on a real **Welcome** screen:
recent folders + an **Open folder** button + **drag-and-drop** a folder onto the window.
Opening a folder runs discovery/analysis and navigates to the **Project** view, where the
user confirms **source/target language** (source auto-detected from the files), an
**auto-detected world type** (overridable), a **tone**, and a **file list** with per-file
**selection checkboxes** and `✓`-translated badges. From there two forks: **Build a
glossary** (when the source supports it) or **Translate now**. All per-folder choices and
the recent-folders list persist in a backend Tauri store.

This is **read-only**: Step 2 discovers, counts, detects, and remembers. It does not
translate or write `.ass` files.

### Done criteria
- `cargo build` and `bun run build` both green; ts-rs bindings regenerated.
- Rust unit tests pass: charset decode (UTF-8 / UTF-8-BOM / GBK / UTF-16), dialogue
  parse + count (comma/tag preservation, malformed-skip), tag strip, world detection
  (the Python xianxia/wuxia/historical/modern + tie-break cases), language
  resolve/validate/equal + source-language detection, file discovery
  (exclude translations, sort, `has_translation`), and the project store
  (recents MRU/cap/prune, folder-prefs round-trip).
- Cold flow: with a usable connection, app opens on **Welcome**; opening a folder of
  `.ass` files lands on **Project** with real file + dialogue-line counts, a detected
  source language and world type, and the rail's workflow items un-dimmed.
- Re-opening a previously opened folder restores its saved language/world/tone/selection.

---

## 2. Non-goals (explicit scope boundaries)

**Deferred to Step 3 (translation/write), already decided — recorded so it is not
relitigated.** The save mechanism for a translated file:
- Copy the original **header verbatim** (everything up to `[Events]`), **dropping**
  `[Aegisub Project Garbage]` and `[Fonts]`/`[Graphics]`.
- Inject one comment line directly under the `[Script Info]` header:
  `; Translated at home with Polygluttony`.
- Keep `[V4+ Styles]` as-is (pruning unused styles is optional, not required).
- **Regenerate** `[Events]` containing only the translated `Dialogue:` lines, **sorted by
  start time** (stable for equal start times).
- Preserve any sections that appear **after** `[Events]` only insofar as we choose; per
  requirements, losing fonts/graphics is acceptable.
- The **positional tag strip + reapply** (`AssTagStripper` port) — needed to send
  tag-free text to the LLM and restitch `{\pos}`/`{\an8}`/`{\i1}` afterward — lands here
  too. Step 2 ships only a **plain** strip (for detection text), not the positional one.

**Deferred to later steps:** glossary build (O10), translation (O16), cleanup pass,
verify (O18), Settings (O20), Help content, file-watch.

The ASS engine in this step is **only** the reader: decode → parse `Dialogue:` lines →
count → plain-strip text for detection. No writer, no reapply, no styles model.

---

## 3. Locked decisions (from brainstorming)

1. **Scope:** Welcome + Project, **read-only**. The writer is Step 3 (design recorded in §2).
2. **ASS library = none for the format; bottom-up.** Use **`encoding_rs` + `chardetng`**
   for robust decoding (the two best-in-class, maintained crates — the exact ones aspasia
   wraps), and **our own small `ass` module** for parsing. Rationale: the only hard part
   is charset detection; a `Dialogue:` line is a split-on-first-9-commas; and we must
   hand-write the positional tag reapply (Step 3) and the writer regardless, so owning the
   trivial parse keeps one coherent, fully-controlled, latest-deps module. Surveyed
   alternatives were worse fits: **ass-core/ass-rs/ass-editor** take `&str` (UTF-8 only —
   can't read GBK/Big5 donghua rips) and are early (v0.1.1); **subparse** (v0.7) handles
   encodings but its value is *non-destructive in-place editing* (the opposite of our
   drop-garbage/sort/inject-comment model) and exposes only a generic timespan+text
   interface; **aspasia** (v0.2.1, June 2024, ~13k dl) is a fine reader but stale and we'd
   use ~⅓ of it. (See Step-1 brainstorming notes / this doc's history.)
3. **Persistence = backend Tauri store.** A second store file (`projects.json`),
   Rust-owned, holds recents + per-folder prefs. Mirrors the `config.json` pattern so the
   Translate step reads the same prefs. Not localStorage.
4. **Drag-and-drop = in scope.** A folder dropped on the window routes to the same
   open-folder flow (the first dropped path; `open_folder` validates it's a directory).
5. **File list = per-file selection checkboxes** + `✓`-translated badges + totals. The
   selection persists per folder; it is *acted on* only in Translate (Step 3).
6. **One `open_folder` command bundles O6/O7/O8** — discover + count, detect source
   language, detect world type, record recent, load saved prefs — returning one
   `ProjectView`. Counts are real (every file parsed on open, in `spawn_blocking`).
7. **World detection is instant, keyword-only** (no LLM), run on every folder open;
   the result is overridable and the override persists per folder.

---

## 4. Architecture overview

```
Welcome / Project view ──invoke──▶ Tauri command ──▶ ass / glossary / config / store
                                     (commands/)        (ass/, glossary/, config/, models/)
        ▲                                                        │
        └────────────────── typed return (ts-rs) ────────────────┘
```

The webview never touches the filesystem directly; folder picking uses the
`tauri-plugin-dialog` JS API and drag-drop uses the core webview drag-drop event — both
yield a path that is handed to `open_folder`. All reading/parsing/persistence is in Rust.

### Backend layout (`src-tauri/src/`)
- `ass/` *(new)* — `decode` (charset detection + decode), `parse` (`Dialogue:` lines →
  `DialogueLine`), `tags` (plain strip). The complete 10-field parse is written **once**
  here; Step 3 reuses it and adds the writer + positional tags.
- `glossary/` *(new)* — `world_detector` (keyword counting → `WorldType`).
- `config/languages.rs` *(new)* — the language table + alias resolution + source-language
  detection. `models/language_pair.rs` *(new)* — `LanguagePair::from_codes`.
- `config/projects.rs` *(new)* — the `projects.json` store: recents + per-folder prefs.
- `utils/` *(new or folded into a command module)* — `discover` (`.ass` discovery +
  existing-translation detection).
- `commands/project.rs` *(new)* — `open_folder`, `list_languages`, `save_folder_prefs`,
  `list_recents`, `remove_recent`, `clear_recents`.
- `models/` — new ts-rs DTOs (§7).

### Frontend layout (`src/`)
- `routes/index.tsx` — becomes the real **Welcome** (no more hard redirect).
- `features/welcome/` *(new)* — the Welcome view + recents hook.
- `routes/project.tsx` + `features/project/` *(new)* — the Project view (replaces the
  current `EmptyState` placeholder) + project hook.
- `components/` — `RecentItem` (folder tile), reuse `SetupField`/`HelpText`/`StatusChip`.
- `stores/app-store.ts` — add per-folder fields the shell needs (world type, counts,
  has-untranslated / has-translated for gating).
- `lib/ipc.ts` — typed wrappers for the new commands.
- `tauri.conf.json` — capabilities for dialog + drag-drop.

---

## 5. Backend design

### 5.1 ASS reader (`ass/`)

**`decode`** (`ass/decode.rs`)
- `decode_file(path) -> Result<String, AppError>`: `std::fs::read` the bytes; if a BOM is
  present (UTF-8 / UTF-16 LE / UTF-16 BE) honor it; otherwise feed the bytes to a
  `chardetng::EncodingDetector` and `guess(None, true)` to pick an
  `encoding_rs::Encoding`; decode with that encoding (lossy-replacement on malformed
  sequences, never erroring). Returns a `String`. (This is the one place we improve on the
  Python's UTF-8-only `open()`.)

**`parse`** (`ass/parse.rs`)
- Section-aware single pass over decoded lines. Track the current `[Section]`. Within
  `[Events]`, a line whose trimmed start is `Dialogue:` (case-insensitive) is parsed:
  drop the `Dialogue:` prefix, `splitn(10, ',')` on the remainder → exactly 10 parts.
  Fewer than 10 ⇒ malformed ⇒ skip (not counted).
- `DialogueLine { layer: i64, start_cs: i64, end_cs: i64, style: String, name: String,
  margin_l: i64, margin_r: i64, margin_v: i64, effect: String, text: String }`. `text` is
  part 10 verbatim (commas and override tags preserved). `start_cs`/`end_cs` parse
  `H:MM:SS.cc` → centiseconds (used for sorting in Step 3; computed now since it's free).
  Non-numeric numeric fields default to 0 rather than failing the line.
- `Comment:` and all non-dialogue lines are ignored. (Mirrors Python `get_dialogue_count`
  = count of `Dialogue:` lines; matches what the status bar's "¶N lines" means.)
- Public for Step 2: `parse_dialogues(text: &str) -> Vec<DialogueLine>`;
  `dialogue_count` = `.len()`. (`DialogueLine` keeps all fields so Step 3 reuses it.)

**`tags`** (`ass/tags.rs`)
- `strip_for_text(s: &str) -> String`: remove ASS override blocks via `regex` `\{[^}]*\}`,
  leaving plain dialogue text. Used only to build clean text for world + language
  detection. (The positional `AssTagStripper`/`AssTagResult` port — strip *with positions*
  + `reapply` — is Step 3.)

**Python source of truth:** `parsers/ass_parser.py` (`parse`/`parse_dialogue_line`,
splitn-on-9-commas), `parsers/ass_file.py` (`get_dialogue_count`,
`get_all_dialogue_text`), `parsers/dialogue_line.py`, `parsers/ass_tags.py` (tag regex
`\{\\[^}]+\}` — we use the broader `\{[^}]*\}` for detection text).

### 5.2 World detector (`glossary/world_detector.rs`)
Pure keyword counting (no LLM), ported verbatim from `glossary/world_detector.py`.

- `WorldType` enum: `Xianxia | Wuxia | Historical | Modern` (ts-rs, serde kebab/lower:
  `xianxia|wuxia|historical|modern`).
- `detect(content: &str, supports_world_detection: bool) -> WorldType`:
  if `!supports_world_detection` ⇒ `Modern`. Else sum `content.matches(kw).count()` over
  each category's keyword list; take the max; ties break in order **xianxia > wuxia >
  historical**; max == 0 ⇒ `Modern`.

Keyword lists (port exactly — Chinese, substring, case-sensitive):
```
XIANXIA = 修仙 筑基 金丹 元婴 渡劫 灵气 仙人 修炼 灵石 丹药 法宝 飞剑 结丹 化神 修真 仙界 魔界 灵根 天劫 飞升
WUXIA   = 武林 江湖 门派 内力 轻功 武功 剑法 掌法 拳法 气功 真气 武者 侠客 大侠 盟主 帮派
HISTORICAL = 皇帝 朝廷 太监 皇后 大臣 科举 宰相 王爷 公主 皇宫 后宫 朝代 太子 皇上 圣旨
```
Input = concatenation of every file's `strip_for_text(dialogue.text)` joined by `\n`
(we already parse all files for counts; reuse that text).

### 5.3 Languages (`config/languages.rs`, `models/language_pair.rs`)
Port `config/languages.py` (`Language` + `LANGUAGES`) and `models/language_pair.py`.

- `Language { code, name, aliases: Vec<String>, output_suffix, character_pattern:
  Option<String>, supports_glossary: bool, supports_world_detection: bool }` (ts-rs).
- The 15-entry table (only `zh` has glossary + world detection). `character_pattern`
  values are Unicode **codepoint ranges** — implement with the `\u`-escape forms from
  `config/languages.py` (zh = `[一-鿿]`, ru/bg = `[Ѐ-ӿ]`,
  ko = `[가-힯ᄀ-ᇿ]`, etc.); the literal glyphs in the table below are
  only for readability:

| code | name | aliases | suffix | character_pattern | glossary | world |
|---|---|---|---|---|---|---|
| zh | Chinese | chi, chs, cht, cn, chinese | chi | `[一-鿿]` | ✓ | ✓ |
| ko | Korean | kor, korean | kor | `[가-힯ᄀ-ᇿ]` | | |
| ja | Japanese | jpn, japanese | jpn | `[぀-ゟ゠-ヿ一-鿿]` | | |
| en | English | eng, english | eng | — | | |
| es | Spanish | spa, spanish | spa | — | | |
| fr | French | fra, french | fra | — | | |
| de | German | ger, deu, german | ger | — | | |
| pt | Portuguese | por, portuguese | por | — | | |
| ru | Russian | rus, russian | rus | `[Ѐ-ӿ]` | | |
| ar | Arabic | ara, arabic | ara | `[؀-ۿ]` | | |
| th | Thai | tha, thai | tha | `[฀-๿]` | | |
| vi | Vietnamese | vie, vietnamese | vie | — | | |
| id | Indonesian | ind, indonesian | ind | — | | |
| ms | Malay | msa, malay | msa | — | | |
| bg | Bulgarian | bul, bulgarian | bul | `[Ѐ-ӿ]` | | |

- `resolve_language_code(code) -> Option<String>`: lowercase+trim; direct code match, else
  alias match.
- `detect_source_language(text) -> Option<String>`: for each language with a
  `character_pattern`, count matching chars in `text`; return the highest-scoring code
  (None if no script chars found → UI falls back to the saved/global default). Powers the
  Project view's "Detected from the files."
- `LanguagePair::from_codes(source, target) -> Result<LanguagePair, AppError>`: resolve
  both; error if either unknown or if `source == target` (mirrors the Python `ValueError`).
  `LanguagePair` carries `output_suffix`, `supports_glossary`, `supports_world_detection`
  from the **source** language. Output/warning filename helpers (`{stem}.{suffix}.ass`,
  `{stem}.warning.{suffix}.ass`) + `strip_language_suffix` are ported now (discovery needs
  them); `get_target_suffixes` = `[output_suffix]` (+ `target` if different).

### 5.4 File discovery (`utils/discover.rs`)
Port `utils/file_utils.py`.
- `discover_source_files(dir, &LanguagePair) -> Vec<PathBuf>`: non-recursive `*.ass`
  (case-insensitive), **excluding** any file whose stem ends with `.{suffix}` for a target
  suffix or whose stem contains `.warning.`; sorted.
- `has_existing_translation(src, &LanguagePair) -> bool`: the output path or warning path
  exists (`find_existing_translation` checks each target suffix + its `.warning.` variant).
- Per source file the command assembles `SourceFile { path, name, dialogue_count,
  has_translation }`.

### 5.5 Project store (`config/projects.rs`)
A second Tauri store file `projects.json` (same `StoreExt` pattern as `config/store.rs`),
Rust-owned, with pure read-modify-write helpers + thin Tauri glue (so the helpers are unit
testable without a running app, mirroring Step 1's `store.rs`).

- `RecentFolder { path: String, file_count: u32, last_opened: i64 }` (epoch seconds).
  Recents are an MRU list, **cap 10**, and **pruned of paths that no longer exist** on read.
- `FolderPrefs { source_lang: String, target_lang: String, world_override:
  WorldType | null, tone: Tone, selected_files: Vec<String> }`. Keyed by absolute folder
  path. **Precedence** (resolved by `open_folder`, see §5.6): a folder's **saved** prefs
  win outright; only when a folder has *no* saved prefs are they seeded —
  `source_lang` = detected → global default → `zh`; `target_lang` = global default → `en`;
  `world_override` = null (use the auto-detected world); `tone` = `Standard`;
  `selected_files` = `[]` (empty = "all files selected"). "Global default" =
  `AppConfig.default_source` / `default_target` from Step 1.
- `Tone` enum: `Standard | Xianxia | Wuxia | Comedic | Funny` (ts-rs, lower/kebab).
- Helpers: `record_recent(path, file_count, now)`, `list_recents` (prune missing),
  `remove_recent`, `clear_recents`, `get_prefs(path)`, `set_prefs(path, prefs)`.

### 5.6 Commands (`commands/project.rs`)
All return `AppResult<…>` with ts-rs payloads.

| Command | Op | Behavior |
|---|---|---|
| `open_folder(path, now) -> ProjectView` | O6/O7/O8 | validate `path` is a directory (else friendly `AppError`); load saved `FolderPrefs` (if any); discover source files using the resolved `LanguagePair`; in `spawn_blocking`, decode+parse each file for `dialogue_count` and accumulate stripped text; `detect_source_language`; `detect` world type; `record_recent`. **Resolve `prefs`** per §5.5 precedence (saved win; else seed from detected/global) and return them *already resolved* alongside the raw `detected_source_lang` and `detected_world` (so the UI can show the "auto-detected" badge and offer reset). The effective world = `prefs.world_override ?? detected_world`. |
| `list_languages() -> Vec<Language>` | O8 | the static table for the selects |
| `save_folder_prefs(path, FolderPrefs)` | — | persist language/world/tone/selection edits |
| `list_recents() -> Vec<RecentFolder>` | — | MRU, missing pruned |
| `remove_recent(path)` | — | drop one entry |
| `clear_recents()` | — | wipe the recents list |

`open_folder` takes `now` (epoch seconds) from the caller so the command stays
deterministic/testable (`Date.now()/1000` on the JS side).

Register all six in `lib.rs`'s `invoke_handler!`.

### 5.7 Capabilities (`src-tauri/`)
Add the capability permissions the new frontend affordances need: `dialog:allow-open`
(folder picker) and the core webview **drag-drop** event. Verify against the installed
plugin versions; the Rust-side file reads use `std::fs` (not the `fs` plugin) so no `fs`
scope is required.

---

## 6. Frontend design

### 6.1 Welcome (`routes/index.tsx` + `features/welcome/`)
Replace the Step-1 redirect-to-Connections with the real Welcome view (the rail already
shows Connections `⚠` and dims workflow items when there's no usable connection / folder).

- **First-run** (no usable connection): the hero + the two numbered steps —
  ① **Connect an AI provider** (button → `/connections`; marked done once a connection has
  a key), ② **Open a folder** (button → picker). Copy per `windows/01-welcome.md`.
- **Returning** (usable connection): hero + **Recent folders** list (`RecentItem`:
  folder icon, name, monospace path, "N files", relative time) + **Open folder** button +
  **"↓ or drag a folder onto the window"** hint. Empty recents → just the button + hint.
- Actions: Open folder → `tauri-plugin-dialog` `open({ directory: true })` → `open_folder`
  → navigate `/project`. Recent row click → `open_folder` → `/project`. Window drag-drop
  of a folder → first path → `open_folder` (a dropped *file* yields the friendly
  not-a-directory error as a toast). **Clear recents** (confirm) + per-row remove.
- Edge cases (from `windows/01-welcome.md`): a folder with **zero** `.ass` files → inline
  "No subtitle files found here", stay on Welcome; a folder with only already-translated
  files → still opens Project (Translate gated, Verify enabled).

### 6.2 Project (`routes/project.tsx` + `features/project/`)
Replace the placeholder `EmptyState`. Layout per `windows/03-project.md`.

- **Header:** folder name + **[change]** (→ Welcome) + context `StatusChip`s (language
  pair, world type, active connection).
- **SetupFields:**
  - **Source language** select. Value = `prefs.source_lang` (the backend already resolved
    saved → detected → global → zh, §5.6). HelpText "Detected from the files; change if
    it's wrong"; when no override differs from `detected_source_lang`, show an
    "auto-detected" affordance.
  - **Target language** select (value = `prefs.target_lang`). Inline error if
    `source == target` (mirrors `from_codes`).
  - **World type** select with an "auto-detected" badge; shown **only** when the source
    `supports_world_detection` (zh today). Value = effective world
    (`prefs.world_override ?? detected_world`); changing it sets `world_override`.
  - **Tone** select (`Standard|Xianxia|Wuxia|Comedic|Funny`).
- **File list:** one row per source file — selection **checkbox** + name + `✓`-translated
  badge; a header row with select-all + the totals ("12 subtitle files · ¶N lines").
- **Fork actions:** **Build a glossary** (→ `/glossary`) shown only when
  `supports_glossary`; **Translate now** (→ `/translate`) always present. Copy/recommended
  hints per the doc.
- **Persistence:** any change to language/world/tone/selection writes `FolderPrefs` via
  `save_folder_prefs` (debounced); on mount, values come from the `open_folder` result.
- **Non-glossary source** (anything but zh): hide the world-type field and the
  glossary fork; Translate still works.

### 6.3 Shell glue
- `stores/app-store.ts`: add `worldType`, `tone`, `fileCount`, `dialogueLineCount`,
  `hasUntranslated`, `hasTranslated` (plus existing `workdir`, source/target lang). The
  `open_folder` result is server-state (TanStack Query, keyed by path); the store holds
  the cross-view slices the **status bar** and **rail gating** read.
- **Status bar:** fill folder + file/line counts + language pair + world chip.
- **Rail gating** (refine `nav-rail.tsx` per `03-operations-and-flows.md`): Project &
  Glossary enabled when a folder is open; **Translate** enabled when a usable connection
  exists **and** ≥1 untranslated file; **Verify** enabled when ≥1 translated file exists.
  Disabled items keep their gating tooltip. **Translate is never gated on the glossary.**

### 6.4 IPC + hooks
- `lib/ipc.ts`: `openFolder`, `listLanguages`, `saveFolderPrefs`, `listRecents`,
  `removeRecent`, `clearRecents` wrappers.
- `features/welcome/use-recents.ts` and `features/project/use-project.ts`: TanStack Query
  hooks (recents query + open-folder/prefs mutations; invalidate recents on open/remove/clear).

---

## 7. Data shapes crossing the seam (ts-rs)

New `#[derive(TS)]` types exported to `src/types/generated/`:
- `Language { code, name, aliases: string[], output_suffix, character_pattern: string|null,
  supports_glossary: boolean, supports_world_detection: boolean }`
- `WorldType` = `"xianxia" | "wuxia" | "historical" | "modern"`
- `Tone` = `"standard" | "xianxia" | "wuxia" | "comedic" | "funny"`
- `SourceFile { path, name, dialogue_count: number, has_translation: boolean }`
- `FolderPrefs { source_lang, target_lang, world_override: WorldType|null, tone: Tone,
  selected_files: string[] }`
- `RecentFolder { path, file_count: number, last_opened: number }`
- `ProjectView { folder: string, files: SourceFile[], total_dialogue_lines: number,
  detected_source_lang: string|null, detected_world: WorldType, prefs: FolderPrefs,
  supports_glossary: boolean }`

Run `bun gen:bindings` after the Rust types land; verify `src/types/generated/`.

---

## 8. Testing strategy

**Rust (unit, no network), with `.ass` fixtures:**
- **decode:** UTF-8, UTF-8-with-BOM, GBK (Chinese), UTF-16LE → correct decoded string;
  malformed bytes don't panic.
- **parse:** dialogue count over a known fixture; `Comment:`/non-dialogue lines excluded;
  commas **and** override tags preserved in `text`; malformed (<10 field) lines skipped;
  `start_cs` parsed correctly (`0:00:01.50` → 150).
- **tags:** `strip_for_text` removes `{\pos(1,2)\an5}` / `{\i1}` and leaves plain text.
- **world_detector:** the Python cases — xianxia/wuxia/historical samples each detect
  correctly; no-keyword → modern; tie → xianxia>wuxia>historical; `supports=false` →
  modern.
- **languages:** `resolve_language_code` (code + alias, case-insensitive);
  `LanguagePair::from_codes` valid / unknown / equal-error; `detect_source_language`
  (Chinese text → `zh`, Latin → `en`/None).
- **discovery:** a temp dir with `ep01.ass`, `ep01.eng.ass`, `ep02.ass`,
  `ep03.warning.eng.ass`, `ep03.ass` → sources = `ep01, ep02, ep03` sorted; `has_translation`
  true for ep01 & ep03; translation/warning files excluded from sources.
- **project store:** recents MRU ordering + cap-10 + prune-missing; `FolderPrefs`
  get/set round-trip + defaults.

**Toolchain:** `cargo build`, `bun gen:bindings`, `bun run build` all green.

**Manual:** open a real folder of `.ass` files; confirm counts, detected source/world,
recents persistence, drag-drop, and that re-opening restores saved prefs.

---

## 9. Risks & follow-ups
- **Charset edge cases:** chardetng can misguess on very short files; honoring BOM first
  and decoding lossily (never erroring) keeps it robust. Re-encoding on write is a Step-3
  concern (we'll write UTF-8).
- **`open_folder` latency on large folders:** parsing every file on open is fine for
  typical sizes (tens of KB × tens of files = ms); it runs in `spawn_blocking`. If a
  pathological folder is slow, a later step can stream counts — log/note, don't silently cap.
- **`projects.json` vs `config.json`:** two store files; keep the helpers pure +
  unit-tested like Step 1. Folder paths are absolute keys — normalize before lookup.
- **Capabilities:** Tauri v2 dialog + drag-drop permissions must be added or the picker /
  drop silently no-ops; verify against installed plugin versions early.
- **First-run flow change:** removing the index→Connections redirect must keep the
  no-connection path obvious (Welcome first-run step ① + rail `⚠`).
- **Selection semantics:** `selected_files` empty vec = "all selected"; document so
  Step 3 doesn't misread an empty selection as "none."

---

## 10. Python / docs source-of-truth map

| Concern | Python reference |
|---|---|
| Dialogue parse / count | `parsers/ass_parser.py` (`parse_dialogue_line`, splitn-9-commas), `parsers/ass_file.py:get_dialogue_count`, `parsers/dialogue_line.py` |
| Combined dialogue text | `parsers/ass_file.py:get_all_dialogue_text` (+ `parsers/ass_tags.py`) |
| World detection | `glossary/world_detector.py` (`XIANXIA/WUXIA/HISTORICAL` lists, `detect`) |
| Languages table | `config/languages.py` (`Language`, `LANGUAGES`, `resolve_language_code`) |
| Language pair / suffixes | `models/language_pair.py` (`from_codes`, `get_output_filename`, `strip_language_suffix`, `get_target_suffixes`) |
| File discovery / existing translation | `utils/file_utils.py` (`discover_ass_files`, `discover_files_for_translation`, `has_existing_translation`, `find_existing_translation`) |

UI references: `polygluttony-docs/windows/01-welcome.md`, `windows/03-project.md`,
`windows/00-shell-rail-statusbar.md`, `01-design-system.md`,
`03-operations-and-flows.md` (O6/O7/O8, per-folder state, gating, event contract),
`02-core-logic-reference.md` (ASS parsing, languages, world detection).
