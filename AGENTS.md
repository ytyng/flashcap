# FlashCap - Project Guide

macOS screenshot capture & annotation app.

## Commands

- `pnpm install` - Install dependencies
- `pnpm tauri dev` - Start development server
- `pnpm tauri build` - Production build
- `pnpm check` - TypeScript type check

## Architecture

- **Frontend** (`src/`): SvelteKit 2 + Svelte 5 (runes syntax), TypeScript
- **Backend** (`src-tauri/`): Rust, Tauri 2.x
- **Pages**: `src/routes/+page.svelte` - Main capture UI
- **Components**: `src/lib/ArrowOverlay.svelte` - Arrow annotation overlay
- **Types**: `src/lib/types.ts`

## Key Details

- Screenshots are saved to `/tmp/flashcap/`
- ESC key exits the app
- Arrow tool for annotation
- Clipboard copy support (image-png feature enabled)

## Framework Note

SvelteKit 2 (Svelte 5 runes) and Tauri 2.x are new frameworks. Use **context7 MCP** to look up current API docs before making changes.
