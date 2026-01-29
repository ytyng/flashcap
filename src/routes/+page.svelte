<script lang="ts">
  import { onMount } from "svelte";
  import { invoke } from "@tauri-apps/api/core";
  import { writeText, writeImage } from "@tauri-apps/plugin-clipboard-manager";

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

  async function copyImage() {
    if (!imageBase64) return;
    const binaryStr = atob(imageBase64);
    const bytes = new Uint8Array(binaryStr.length);
    for (let i = 0; i < binaryStr.length; i++) {
      bytes[i] = binaryStr.charCodeAt(i);
    }
    await writeImage(bytes);
    copyImageSuccess = true;
    setTimeout(() => (copyImageSuccess = false), 3000);
  }
</script>

<div class="app">
  <div class="toolbar">
    <button class="tool-btn" title="Arrow tool">
      <i class="bi bi-cursor"></i>
    </button>

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
      <img src={imageUrl} alt="Screenshot" />
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

  .image-area img {
    max-width: 100%;
    max-height: 100%;
    object-fit: contain;
    border-radius: 4px;
    box-shadow: 0 4px 20px rgba(0, 0, 0, 0.5);
  }

  .placeholder {
    color: #666;
    font-size: 14px;
  }
</style>
