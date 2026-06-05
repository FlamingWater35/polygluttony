# polygluttony

LLM-powered subtitle translation for donghua and anime — a cross-platform
desktop app (Linux / macOS / Windows). A ground-up rebuild of the Python
`subs-translate` project on a Tauri + Rust core with a React frontend.

It translates `.ass` subtitle files through a three-pass pipeline — glossary
extraction, translation with validation, and quality verification — against any
Anthropic, OpenAI, or OpenAI-compatible LLM provider.

## Stack

**Backend (Rust, in `src-tauri/`)**
- Tauri 2 shell — exposes the engine to the webview via commands + events
- The translation engine lives in `src-tauri/src/` (no separate crate, no CLI):
  `ass/` (subtitle parsing + tag preservation), `llm/` (Anthropic / OpenAI /
  OpenAI-Responses drivers over `reqwest`), `glossary/` (extraction, world-type
  detection, diff), `validation/` (marker checks + five-signal drift detector),
  `translation/` (token-aware batching, concurrency, verification)
- `tokio`, `futures`, `tiktoken-rs`, `regex`, `tracing`, `thiserror`/`anyhow`
- Plugins: `store` (config), `dialog` (folder picker), `fs`, `notification`,
  `opener`
- `ts-rs` generates TypeScript types for the IPC boundary into
  `src/types/generated/`

**Frontend (React, in `src/`)**
- React 19 + Vite + TypeScript
- Tailwind v4 + shadcn/ui (radix base, `maia` style), Phosphor icons
  (`@phosphor-icons/react`), `motion`
- TanStack Router (file-based, `src/routes/`) + TanStack Query
- Zustand (app state), react-hook-form + zod (forms), sonner (toasts)
- Single-window shell (icon rail + header + status bar). Feature screens in
  `src/features/`: `connections`, `welcome`, `project` (built); `glossary`,
  `translate`, `verify`, `settings` (later)

## Prerequisites

- [Bun](https://bun.sh)
- Rust (stable) + the Tauri system dependencies for your OS — see
  <https://tauri.app/start/prerequisites/>

## Development

```bash
bun install              # install frontend dependencies
bun tauri dev            # run the desktop app (Vite + Rust, hot reload)
```

Useful scripts:

```bash
bun run build            # tsr generate -> tsc -> vite build (frontend only)
bun run gen:routes       # regenerate the TanStack route tree
bun run gen:bindings     # regenerate ts-rs TypeScript bindings from Rust types
bun tauri build          # produce a distributable bundle for the current OS
```

When you change a Rust type that crosses the IPC boundary (anything deriving
`TS`), run `bun run gen:bindings` to refresh `src/types/generated/`.

## License

MIT
