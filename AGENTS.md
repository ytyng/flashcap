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
- **Components**:
  - `src/lib/ArrowOverlay.svelte` - Arrow annotation overlay
  - `src/lib/MaskOverlay.svelte` - Mask (mosaic/blur/fill) overlay
- **Types**: `src/lib/types.ts`
- **Preferences**: `src/routes/preferences/+page.svelte`

## Key Details

- Screenshots are saved to `/tmp/flashcap/` (configurable in Preferences)
- ESC key exits the app
- Arrow tool for annotation (with white stroke, drop shadow options)
- Mask tool: mosaic, blur, fill modes with 8-direction resize handles
- Timer capture: `screencapture -i -T <delay>` via async Rust command (delay configurable in Preferences)
- Clipboard copy support (image-png feature enabled)
- Settings stored via `tauri-plugin-store` (`settings.json`)

## Rust Commands (src-tauri/src/lib.rs)

- `take_screenshot_interactive` - Standard interactive capture (`screencapture -i`)
- `take_screenshot_timer` - Timer capture (`screencapture -i -T <N>`, async to avoid UI freeze)
- `write_image_to_file` - Save annotated image (path restricted to save directory)
- Common result loading: `load_screenshot_result()` shared by both capture commands

## Build & Check

- `cargo check` in `src-tauri/` for Rust type check
- `pnpm check` for Svelte/TypeScript check
- Run both before committing
- Production build: `cargo build --release` in `src-tauri/` (run before push)

## Framework Note

SvelteKit 2 (Svelte 5 runes) and Tauri 2.x are new frameworks. Use **context7 MCP** to look up current API docs before making changes.
