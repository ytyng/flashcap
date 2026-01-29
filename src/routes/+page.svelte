<script lang="ts">
  import { onMount } from "svelte";
  import { invoke } from "@tauri-apps/api/core";
  import { writeText, writeImage } from "@tauri-apps/plugin-clipboard-manager";
  import { revealItemInDir } from "@tauri-apps/plugin-opener";
  import ArrowOverlay from "$lib/ArrowOverlay.svelte";
  import type { Arrow, ArrowSettings } from "$lib/types";

  interface ScreenshotResult {
    width: number;
    height: number;
    data: string;
    file_path: string;
  }

  let isCapturing = $state(false);
  let imageUrl = $state<string | null>(null);
  let imageBase64 = $state<string | null>(null);
  let filePath = $state<string | null>(null);
  let copyPathSuccess = $state(false);
  let copyImageSuccess = $state(false);

  // Arrow tool state
  let arrowToolActive = $state(false);
  let arrows = $state<Arrow[]>([]);
  let arrowSettings = $state<ArrowSettings>({
    color: "#FF0000",
    thickness: 4,
    whiteStroke: true,
    dropShadow: true,
  });

  // Image element reference for composite rendering
  let imgEl = $state<HTMLImageElement | null>(null);

  onMount(() => {
    captureScreen();

    function handleKeydown(e: KeyboardEvent) {
      if (e.key === "Escape") {
        e.preventDefault();
        import("@tauri-apps/api/window").then(({ getCurrentWindow }) =>
          getCurrentWindow().close()
        );
      } else if (e.metaKey && e.shiftKey && e.key === "c") {
        e.preventDefault();
        copyImage();
      } else if (e.metaKey && e.key === "c") {
        e.preventDefault();
        copyPath();
      }
    }
    window.addEventListener("keydown", handleKeydown);
    return () => window.removeEventListener("keydown", handleKeydown);
  });

  async function captureScreen() {
    isCapturing = true;
    try {
      const result = await invoke<ScreenshotResult>("take_screenshot_interactive");
      imageBase64 = result.data;
      imageUrl = `data:image/png;base64,${result.data}`;
      filePath = result.file_path;
      arrows = [];
    } catch (e) {
      const errorStr = String(e);
      if (!errorStr.includes("cancelled")) {
        console.error("Capture failed:", e);
      }
    } finally {
      isCapturing = false;
    }
  }

  async function copyPath() {
    if (!filePath) return;
    await writeText(filePath);
    copyPathSuccess = true;
    setTimeout(() => (copyPathSuccess = false), 3000);
  }

  // Render arrows onto a canvas and return PNG bytes
  function renderComposite(): Promise<Uint8Array> {
    return new Promise((resolve) => {
      const img = new Image();
      img.onload = () => {
        const canvas = document.createElement("canvas");
        canvas.width = img.naturalWidth;
        canvas.height = img.naturalHeight;
        const ctx = canvas.getContext("2d")!;
        ctx.drawImage(img, 0, 0);

        // Scale factor: SVG overlay uses displayed size, canvas uses natural size
        const scaleX = imgEl ? img.naturalWidth / imgEl.clientWidth : 1;
        const scaleY = imgEl ? img.naturalHeight / imgEl.clientHeight : 1;

        for (const arrow of arrows) {
          const sx = arrow.startX * scaleX;
          const sy = arrow.startY * scaleY;
          const ex = arrow.endX * scaleX;
          const ey = arrow.endY * scaleY;
          const t = arrow.thickness * Math.max(scaleX, scaleY);
          const hs = t * 4;

          const dx = sx - ex;
          const dy = sy - ey;
          const len = Math.sqrt(dx * dx + dy * dy);
          if (len === 0) continue;

          const ux = dx / len;
          const uy = dy / len;

          // Line start shortened to arrowhead base
          const lsX = sx - ux * hs;
          const lsY = sy - uy * hs;

          // Arrowhead triangle points
          const perpX = -uy;
          const perpY = ux;
          const halfW = hs * 0.4;
          const bX = sx - ux * hs;
          const bY = sy - uy * hs;

          if (arrow.dropShadow) {
            ctx.shadowColor = "rgba(0,0,0,0.5)";
            ctx.shadowBlur = 4 * Math.max(scaleX, scaleY);
            ctx.shadowOffsetX = 2 * scaleX;
            ctx.shadowOffsetY = 2 * scaleY;
          }

          if (arrow.whiteStroke) {
            ctx.strokeStyle = "white";
            ctx.lineWidth = t + 4 * Math.max(scaleX, scaleY);
            ctx.lineCap = "round";
            ctx.beginPath();
            ctx.moveTo(lsX, lsY);
            ctx.lineTo(ex, ey);
            ctx.stroke();

            ctx.fillStyle = "white";
            ctx.lineJoin = "round";
            ctx.beginPath();
            ctx.moveTo(sx, sy);
            ctx.lineTo(bX + perpX * halfW, bY + perpY * halfW);
            ctx.lineTo(bX - perpX * halfW, bY - perpY * halfW);
            ctx.closePath();
            ctx.fill();
            ctx.lineWidth = 4 * Math.max(scaleX, scaleY);
            ctx.stroke();
          }

          // Main line
          ctx.strokeStyle = arrow.color;
          ctx.lineWidth = t;
          ctx.lineCap = "round";
          ctx.beginPath();
          ctx.moveTo(lsX, lsY);
          ctx.lineTo(ex, ey);
          ctx.stroke();

          // Arrowhead
          ctx.fillStyle = arrow.color;
          ctx.beginPath();
          ctx.moveTo(sx, sy);
          ctx.lineTo(bX + perpX * halfW, bY + perpY * halfW);
          ctx.lineTo(bX - perpX * halfW, bY - perpY * halfW);
          ctx.closePath();
          ctx.fill();

          // Reset shadow
          ctx.shadowColor = "transparent";
          ctx.shadowBlur = 0;
          ctx.shadowOffsetX = 0;
          ctx.shadowOffsetY = 0;
        }

        canvas.toBlob((blob) => {
          blob!.arrayBuffer().then((buf) => resolve(new Uint8Array(buf)));
        }, "image/png");
      };
      img.src = `data:image/png;base64,${imageBase64}`;
    });
  }

  async function copyImage() {
    if (!imageBase64) return;

    let bytes: Uint8Array;
    if (arrows.length > 0) {
      bytes = await renderComposite();
    } else {
      const binaryStr = atob(imageBase64);
      bytes = new Uint8Array(binaryStr.length);
      for (let i = 0; i < binaryStr.length; i++) {
        bytes[i] = binaryStr.charCodeAt(i);
      }
    }
    await writeImage(bytes);
    copyImageSuccess = true;
    setTimeout(() => (copyImageSuccess = false), 3000);
  }

  async function openFolder() {
    await revealItemInDir("/tmp/flashcap/");
  }

  function toggleArrowTool() {
    arrowToolActive = !arrowToolActive;
  }
</script>

<div class="app">
  <div class="toolbar">
    <button
      class="tool-btn"
      class:active={arrowToolActive}
      onclick={toggleArrowTool}
      title="Arrow tool"
    >
      <i class="bi bi-arrow-up-right"></i>
    </button>

    {#if arrowToolActive}
      <div class="separator"></div>

      <input
        type="color"
        class="color-picker"
        bind:value={arrowSettings.color}
        title="Arrow color"
      />

      <select
        class="thickness-select"
        bind:value={arrowSettings.thickness}
        title="Arrow thickness"
      >
        <option value={2}>Thin</option>
        <option value={4}>Medium</option>
        <option value={6}>Thick</option>
        <option value={8}>Extra thick</option>
      </select>

      <button
        class="tool-btn"
        class:active={arrowSettings.whiteStroke}
        onclick={() => (arrowSettings.whiteStroke = !arrowSettings.whiteStroke)}
        title="White border"
      >
        <i class="bi bi-border-width"></i>
      </button>

      <button
        class="tool-btn"
        class:active={arrowSettings.dropShadow}
        onclick={() => (arrowSettings.dropShadow = !arrowSettings.dropShadow)}
        title="Drop shadow"
      >
        <i class="bi bi-shadows"></i>
      </button>
    {/if}

    <div class="separator"></div>

    {#if filePath}
      <div class="file-path" title={filePath}>
        {filePath}
      </div>
      <button
        class="tool-btn"
        onclick={copyPath}
        title="Copy file path (⌘C)"
      >
        {#if copyPathSuccess}
          <i class="bi bi-check-lg"></i>
        {:else}
          <i class="bi bi-clipboard"></i>
        {/if}
      </button>
      <button
        class="tool-btn"
        onclick={copyImage}
        title="Copy image (⌘⇧C)"
      >
        {#if copyImageSuccess}
          <i class="bi bi-check-lg"></i>
        {:else}
          <i class="bi bi-image"></i>
        {/if}
      </button>
      <button
        class="tool-btn"
        onclick={openFolder}
        title="Open save folder"
      >
        <i class="bi bi-folder2-open"></i>
      </button>
    {/if}

    <div class="spacer"></div>

    <button
      class="tool-btn capture-btn"
      onclick={captureScreen}
      disabled={isCapturing}
      title="Capture new area"
    >
      <i class="bi bi-camera"></i>
    </button>
  </div>

  <div class="image-area">
    {#if imageUrl}
      <div class="image-container">
        <img bind:this={imgEl} src={imageUrl} alt="Screenshot" />
        <ArrowOverlay
          {arrows}
          settings={arrowSettings}
          toolActive={arrowToolActive}
          onArrowsChange={(newArrows) => (arrows = newArrows)}
        />
      </div>
    {:else if isCapturing}
      <div class="placeholder">Capturing...</div>
    {:else}
      <div class="placeholder">No image</div>
    {/if}
  </div>
</div>

<style>
  :global(body) {
    margin: 0;
    padding: 0;
    overflow: hidden;
  }

  .app {
    display: flex;
    flex-direction: column;
    height: 100vh;
    background: #1a1a1a;
    color: #fff;
    font-family: -apple-system, BlinkMacSystemFont, "Segoe UI", Roboto,
      sans-serif;
  }

  .toolbar {
    display: flex;
    align-items: center;
    gap: 8px;
    padding: 8px 12px;
    background: #2d2d2d;
    border-bottom: 1px solid #3d3d3d;
    min-height: 40px;
  }

  .tool-btn {
    display: flex;
    align-items: center;
    justify-content: center;
    width: 32px;
    height: 32px;
    border: none;
    border-radius: 6px;
    background: transparent;
    color: #ccc;
    cursor: pointer;
    transition: background 0.15s, color 0.15s;
    font-size: 16px;
  }

  .tool-btn:hover:not(:disabled) {
    background: #3d3d3d;
    color: #fff;
  }

  .tool-btn:disabled {
    opacity: 0.5;
    cursor: not-allowed;
  }

  .tool-btn.active {
    background: #0066cc;
    color: #fff;
  }

  .capture-btn {
    background: #0066cc;
    color: #fff;
  }

  .capture-btn:hover:not(:disabled) {
    background: #0077ee;
  }

  .separator {
    width: 1px;
    height: 24px;
    background: #3d3d3d;
  }

  .file-path {
    font-size: 13px;
    color: #aaa;
    max-width: 300px;
    overflow: hidden;
    text-overflow: ellipsis;
    white-space: nowrap;
  }

  .spacer {
    flex: 1;
  }

  .image-area {
    flex: 1;
    display: flex;
    align-items: center;
    justify-content: center;
    overflow: auto;
    padding: 16px;
  }

  .image-container {
    position: relative;
    display: inline-block;
    max-width: 100%;
    max-height: 100%;
  }

  .image-container img {
    max-width: 100%;
    max-height: calc(100vh - 80px);
    object-fit: contain;
    border-radius: 4px;
    box-shadow: 0 4px 20px rgba(0, 0, 0, 0.5);
    display: block;
  }

  .placeholder {
    color: #666;
    font-size: 14px;
  }

  .color-picker {
    width: 28px;
    height: 28px;
    border: none;
    border-radius: 4px;
    padding: 0;
    cursor: pointer;
    background: transparent;
  }

  .color-picker::-webkit-color-swatch-wrapper {
    padding: 2px;
  }

  .color-picker::-webkit-color-swatch {
    border: 1px solid #555;
    border-radius: 3px;
  }

  .thickness-select {
    background: #3d3d3d;
    color: #ccc;
    border: 1px solid #555;
    border-radius: 4px;
    padding: 4px 6px;
    font-size: 12px;
    cursor: pointer;
  }
</style>
