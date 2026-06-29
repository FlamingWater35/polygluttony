<div align="center">

<img src="assets/welcome.png" alt="Polygluttony Next" width="860">

# Polygluttony Next

**LLM-powered subtitle translation for donghua & anime — built to protect the things that break.**

![Tauri 2](https://img.shields.io/badge/Tauri-2-FFC131?logo=tauri&logoColor=000)
![Rust](https://img.shields.io/badge/Rust-000?logo=rust&logoColor=fff)
![React](https://img.shields.io/badge/React-61DAFB?logo=react&logoColor=000)
![TypeScript](https://img.shields.io/badge/TypeScript-3178C6?logo=typescript&logoColor=fff)
![Tailwind CSS v4](https://img.shields.io/badge/Tailwind-v4-06B6D4?logo=tailwindcss&logoColor=fff)
![TanStack](https://img.shields.io/badge/TanStack-Router%20%7C%20Query-EF4444?logo=reactquery&logoColor=fff)

<div style="display: flex; gap: 8px; justify-content: center; margin: 16px 0;">
  <span style="display: inline-block; background: #e1a636; color: black; padding: 4px 12px; border-radius: 12px; font-size: 13px; font-weight: 500;">macOS</span>
  <span style="display: inline-block; background: #e1a636; color: black; padding: 4px 12px; border-radius: 12px; font-size: 13px; font-weight: 500;">Windows</span>
  <span style="display: inline-block; background: #e1a636; color: black; padding: 4px 12px; border-radius: 12px; font-size: 13px; font-weight: 500;">Linux</span>
</div>

<div style="display: flex; gap: 12px; justify-content: center; margin: 20px 0;">
  <a href="https://github.com/blyat-uk/polygluttony" style="display: inline-block; background: #24292e; color: white; padding: 10px 24px; border-radius: 8px; text-decoration: none; font-weight: 600; font-size: 15px; transition: all 0.2s; box-shadow: 0 2px 8px rgba(0,0,0,0.15);">
    <svg style="display: inline-block; vertical-align: middle; margin-right: 8px; width: 20px; height: 20px; fill: currentColor;" viewBox="0 0 16 16">
      <path d="M8 0C3.58 0 0 3.58 0 8c0 3.54 2.29 6.53 5.47 7.59.4.07.55-.17.55-.38 0-.19-.01-.82-.01-1.49-2.01.37-2.53-.49-2.69-.94-.09-.23-.48-.94-.82-1.13-.28-.15-.68-.52-.01-.53.63-.01 1.08.58 1.23.82.72 1.21 1.87.87 2.33.66.07-.52.28-.87.51-1.07-1.78-.2-3.64-.89-3.64-3.95 0-.87.31-1.59.82-2.15-.08-.2-.36-1.02.08-2.12 0 0 .67-.21 2.2.82.64-.18 1.32-.27 2-.27.68 0 1.36.09 2 .27 1.53-1.04 2.2-.82 2.2-.82.44 1.1.16 1.92.08 2.12.51.56.82 1.27.82 2.15 0 3.07-1.87 3.75-3.65 3.95.29.25.54.73.54 1.48 0 1.07-.01 1.93-.01 2.2 0 .21.15.46.55.38A8.013 8.013 0 0016 8c0-4.42-3.58-8-8-8z"/>
    </svg>
    Original Repository
  </a>
  <a href="../../releases/latest" style="display: inline-block; background: #2ea043; color: white; padding: 10px 28px; border-radius: 8px; text-decoration: none; font-weight: 600; font-size: 15px; transition: all 0.2s; box-shadow: 0 2px 8px rgba(46, 160, 67, 0.3);">
    ⬇️ Download Latest
  </a>
</div>

*Note: This is a fork of the original Polygluttony project. Please support the [original developer](https://github.com/blyat-uk).*

</div>

---

Polygluttony Next translates `.ass` subtitle files with an LLM while guarding against the failure modes that wreck naive machine translation. Point it at a folder, connect a provider (Anthropic, Gemini, Openrouter, OpenAI, or any OpenAI-compatible endpoint), optionally build a glossary, and run — watching live, honest telemetry the whole way.

## Why it's different

- **Line markers & partial-failure recovery** — every line is tracked, so when a model drops, merges, or reorders lines, Polygluttony Next detects exactly where it broke and salvages the correct prefix instead of failing the whole batch.
- **Drift detection** — a five-signal weighted detector catches translations wandering off the source mid-batch and retranslates only the part that drifted.
- **Byte-faithful ASS tags** — `{\pos}`, `{\an8}`, fonts, styles, and metadata come back exactly as they went in; only the dialogue is translated.
- **Cross-episode glossary** — a six-category glossary, with auto-detected world type (xianxia / wuxia / historical / modern), keeps names and terms consistent across a whole season.
- **Verification, not a score** — every file checks its own work and surfaces an actionable issue list, never a number.
- **Mission-control UI** — a single window with live, batched telemetry: watch batches land, terms stream into the glossary, and drift get caught in real time.

## Download

Grab the latest build for your OS from the [**Releases**](../../releases/latest) page — macOS (Apple Silicon), Windows, and Linux.

> These builds aren't signed with a paid developer certificate, so the OS warns on first launch:
>
> - **macOS** — the first launch is blocked. Open **System Settings → Privacy & Security**, scroll to the bottom, and click **Open Anyway**, then launch again and confirm. (Right-click → Open no longer works on recent macOS.)
> - **Windows** — on the SmartScreen prompt, choose **More info → Run anyway**.
> - **Linux** — `chmod +x Polygluttony-Next-*.AppImage`, then run it.

## Build from source

Requires [Bun](https://bun.sh) and [Rust](https://rustup.rs) (stable), plus the [Tauri prerequisites](https://tauri.app/start/prerequisites/) for your OS.

```bash
bun install
bun tauri dev      # run with hot reload
bun tauri build    # produce a distributable bundle
```

## Development

To run generators:

```bash
bun run gen:routes
bun run gen:bindings
```

## License

[MIT](LICENSE.md)
