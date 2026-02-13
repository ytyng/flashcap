<script lang="ts">
  import type { TextAnnotation, TextSettings } from "./types";

  interface Props {
    texts: TextAnnotation[];
    settings: TextSettings;
    toolActive: boolean;
    interactive: boolean;
    scale: number;
    onTextsChange: (texts: TextAnnotation[]) => void;
    onBeforeMutate?: () => void;
  }

  let { texts, settings, toolActive, interactive, scale, onTextsChange, onBeforeMutate }: Props = $props();

  let selectedId = $state<string | null>(null);
  let editingId = $state<string | null>(null);
  let dragging = $state<"move" | null>(null);
  let dragStart = $state<{ x: number; y: number } | null>(null);
  let dragOrigPos = $state<{ x: number; y: number } | null>(null);
  let hoverCursor = $state<string>("default");

  // 新規テキスト作成中の一時状態
  let pendingText = $state<TextAnnotation | null>(null);

  function getSvgCoords(e: MouseEvent): { x: number; y: number } | null {
    const svg = (e.currentTarget as SVGSVGElement) ?? (e.target as Element).closest("svg");
    if (!svg) return null;
    const rect = svg.getBoundingClientRect();
    return { x: (e.clientX - rect.left) / scale, y: (e.clientY - rect.top) / scale };
  }

  // テキストのバウンディングボックスを推定
  function getTextBBox(t: TextAnnotation): { x: number; y: number; width: number; height: number } {
    // 各行を考慮した幅推定
    const lines = t.text.split("\n");
    const lineHeight = t.fontSize * 1.3;
    const maxLineWidth = Math.max(...lines.map((line) => line.length * t.fontSize * 0.6), t.fontSize * 2);
    return {
      x: t.x,
      y: t.y,
      width: maxLineWidth + 8,
      height: lineHeight * lines.length + 8,
    };
  }

  function hitTestText(t: TextAnnotation, px: number, py: number): boolean {
    const bbox = getTextBBox(t);
    return px >= bbox.x && px <= bbox.x + bbox.width && py >= bbox.y && py <= bbox.y + bbox.height;
  }

  function handleMouseDown(e: MouseEvent) {
    // 編集中の input クリックは透過
    if ((e.target as Element).closest(".text-edit-input")) return;

    const pt = getSvgCoords(e);
    if (!pt) return;

    // 選択済みテキストの移動開始
    if (selectedId && !editingId) {
      const sel = texts.find((t) => t.id === selectedId);
      if (sel && hitTestText(sel, pt.x, pt.y)) {
        onBeforeMutate?.();
        dragging = "move";
        dragStart = pt;
        dragOrigPos = { x: sel.x, y: sel.y };
        return;
      }
    }

    // 既存テキストをクリックで選択
    const hit = [...texts].reverse().find((t) => hitTestText(t, pt.x, pt.y));
    if (hit) {
      if (selectedId === hit.id) {
        // ダブルクリック相当: 再編集
        editingId = hit.id;
      } else {
        selectedId = hit.id;
        editingId = null;
      }
      return;
    }

    if (!toolActive) {
      commitEditing();
      selectedId = null;
      editingId = null;
      return;
    }

    // 新規テキスト作成
    commitEditing();
    onBeforeMutate?.();
    const id = crypto.randomUUID();
    pendingText = {
      id,
      x: pt.x,
      y: pt.y,
      text: "",
      fontSize: settings.fontSize,
      color: settings.color,
      bold: settings.bold,
      italic: settings.italic,
      whiteStroke: settings.whiteStroke,
      dropShadow: settings.dropShadow,
    };
    editingId = id;
    selectedId = id;
  }

  function handleMouseMove(e: MouseEvent) {
    const pt = getSvgCoords(e);
    if (!pt) return;

    if (dragging === "move" && selectedId && dragStart && dragOrigPos) {
      const dx = pt.x - dragStart.x;
      const dy = pt.y - dragStart.y;
      onTextsChange(
        texts.map((t) =>
          t.id === selectedId
            ? { ...t, x: dragOrigPos!.x + dx, y: dragOrigPos!.y + dy }
            : t
        )
      );
      return;
    }

    if (!dragging) {
      updateHoverCursor(pt);
    }
  }

  function handleMouseUp() {
    dragging = null;
    dragStart = null;
    dragOrigPos = null;
  }

  function updateHoverCursor(pt: { x: number; y: number }) {
    if (dragging) return;

    if (selectedId && !editingId) {
      const sel = texts.find((t) => t.id === selectedId);
      if (sel && hitTestText(sel, pt.x, pt.y)) {
        hoverCursor = "move";
        return;
      }
    }

    const hit = [...texts].reverse().find((t) => hitTestText(t, pt.x, pt.y));
    if (hit) {
      hoverCursor = "pointer";
      return;
    }

    hoverCursor = toolActive ? "text" : "default";
  }

  function commitEditing() {
    if (pendingText) {
      if (pendingText.text.trim()) {
        onTextsChange([...texts, pendingText]);
      }
      pendingText = null;
    }
    editingId = null;
  }

  function handleEditInput(e: Event, id: string) {
    const value = (e.target as HTMLTextAreaElement).value;
    if (pendingText && pendingText.id === id) {
      pendingText = { ...pendingText, text: value };
    } else {
      onTextsChange(texts.map((t) => (t.id === id ? { ...t, text: value } : t)));
    }
  }

  function handleEditKeyDown(e: KeyboardEvent) {
    // Escape で編集を確定して終了
    if (e.key === "Escape") {
      e.preventDefault();
      e.stopPropagation();
      commitEditing();
      return;
    }
    // Cmd+Enter で確定
    if (e.key === "Enter" && e.metaKey && !(e as any).isComposing) {
      e.preventDefault();
      commitEditing();
      return;
    }
    // 通常の Enter はそのまま改行（textarea のデフォルト動作）
    // Backspace/Delete がグローバルに伝播しないようにする
    e.stopPropagation();
  }

  function handleKeyDown(e: KeyboardEvent) {
    if (editingId) return; // 編集中はグローバルショートカット無効
    if ((e.key === "Delete" || e.key === "Backspace") && selectedId) {
      e.preventDefault();
      e.stopPropagation();
      onBeforeMutate?.();
      onTextsChange(texts.filter((t) => t.id !== selectedId));
      selectedId = null;
    }
  }

  // 表示用: テキスト一覧 (pending を含む)
  let allTexts = $derived(pendingText ? [...texts, pendingText] : texts);

  function focusOnMount(node: HTMLElement) {
    setTimeout(() => node.focus(), 100);
  }

  // 編集中/選択中のテキストの属性を更新する
  export function updateActiveAttribute(key: string, value: unknown) {
    const targetId = editingId ?? selectedId;
    if (!targetId) return;

    if (pendingText && pendingText.id === targetId) {
      pendingText = { ...pendingText, [key]: value };
    } else {
      onTextsChange(texts.map((t) => (t.id === targetId ? { ...t, [key]: value } : t)));
    }
  }

  export function deselect() {
    commitEditing();
    selectedId = null;
    editingId = null;
  }
</script>

<svelte:window onkeydown={handleKeyDown} />

<!-- svelte-ignore a11y_no_static_element_interactions -->
<svg
  class="text-overlay"
  onmousedown={handleMouseDown}
  onmousemove={handleMouseMove}
  onmouseup={handleMouseUp}
  style:cursor={dragging === "move" ? "move" : hoverCursor}
  style:pointer-events={interactive ? "auto" : "none"}
>
  <defs>
    <filter id="text-shadow" x="-20%" y="-20%" width="140%" height="140%">
      <feDropShadow dx="1" dy="1" stdDeviation="1.5" flood-opacity="0.6" />
    </filter>
  </defs>

  {#each allTexts as t (t.id)}
    {@const isSelected = t.id === selectedId}
    {@const isEditing = t.id === editingId}
    {@const filterAttr = t.dropShadow ? "url(#text-shadow)" : undefined}
    {@const bbox = getTextBBox(t)}
    {@const lineHeight = t.fontSize * 1.3}
    {@const fontWeight = t.bold ? "900" : "normal"}
    {@const fontStyle = t.italic ? "italic" : "normal"}

    {#if isEditing}
      <foreignObject
        x={t.x} y={t.y}
        width={Math.max(200, bbox.width + 40)}
        height={Math.max(lineHeight + 16, bbox.height + 16)}
      >
        <textarea
          xmlns="http://www.w3.org/1999/xhtml"
          class="text-edit-input"
          style="font-size:{t.fontSize}px;color:{t.color};line-height:{lineHeight}px;min-width:180px;min-height:{lineHeight}px;font-weight:{fontWeight};font-style:{fontStyle};"
          value={pendingText && pendingText.id === t.id ? pendingText.text : t.text}
          oninput={(e) => handleEditInput(e, t.id)}
          onkeydown={handleEditKeyDown}
          use:focusOnMount
        ></textarea>
      </foreignObject>
    {:else}
      {#if t.text}
        {#if t.dropShadow && t.whiteStroke}
          <!-- 白枠のみに影を適用 -->
          <g filter="url(#text-shadow)">
            {#each t.text.split("\n") as line, i}
              <text
                x={t.x + 4} y={t.y + t.fontSize + i * lineHeight}
                font-size={t.fontSize}
                font-family="-apple-system, BlinkMacSystemFont, 'Segoe UI', Roboto, sans-serif"
                font-weight={fontWeight}
                font-style={fontStyle}
                fill="none"
                stroke="white"
                stroke-width={3}
                stroke-linejoin="round"
              >{line}</text>
            {/each}
          </g>
          <!-- 本体は影なし -->
          {#each t.text.split("\n") as line, i}
            <text
              x={t.x + 4} y={t.y + t.fontSize + i * lineHeight}
              font-size={t.fontSize}
              font-family="-apple-system, BlinkMacSystemFont, 'Segoe UI', Roboto, sans-serif"
              font-weight={fontWeight}
              font-style={fontStyle}
              fill={t.color}
            >{line}</text>
          {/each}
        {:else}
          <g filter={filterAttr}>
            {#each t.text.split("\n") as line, i}
              <text
                x={t.x + 4} y={t.y + t.fontSize + i * lineHeight}
                font-size={t.fontSize}
                font-family="-apple-system, BlinkMacSystemFont, 'Segoe UI', Roboto, sans-serif"
                font-weight={fontWeight}
                font-style={fontStyle}
                fill={t.color}
                stroke={t.whiteStroke ? "white" : "none"}
                stroke-width={t.whiteStroke ? 3 : 0}
                stroke-linejoin="round"
                paint-order="stroke"
              >{line}</text>
            {/each}
          </g>
        {/if}
      {/if}

      {#if isSelected}
        <rect
          x={bbox.x} y={bbox.y} width={bbox.width} height={bbox.height}
          fill="none" stroke="#0066cc" stroke-width={2 / scale} stroke-dasharray="{4 / scale} {2 / scale}"
        />
      {/if}
    {/if}
  {/each}
</svg>

<style>
  .text-overlay {
    position: absolute;
    top: 0;
    left: 0;
    width: 100%;
    height: 100%;
  }

  .text-edit-input {
    background: rgba(255, 255, 255, 0.15);
    border: 1px solid #0066cc;
    border-radius: 2px;
    outline: none;
    padding: 2px 4px;
    font-family: -apple-system, BlinkMacSystemFont, 'Segoe UI', Roboto, sans-serif;
    resize: none;
    overflow: hidden;
    box-sizing: border-box;
    width: 100%;
    height: 100%;
  }
</style>
