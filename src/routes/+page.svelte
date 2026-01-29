<script lang="ts">
  import { onMount } from "svelte";
  import { invoke } from "@tauri-apps/api/core";
  import { listen } from "@tauri-apps/api/event";
  import { writeText, writeImage } from "@tauri-apps/plugin-clipboard-manager";
  import { revealItemInDir } from "@tauri-apps/plugin-opener";
  import { startDrag } from "@crabnebula/tauri-plugin-drag";
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
  const ARROW_SETTINGS_KEY = "flashcap-arrow-settings";
  let arrowSettings = $state<ArrowSettings>({
    color: "#FF0000",
    thickness: 4,
    whiteStroke: true,
    dropShadow: true,
  });

  // Image element reference for composite rendering
  let imgEl = $state<HTMLImageElement | null>(null);

  onMount(() => {
    // Restore arrow settings from localStorage
    const saved = localStorage.getItem(ARROW_SETTINGS_KEY);
    if (saved) {
      try {
        const parsed = JSON.parse(saved);
        arrowSettings.color = parsed.color ?? arrowSettings.color;
        arrowSettings.thickness = parsed.thickness ?? arrowSettings.thickness;
        arrowSettings.whiteStroke = parsed.whiteStroke ?? arrowSettings.whiteStroke;
        arrowSettings.dropShadow = parsed.dropShadow ?? arrowSettings.dropShadow;
      } catch { /* ignore invalid JSON */ }
    }

    captureScreen();

    // アプリ再アクティブ時にキャプチャーモードを開始
    const unlisten = listen("reactivate", () => {
      captureScreen();
    });

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
    return () => {
      window.removeEventListener("keydown", handleKeydown);
      unlisten.then((fn) => fn());
    };
  });

  // Persist arrow settings to localStorage on change
  $effect(() => {
    const { color, thickness, whiteStroke, dropShadow } = arrowSettings;
    localStorage.setItem(
      ARROW_SETTINGS_KEY,
      JSON.stringify({ color, thickness, whiteStroke, dropShadow })
    );
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
    await saveCompositeToFile();
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

  function uint8ToBase64(bytes: Uint8Array): string {
    const chunks: string[] = [];
    for (let i = 0; i < bytes.length; i += 8192) {
      chunks.push(String.fromCharCode(...bytes.subarray(i, i + 8192)));
    }
    return btoa(chunks.join(""));
  }

  function base64ToUint8(base64: string): Uint8Array {
    const binary = atob(base64);
    const bytes = new Uint8Array(binary.length);
    for (let i = 0; i < binary.length; i++) {
      bytes[i] = binary.charCodeAt(i);
    }
    return bytes;
  }

  // 合成画像をファイルに書き出す（矢印がある場合のみ）
  // メモリ上の元画像 (imageBase64) はそのまま保持する
  // compositeBytes が渡された場合は再レンダリングをスキップする
  async function saveCompositeToFile(compositeBytes?: Uint8Array) {
    if (!filePath || !imageBase64 || arrows.length === 0) return;
    const bytes = compositeBytes ?? await renderComposite();
    await invoke("write_image_to_file", {
      path: filePath,
      dataBase64: uint8ToBase64(bytes),
    });
  }

  async function copyImage() {
    if (!imageBase64) return;

    const bytes = arrows.length > 0
      ? await renderComposite()
      : base64ToUint8(imageBase64);

    await saveCompositeToFile(bytes);
    await writeImage(bytes);
    copyImageSuccess = true;
    setTimeout(() => (copyImageSuccess = false), 3000);
  }

  async function openFolder() {
    if (filePath) {
      // ファイルを指定することでフォルダ内のファイル一覧が表示される
      await revealItemInDir(filePath);
    }
  }

  function toggleArrowTool() {
    arrowToolActive = !arrowToolActive;
  }
</script>

<div class="flex flex-col h-screen bg-[#1a1a1a] text-white font-[-apple-system,BlinkMacSystemFont,'Segoe_UI',Roboto,sans-serif]">
  <div class="flex items-center gap-2 px-3 py-2 bg-[#2d2d2d] border-b border-[#3d3d3d] min-h-[40px]">
    <button
      class="tool-btn"
      class:active={arrowToolActive}
      onclick={toggleArrowTool}
      data-tooltip="Arrow tool"
    >
      <i class="bi bi-arrow-up-right"></i>
    </button>

    {#if arrowToolActive}
      <div class="w-px h-6 bg-[#3d3d3d]"></div>

      <input
        type="color"
        class="color-picker"
        bind:value={arrowSettings.color}
        data-tooltip="Arrow color"
      />

      <select
        class="bg-[#3d3d3d] text-[#ccc] border border-[#555] rounded px-1.5 py-1 text-xs cursor-pointer"
        bind:value={arrowSettings.thickness}
        data-tooltip="Arrow thickness"
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
        data-tooltip="White border"
      >
        <i class="bi bi-border-width"></i>
      </button>

      <button
        class="tool-btn"
        class:active={arrowSettings.dropShadow}
        onclick={() => (arrowSettings.dropShadow = !arrowSettings.dropShadow)}
        data-tooltip="Drop shadow"
      >
        <i class="bi bi-shadows"></i>
      </button>
    {/if}

    <div class="w-px h-6 bg-[#3d3d3d]"></div>

    {#if filePath}
      <!-- Drag this icon to external apps (e.g. Slack) to share the file -->
      <button
        class="tool-btn cursor-grab active:cursor-grabbing"
        data-tooltip="Drag to share file"
        onmousedown={async () => {
          if (filePath) {
            await saveCompositeToFile();
            startDrag({ item: [filePath], icon: filePath });
          }
        }}
      >
        <i class="bi bi-grip-vertical"></i>
      </button>
      <input
        type="text"
        class="w-[200px] text-[13px] text-[#aaa] bg-transparent border-none outline-none"
        value={filePath}
        readonly
        title={filePath}
      />
      <button
        class="tool-btn"
        onclick={copyPath}
        data-tooltip="Copy file path (⌘C)"
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
        data-tooltip="Copy image (⌘⇧C)"
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
        data-tooltip="Open save folder"
      >
        <i class="bi bi-folder2-open"></i>
      </button>
    {/if}

    <div class="flex-1"></div>

    <button
      class="tool-btn bg-[#0066cc] text-white hover:bg-[#0077ee]"
      onclick={captureScreen}
      disabled={isCapturing}
      data-tooltip="Capture new area"
    >
      <i class="bi bi-camera"></i>
    </button>
  </div>

  <div class="flex-1 flex items-center justify-center overflow-auto p-4">
    {#if imageUrl}
      <div class="relative inline-block max-w-full max-h-full">
        <img
          bind:this={imgEl}
          src={imageUrl}
          alt="Screenshot"
          class="max-w-full max-h-[calc(100vh-80px)] object-contain rounded shadow-[0_4px_20px_rgba(0,0,0,0.5)] block"
        />
        <ArrowOverlay
          {arrows}
          settings={arrowSettings}
          toolActive={arrowToolActive}
          onArrowsChange={(newArrows) => (arrows = newArrows)}
        />
      </div>
    {:else if isCapturing}
      <div class="text-[#666] text-sm">Capturing...</div>
    {:else}
      <div class="text-[#666] text-sm">No image</div>
    {/if}
  </div>
</div>

<style>
  :global(body) {
    margin: 0;
    padding: 0;
    overflow: hidden;
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

  /* Instant tooltip via data-tooltip attribute + ::after */
  [data-tooltip] {
    position: relative;
  }

  [data-tooltip]::after {
    content: attr(data-tooltip);
    position: absolute;
    bottom: calc(100% + 6px);
    left: 50%;
    transform: translateX(-50%);
    padding: 4px 8px;
    background: #000;
    color: #eee;
    font-size: 11px;
    line-height: 1.3;
    border-radius: 4px;
    white-space: nowrap;
    pointer-events: none;
    opacity: 0;
    transition: opacity 0.1s;
    z-index: 100;
  }

  [data-tooltip]:hover::after {
    opacity: 1;
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
</style>
