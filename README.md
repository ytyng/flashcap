# FlashCap

A macOS screenshot capture & annotation app built with Tauri 2.x and SvelteKit.

## Features

- Screenshot capture
- Arrow annotation tool
- Clipboard integration (copy screenshot to clipboard)
- Keyboard shortcuts (ESC to quit)
- Auto-save to `/tmp/flashcap/`
- Open save folder from toolbar

## Tech Stack

- **Frontend**: SvelteKit 2, Svelte 5, TypeScript
- **Backend**: Rust (Tauri 2.x)
- **Build**: Vite, pnpm

## Prerequisites

- Rust (stable)
- Node.js
- pnpm
- macOS

## Development

```bash
pnpm install
pnpm tauri dev
```

## Build

```bash
pnpm tauri build
```

## Type Check

```bash
pnpm check
```

## Project Structure

```
src/               # SvelteKit frontend
  routes/          # Pages
  lib/             # Shared components & types
src-tauri/         # Rust backend (Tauri)
  src/             # Rust source
```

## Note for AI Assistants

SvelteKit 2 (with Svelte 5 runes) and Tauri 2.x are relatively new frameworks. When working on this project, use **context7 MCP** to look up the latest API documentation.
