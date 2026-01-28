<script lang="ts">
  import { invoke } from "@tauri-apps/api/core";

  interface ScreenshotResult {
    width: number;
    height: number;
    data: string;
  }

  let isCapturing = $state(false);
  let error = $state<string | null>(null);
  let imageUrl = $state<string | null>(null);
  let imageDimensions = $state<{ width: number; height: number } | null>(null);

  // Auto-start capture on mount
  $effect(() => {
    captureScreen();
  });

  async function captureScreen() {
    isCapturing = true;
    error = null;

    try {
      const result = await invoke<ScreenshotResult>("take_screenshot_interactive");
      imageUrl = `data:image/png;base64,${result.data}`;
      imageDimensions = { width: result.width, height: result.height };
    } catch (e) {
      // Don't show error if user just cancelled
      const errorStr = String(e);
      if (!errorStr.includes("cancelled")) {
        error = `Capture failed: ${e}`;
      }
    } finally {
      isCapturing = false;
    }
  }
</script>

<main class="container">
  <h1>FlashCap</h1>

  <div class="controls">
    <button onclick={captureScreen} disabled={isCapturing}>
      {isCapturing ? "Capturing..." : "Capture Screen"}
    </button>
  </div>

  {#if error}
    <p class="error">{error}</p>
  {/if}

  <div class="image-container">
    {#if imageUrl}
      {#if imageDimensions}
        <p class="dimensions">{imageDimensions.width} × {imageDimensions.height}</p>
      {/if}
      <img src={imageUrl} alt="Screenshot" />
    {/if}
  </div>
</main>

<style>
  :root {
    font-family: Inter, Avenir, Helvetica, Arial, sans-serif;
    font-size: 16px;
    line-height: 24px;
    font-weight: 400;
    color: #0f0f0f;
    background-color: #f6f6f6;
    font-synthesis: none;
    text-rendering: optimizeLegibility;
    -webkit-font-smoothing: antialiased;
    -moz-osx-font-smoothing: grayscale;
  }

  .container {
    margin: 0;
    padding: 20px;
    display: flex;
    flex-direction: column;
    align-items: center;
  }

  h1 {
    margin-bottom: 20px;
  }

  .controls {
    display: flex;
    gap: 10px;
    margin-bottom: 20px;
  }

  button {
    border-radius: 8px;
    border: 1px solid transparent;
    padding: 0.6em 1.2em;
    font-size: 1em;
    font-weight: 500;
    font-family: inherit;
    color: #0f0f0f;
    background-color: #ffffff;
    transition: border-color 0.25s;
    box-shadow: 0 2px 2px rgba(0, 0, 0, 0.2);
    cursor: pointer;
  }

  button:hover:not(:disabled) {
    border-color: #396cd8;
  }

  button:disabled {
    opacity: 0.6;
    cursor: not-allowed;
  }

  .error {
    color: #dc3545;
    margin-bottom: 10px;
  }

  .dimensions {
    text-align: center;
    color: #666;
    font-size: 14px;
    margin: 10px 0;
  }

  .image-container {
    width: 100%;
    max-width: 100%;
    overflow: auto;
    border: 1px solid #ccc;
    border-radius: 4px;
    background: #fff;
  }

  img {
    display: block;
    max-width: 100%;
    height: auto;
  }

  @media (prefers-color-scheme: dark) {
    :root {
      color: #f6f6f6;
      background-color: #2f2f2f;
    }

    button {
      color: #ffffff;
      background-color: #0f0f0f98;
    }

    .dimensions {
      color: #aaa;
    }

    .image-container {
      border-color: #555;
      background: #1a1a1a;
    }
  }
</style>
