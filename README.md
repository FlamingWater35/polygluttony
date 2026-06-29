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

## Platforms

![macOS](https://img.shields.io/badge/macOS-000000?logo=apple&logoColor=white)
![Windows](https://img.shields.io/badge/Windows-0078D4?logo=windows&logoColor=white)
![Linux](https://img.shields.io/badge/Linux-FCC624?logo=linux&logoColor=black)

## Links

[![Original Repository](https://img.shields.io/badge/Original-Repository-24292E?logo=github&logoColor=white&style=for-the-badge)](https://github.com/blyat-uk/polygluttony)
[![Download](https://img.shields.io/badge/Download-Latest-2EA043?logo=github&logoColor=white&style=for-the-badge)](../../releases/latest)

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
