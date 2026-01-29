<script lang="ts">
  import type { MaskRect, MaskSettings } from "./types";

  interface Props {
    masks: MaskRect[];
    settings: MaskSettings;
    toolActive: boolean;
    interactive: boolean;
    imageElement: HTMLImageElement | null;
    onMasksChange: (masks: MaskRect[]) => void;
  }

  let { masks, settings, toolActive, interactive, imageElement, onMasksChange }: Props = $props();

  let selectedId = $state<string | null>(null);
  let dragging = $state(false);
  let drawingRect = $state<MaskRect | null>(null);
  let dragStart = $state<{ x: number; y: number } | null>(null);

  function getSvgCoords(e: MouseEvent): { x: number; y: number } | null {
    const svg = (e.currentTarget as SVGSVGElement) ?? (e.target as Element).closest("svg");
    if (!svg) return null;
    const rect = svg.getBoundingClientRect();
    return { x: e.clientX - rect.left, y: e.clientY - rect.top };
  }

  function handleMouseDown(e: MouseEvent) {
    const pt = getSvgCoords(e);
    if (!pt) return;

    if (!toolActive) {
      const hit = [...masks].reverse().find((m) =>
        pt.x >= m.x && pt.x <= m.x + m.width && pt.y >= m.y && pt.y <= m.y + m.height
      );
      selectedId = hit?.id ?? null;
      return;
    }

    dragStart = pt;
    drawingRect = {
      id: crypto.randomUUID(),
      x: pt.x,
      y: pt.y,
      width: 0,
      height: 0,
      mode: settings.mode,
      color: settings.color,
    };
    dragging = true;
    selectedId = null;
  }

  function handleMouseMove(e: MouseEvent) {
    if (!dragging || !dragStart) return;
    const pt = getSvgCoords(e);
    if (!pt || !drawingRect) return;

    const x = Math.min(dragStart.x, pt.x);
    const y = Math.min(dragStart.y, pt.y);
    const width = Math.abs(pt.x - dragStart.x);
    const height = Math.abs(pt.y - dragStart.y);
    drawingRect = { ...drawingRect, x, y, width, height };
  }

  function handleMouseUp() {
    if (dragging && drawingRect && drawingRect.width > 3 && drawingRect.height > 3) {
      onMasksChange([...masks, drawingRect]);
      selectedId = drawingRect.id;
    }
    drawingRect = null;
    dragging = false;
    dragStart = null;
  }

  function handleKeyDown(e: KeyboardEvent) {
    if ((e.key === "Delete" || e.key === "Backspace") && selectedId) {
      e.preventDefault();
      e.stopPropagation();
      onMasksChange(masks.filter((m) => m.id !== selectedId));
      selectedId = null;
    }
  }

  let allMasks = $derived(drawingRect ? [...masks, drawingRect] : masks);

  /**
   * 画像の指定領域をピクセル化した data URL を返す。
   * 表示座標 → 元画像の自然解像度に変換し、縮小→拡大でモザイク化。
   */
  function pixelateRegion(mask: MaskRect): string | null {
    if (!imageElement) return null;
    const scaleX = imageElement.naturalWidth / imageElement.clientWidth;
    const scaleY = imageElement.naturalHeight / imageElement.clientHeight;
    const sx = Math.round(mask.x * scaleX);
    const sy = Math.round(mask.y * scaleY);
    const sw = Math.round(mask.width * scaleX);
    const sh = Math.round(mask.height * scaleY);
    if (sw <= 0 || sh <= 0) return null;

    // 元画像から対象領域を切り出し
    const src = document.createElement("canvas");
    src.width = sw;
    src.height = sh;
    const sCtx = src.getContext("2d")!;
    sCtx.drawImage(imageElement, sx, sy, sw, sh, 0, 0, sw, sh);

    // 縮小（ブロックサイズ分の1に）
    const blockSize = 10;
    const tw = Math.max(1, Math.ceil(sw / blockSize));
    const th = Math.max(1, Math.ceil(sh / blockSize));
    const tiny = document.createElement("canvas");
    tiny.width = tw;
    tiny.height = th;
    const tCtx = tiny.getContext("2d")!;
    tCtx.drawImage(src, 0, 0, tw, th);

    // 拡大（nearest-neighbor）
    const out = document.createElement("canvas");
    out.width = sw;
    out.height = sh;
    const oCtx = out.getContext("2d")!;
    oCtx.imageSmoothingEnabled = false;
    oCtx.drawImage(tiny, 0, 0, sw, sh);

    return out.toDataURL();
  }

  export function deselect() {
    selectedId = null;
  }
</script>

<svelte:window onkeydown={handleKeyDown} />

<!-- svelte-ignore a11y_no_static_element_interactions -->
<svg
  class="mask-overlay"
  onmousedown={handleMouseDown}
  onmousemove={handleMouseMove}
  onmouseup={handleMouseUp}
  style:cursor={toolActive ? "crosshair" : "default"}
  style:pointer-events={interactive ? "auto" : "none"}
>
  {#each allMasks as mask (mask.id)}
    {@const isSelected = mask.id === selectedId}

    {#if mask.mode === "blur"}
      <foreignObject x={mask.x} y={mask.y} width={mask.width} height={mask.height}>
        <div
          xmlns="http://www.w3.org/1999/xhtml"
          style="width:100%;height:100%;backdrop-filter:blur(10px);-webkit-backdrop-filter:blur(10px);"
        ></div>
      </foreignObject>
    {:else if mask.mode === "fill"}
      <rect
        x={mask.x} y={mask.y} width={mask.width} height={mask.height}
        fill={mask.color}
      />
    {:else if mask.mode === "mosaic"}
      {@const url = pixelateRegion(mask)}
      {#if url}
        <image
          href={url}
          x={mask.x} y={mask.y}
          width={mask.width} height={mask.height}
          preserveAspectRatio="none"
        />
      {:else}
        <rect
          x={mask.x} y={mask.y} width={mask.width} height={mask.height}
          fill="rgba(128,128,128,0.5)"
        />
      {/if}
    {/if}

    {#if isSelected}
      <rect
        x={mask.x} y={mask.y} width={mask.width} height={mask.height}
        fill="none" stroke="#0066cc" stroke-width="2" stroke-dasharray="4 2"
      />
    {/if}
  {/each}
</svg>

<style>
  .mask-overlay {
    position: absolute;
    top: 0;
    left: 0;
    width: 100%;
    height: 100%;
  }
</style>
