<script lang="ts">
  import type {
    ArrowSettings,
    MaskSettings,
    ShapeSettings,
    TextSettings,
  } from "$lib/types";

  let {
    arrowToolActive,
    maskToolActive,
    shapeToolActive,
    textToolActive,
    arrowSettings,
    maskSettings,
    shapeSettings,
    textSettings,
    filePath,
    copyPathSuccess,
    copyImageSuccess,
    isCapturing,
    timerDelay,
    onToggleArrowTool,
    onToggleMaskTool,
    onToggleShapeTool,
    onToggleTextTool,
    onCopyPath,
    onCopyImage,
    onOpenFolder,
    onCapture,
    onDragFile,
    onUpdateTextSetting,
  }: {
    arrowToolActive: boolean;
    maskToolActive: boolean;
    shapeToolActive: boolean;
    textToolActive: boolean;
    arrowSettings: ArrowSettings;
    maskSettings: MaskSettings;
    shapeSettings: ShapeSettings;
    textSettings: TextSettings;
    filePath: string | null;
    copyPathSuccess: boolean;
    copyImageSuccess: boolean;
    isCapturing: boolean;
    timerDelay: number;
    onToggleArrowTool: () => void;
    onToggleMaskTool: () => void;
    onToggleShapeTool: (type: "rect" | "ellipse") => void;
    onToggleTextTool: () => void;
    onCopyPath: () => void;
    onCopyImage: () => void;
    onOpenFolder: () => void;
    onCapture: (command?: string) => void;
    onDragFile: () => void;
    onUpdateTextSetting: <K extends keyof TextSettings>(key: K, value: TextSettings[K]) => void;
  } = $props();
</script>

<div class="flex items-center gap-2 px-3 py-2 bg-neutral-800 border-b border-neutral-700 min-h-[40px]">
  <button
    class="tool-btn"
    class:active={arrowToolActive}
    onclick={onToggleArrowTool}
    aria-label="Arrow tool"
    data-tooltip="Arrow tool"
  >
    <i class="bi bi-arrow-up-right"></i>
  </button>

  {#if arrowToolActive}
    <div class="tool-settings">
      <input
        type="color"
        class="color-picker"
        bind:value={arrowSettings.color}
        data-tooltip="Arrow color"
      />

      <select
        class="bg-neutral-700 text-neutral-300 border border-neutral-600 rounded px-1.5 py-1 text-xs cursor-pointer"
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
        aria-label="White border"
        data-tooltip="White border"
      >
        <i class="bi bi-back"></i>
      </button>

      <button
        class="tool-btn"
        class:active={arrowSettings.dropShadow}
        onclick={() => (arrowSettings.dropShadow = !arrowSettings.dropShadow)}
        aria-label="Drop shadow"
        data-tooltip="Drop shadow"
      >
        <i class="bi bi-shadows"></i>
      </button>
    </div>
  {/if}

  {#snippet shapeSettingsPanel()}
    <div class="tool-settings">
      <input
        type="color"
        class="color-picker"
        bind:value={shapeSettings.color}
        data-tooltip="Shape color"
      />

      <i class="bi bi-border-width text-neutral-400 text-xs"></i>
      <select
        class="bg-neutral-700 text-neutral-300 border border-neutral-600 rounded px-1.5 py-1 text-xs cursor-pointer"
        bind:value={shapeSettings.thickness}
        data-tooltip="Shape thickness"
      >
        <option value={2}>Thin</option>
        <option value={4}>Medium</option>
        <option value={6}>Thick</option>
        <option value={8}>Extra thick</option>
      </select>

      <button
        class="tool-btn"
        class:active={shapeSettings.whiteStroke}
        onclick={() => (shapeSettings.whiteStroke = !shapeSettings.whiteStroke)}
        aria-label="White border"
        data-tooltip="White border"
      >
        <i class="bi bi-back"></i>
      </button>

      <button
        class="tool-btn"
        class:active={shapeSettings.dropShadow}
        onclick={() => (shapeSettings.dropShadow = !shapeSettings.dropShadow)}
        aria-label="Drop shadow"
        data-tooltip="Drop shadow"
      >
        <i class="bi bi-shadows"></i>
      </button>
    </div>
  {/snippet}

  <button
    class="tool-btn"
    class:active={shapeToolActive && shapeSettings.type === "rect"}
    onclick={() => onToggleShapeTool("rect")}
    aria-label="Rectangle tool"
    data-tooltip="Rectangle tool"
  >
    <i class="bi bi-square"></i>
  </button>

  {#if shapeToolActive && shapeSettings.type === "rect"}
    {@render shapeSettingsPanel()}
  {/if}

  <button
    class="tool-btn"
    class:active={shapeToolActive && shapeSettings.type === "ellipse"}
    onclick={() => onToggleShapeTool("ellipse")}
    aria-label="Ellipse tool"
    data-tooltip="Ellipse tool"
  >
    <i class="bi bi-circle"></i>
  </button>

  {#if shapeToolActive && shapeSettings.type === "ellipse"}
    {@render shapeSettingsPanel()}
  {/if}

  <button
    class="tool-btn"
    class:active={textToolActive}
    onclick={onToggleTextTool}
    aria-label="Text tool"
    data-tooltip="Text tool"
  >
    <i class="bi bi-fonts" style="font-size: 125%"></i>
  </button>

  {#if textToolActive}
    <div class="tool-settings">
      <input
        type="color"
        class="color-picker"
        value={textSettings.color}
        oninput={(e) => onUpdateTextSetting("color", (e.target as HTMLInputElement).value)}
        data-tooltip="Text color"
      />

      <select
        class="bg-neutral-700 text-neutral-300 border border-neutral-600 rounded px-1.5 py-1 text-xs cursor-pointer"
        value={textSettings.fontSize}
        onchange={(e) => onUpdateTextSetting("fontSize", Number((e.target as HTMLSelectElement).value))}
        data-tooltip="Font size"
      >
        <option value={16}>16px</option>
        <option value={20}>20px</option>
        <option value={24}>24px</option>
        <option value={32}>32px</option>
        <option value={48}>48px</option>
      </select>

      <button
        class="tool-btn"
        class:active={textSettings.bold}
        onclick={() => onUpdateTextSetting("bold", !textSettings.bold)}
        aria-label="Bold"
        data-tooltip="Bold"
      >
        <i class="bi bi-type-bold"></i>
      </button>

      <button
        class="tool-btn"
        class:active={textSettings.italic}
        onclick={() => onUpdateTextSetting("italic", !textSettings.italic)}
        aria-label="Italic"
        data-tooltip="Italic"
      >
        <i class="bi bi-type-italic"></i>
      </button>

      <button
        class="tool-btn"
        class:active={textSettings.whiteStroke}
        onclick={() => onUpdateTextSetting("whiteStroke", !textSettings.whiteStroke)}
        aria-label="White border"
        data-tooltip="White border"
      >
        <i class="bi bi-back"></i>
      </button>

      <button
        class="tool-btn"
        class:active={textSettings.dropShadow}
        onclick={() => onUpdateTextSetting("dropShadow", !textSettings.dropShadow)}
        aria-label="Drop shadow"
        data-tooltip="Drop shadow"
      >
        <i class="bi bi-shadows"></i>
      </button>
    </div>
  {/if}

  <button
    class="tool-btn"
    class:active={maskToolActive}
    onclick={onToggleMaskTool}
    aria-label="Mask tool"
    data-tooltip="Mask tool"
  >
    <i class="bi bi-grid-3x3"></i>
  </button>

  {#if maskToolActive}
    <div class="tool-settings">
      <button
        class="tool-btn text-xs"
        class:active={maskSettings.mode === "mosaic"}
        onclick={() => (maskSettings.mode = "mosaic")}
        data-tooltip="Mosaic"
      >▦</button>
      <button
        class="tool-btn text-xs"
        class:active={maskSettings.mode === "blur"}
        onclick={() => (maskSettings.mode = "blur")}
        aria-label="Blur"
        data-tooltip="Blur"
      >
        <i class="bi bi-droplet-half"></i>
      </button>
      <button
        class="tool-btn text-xs"
        class:active={maskSettings.mode === "fill"}
        onclick={() => (maskSettings.mode = "fill")}
        aria-label="Fill"
        data-tooltip="Fill"
      >
        <i class="bi bi-paint-bucket"></i>
      </button>

      {#if maskSettings.mode === "fill"}
        <input
          type="color"
          class="color-picker"
          bind:value={maskSettings.color}
          data-tooltip="Fill color"
        />
      {/if}
    </div>
  {/if}

  <div class="toolbar-divider"></div>

  {#if filePath}
    <button
      class="tool-btn cursor-grab active:cursor-grabbing"
      aria-label="Drag to share file"
      data-tooltip="Drag to share file"
      onmousedown={onDragFile}
    >
      <i class="bi bi-grip-vertical"></i>
    </button>
    <input
      type="text"
      class="flex-1 min-w-0 text-[13px] text-neutral-400 bg-transparent border-none outline-none"
      value={filePath}
      readonly
      title={filePath}
    />
    <button
      class="tool-btn"
      onclick={onCopyPath}
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
      onclick={onCopyImage}
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
      onclick={onOpenFolder}
      aria-label="Open save folder"
      data-tooltip="Open save folder"
    >
      <i class="bi bi-folder2-open"></i>
    </button>
  {/if}

  <div class="toolbar-divider"></div>

  <button
    class="tool-btn"
    onclick={() => onCapture("take_screenshot_timer")}
    disabled={isCapturing}
    aria-label="Timer capture"
    data-tooltip="Timer capture ({timerDelay}s)"
  >
    <i class="bi bi-stopwatch"></i>
  </button>
  <button
    class="tool-btn !bg-blue-600 !text-white hover:!bg-blue-500"
    onclick={() => onCapture()}
    disabled={isCapturing}
    aria-label="Capture new area"
    data-tooltip="Capture new area"
  >
    <i class="bi bi-camera"></i>
  </button>
</div>

<style>
  .tool-settings {
    display: flex;
    align-items: center;
    gap: 0.5rem;
    background-color: var(--color-neutral-700);
    margin: -0.5rem 0;
    padding: 0.5rem 0.5rem;
    border-left: 1px solid var(--color-neutral-600);
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
    color: var(--color-neutral-300);
    cursor: pointer;
    transition: background 0.15s, color 0.15s;
    font-size: 16px;
  }

  .tool-btn:hover:not(:disabled) {
    background: var(--color-neutral-700);
    color: #fff;
  }

  .tool-btn:disabled {
    opacity: 0.5;
    cursor: not-allowed;
  }

  .tool-btn.active {
    background: var(--color-blue-600);
    color: #fff;
  }

  .tool-btn.active:hover:not(:disabled) {
    background: var(--color-blue-500);
  }

  .toolbar-divider {
    width: 1px;
    height: 1.5rem;
    background-color: var(--color-neutral-700);
  }

  /* Instant tooltip via data-tooltip attribute + ::after */
  [data-tooltip] {
    position: relative;
  }

  [data-tooltip]::after {
    content: attr(data-tooltip);
    position: absolute;
    top: calc(100% + 6px);
    left: 50%;
    transform: translateX(-50%);
    padding: 4px 8px;
    background: #000;
    color: var(--color-neutral-200);
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
    border: 1px solid var(--color-neutral-600);
    border-radius: 3px;
  }
</style>
