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
  let dragging = $state<"draw" | "move" | "resize" | null>(null);
  let drawingRect = $state<MaskRect | null>(null);
  let dragStart = $state<{ x: number; y: number } | null>(null);

  // move/resize 用の初期状態
  let dragOrigRect = $state<{ x: number; y: number; width: number; height: number } | null>(null);
  let resizeHandle = $state<string | null>(null); // "nw","n","ne","e","se","s","sw","w"

  function getSvgCoords(e: MouseEvent): { x: number; y: number } | null {
    const svg = (e.currentTarget as SVGSVGElement) ?? (e.target as Element).closest("svg");
    if (!svg) return null;
    const rect = svg.getBoundingClientRect();
    return { x: e.clientX - rect.left, y: e.clientY - rect.top };
  }

  function handleMouseDown(e: MouseEvent) {
    const pt = getSvgCoords(e);
    if (!pt) return;

    // 選択中マスクのハンドル or 本体をドラッグ
    if (selectedId) {
      const sel = masks.find((m) => m.id === selectedId);
      if (sel) {
        const handle = hitTestHandle(sel, pt.x, pt.y);
        if (handle) {
          resizeHandle = handle;
          dragging = "resize";
          dragStart = pt;
          dragOrigRect = { x: sel.x, y: sel.y, width: sel.width, height: sel.height };
          return;
        }
        if (pt.x >= sel.x && pt.x <= sel.x + sel.width && pt.y >= sel.y && pt.y <= sel.y + sel.height) {
          dragging = "move";
          dragStart = pt;
          dragOrigRect = { x: sel.x, y: sel.y, width: sel.width, height: sel.height };
          return;
        }
      }
    }

    // 既存マスクをクリックして選択
    const hit = [...masks].reverse().find((m) =>
      pt.x >= m.x && pt.x <= m.x + m.width && pt.y >= m.y && pt.y <= m.y + m.height
    );
    if (hit) {
      selectedId = hit.id;
      return;
    }

    if (!toolActive) {
      selectedId = null;
      return;
    }

    // 新しいマスクを描画開始
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
    dragging = "draw";
    selectedId = null;
  }

  function handleMouseMove(e: MouseEvent) {
    if (!dragging || !dragStart) return;
    const pt = getSvgCoords(e);
    if (!pt) return;

    const dx = pt.x - dragStart.x;
    const dy = pt.y - dragStart.y;

    if (dragging === "draw" && drawingRect) {
      const x = Math.min(dragStart.x, pt.x);
      const y = Math.min(dragStart.y, pt.y);
      const width = Math.abs(pt.x - dragStart.x);
      const height = Math.abs(pt.y - dragStart.y);
      drawingRect = { ...drawingRect, x, y, width, height };
    } else if (dragging === "move" && selectedId && dragOrigRect) {
      onMasksChange(
        masks.map((m) =>
          m.id === selectedId
            ? { ...m, x: dragOrigRect!.x + dx, y: dragOrigRect!.y + dy }
            : m
        )
      );
    } else if (dragging === "resize" && selectedId && dragOrigRect && resizeHandle) {
      const updated = computeResize(dragOrigRect, resizeHandle, dx, dy);
      onMasksChange(
        masks.map((m) => (m.id === selectedId ? { ...m, ...updated } : m))
      );
    }
  }

  function handleMouseUp() {
    if (dragging === "draw" && drawingRect && drawingRect.width > 3 && drawingRect.height > 3) {
      onMasksChange([...masks, drawingRect]);
      selectedId = drawingRect.id;
    }
    drawingRect = null;
    dragging = null;
    dragStart = null;
    dragOrigRect = null;
    resizeHandle = null;
  }

  function handleKeyDown(e: KeyboardEvent) {
    if ((e.key === "Delete" || e.key === "Backspace") && selectedId) {
      e.preventDefault();
      e.stopPropagation();
      onMasksChange(masks.filter((m) => m.id !== selectedId));
      selectedId = null;
    }
  }

  // リサイズハンドルのヒットテスト (8方向)
  const HANDLE_SIZE = 6;
  function hitTestHandle(mask: MaskRect, px: number, py: number): string | null {
    const { x, y, width: w, height: h } = mask;
    const handles: [string, number, number][] = [
      ["nw", x, y], ["n", x + w / 2, y], ["ne", x + w, y],
      ["w", x, y + h / 2], ["e", x + w, y + h / 2],
      ["sw", x, y + h], ["s", x + w / 2, y + h], ["se", x + w, y + h],
    ];
    for (const [name, hx, hy] of handles) {
      if (Math.abs(px - hx) <= HANDLE_SIZE && Math.abs(py - hy) <= HANDLE_SIZE) return name;
    }
    return null;
  }

  // ハンドル方向に応じたリサイズ計算 (最小サイズ保証)
  function computeResize(
    orig: { x: number; y: number; width: number; height: number },
    handle: string,
    dx: number,
    dy: number
  ): { x: number; y: number; width: number; height: number } {
    let { x, y, width, height } = orig;
    const minSize = 5;

    if (handle.includes("w")) {
      const newX = Math.min(x + dx, x + width - minSize);
      width = width - (newX - x);
      x = newX;
    }
    if (handle.includes("e")) {
      width = Math.max(minSize, width + dx);
    }
    if (handle.includes("n")) {
      const newY = Math.min(y + dy, y + height - minSize);
      height = height - (newY - y);
      y = newY;
    }
    if (handle.includes("s")) {
      height = Math.max(minSize, height + dy);
    }
    return { x, y, width, height };
  }

  function handleCursorForHandle(handle: string): string {
    const map: Record<string, string> = {
      nw: "nwse-resize", se: "nwse-resize",
      ne: "nesw-resize", sw: "nesw-resize",
      n: "ns-resize", s: "ns-resize",
      e: "ew-resize", w: "ew-resize",
    };
    return map[handle] ?? "default";
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
      <!-- 8方向リサイズハンドル -->
      {#each [
        ["nw", mask.x, mask.y],
        ["n", mask.x + mask.width / 2, mask.y],
        ["ne", mask.x + mask.width, mask.y],
        ["w", mask.x, mask.y + mask.height / 2],
        ["e", mask.x + mask.width, mask.y + mask.height / 2],
        ["sw", mask.x, mask.y + mask.height],
        ["s", mask.x + mask.width / 2, mask.y + mask.height],
        ["se", mask.x + mask.width, mask.y + mask.height],
      ] as [name, hx, hy]}
        <rect
          x={Number(hx) - HANDLE_SIZE / 2}
          y={Number(hy) - HANDLE_SIZE / 2}
          width={HANDLE_SIZE}
          height={HANDLE_SIZE}
          fill="white"
          stroke="#0066cc"
          stroke-width="1.5"
          style:cursor={handleCursorForHandle(String(name))}
          class="handle"
        />
      {/each}
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

  .handle {
    pointer-events: auto;
  }
</style>
