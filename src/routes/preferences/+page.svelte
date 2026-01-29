<script lang="ts">
  import { onMount } from "svelte";
  import { load, type Store } from "@tauri-apps/plugin-store";
  import { open } from "@tauri-apps/plugin-dialog";

  type SaveMode = "tmp" | "macos_default" | "custom";

  let saveMode = $state<SaveMode>("tmp");
  let customPath = $state("");
  let store = $state<Store | null>(null);

  onMount(async () => {
    store = await load("settings.json");
    const saved = await store.get<string>("save_directory");
    if (saved) {
      if (saved === "tmp") {
        saveMode = "tmp";
      } else if (saved === "macos_default") {
        saveMode = "macos_default";
      } else if (saved.startsWith("custom:")) {
        saveMode = "custom";
        customPath = saved.slice("custom:".length);
      }
    }
  });

  async function save() {
    if (!store) return;
    let value: string;
    if (saveMode === "tmp") {
      value = "tmp";
    } else if (saveMode === "macos_default") {
      value = "macos_default";
    } else {
      value = `custom:${customPath}`;
    }
    await store.set("save_directory", value);
    await store.save();
  }

  async function selectFolder() {
    const selected = await open({ directory: true, multiple: false });
    if (selected) {
      customPath = selected as string;
      saveMode = "custom";
      await save();
    }
  }

  async function onModeChange(mode: SaveMode) {
    saveMode = mode;
    if (mode === "custom" && !customPath) {
      // フォルダ未選択なら選択ダイアログを開く
      await selectFolder();
      return;
    }
    await save();
  }
</script>

<div class="min-h-screen bg-[#1a1a1a] text-white p-6 font-[-apple-system,BlinkMacSystemFont,'Segoe_UI',Roboto,sans-serif]">
  <h2 class="text-xl font-semibold mb-6">Preferences</h2>

  <section>
    <h3 class="text-sm font-semibold text-gray-400 uppercase tracking-wider mb-4">
      Save Location
    </h3>

    <label class="flex items-start gap-3 px-3 py-2.5 rounded-lg cursor-pointer hover:bg-[#2d2d2d] transition-colors">
      <input
        type="radio"
        name="saveMode"
        value="tmp"
        checked={saveMode === "tmp"}
        onchange={() => onModeChange("tmp")}
        class="mt-0.5 accent-blue-600"
      />
      <div>
        <div class="text-sm font-medium">/tmp/flashcap/</div>
        <div class="text-xs text-gray-500 mt-0.5">Temporary directory (default)</div>
      </div>
    </label>

    <label class="flex items-start gap-3 px-3 py-2.5 rounded-lg cursor-pointer hover:bg-[#2d2d2d] transition-colors">
      <input
        type="radio"
        name="saveMode"
        value="macos_default"
        checked={saveMode === "macos_default"}
        onchange={() => onModeChange("macos_default")}
        class="mt-0.5 accent-blue-600"
      />
      <div>
        <div class="text-sm font-medium">macOS Default</div>
        <div class="text-xs text-gray-500 mt-0.5">System screenshot save location (Desktop or custom)</div>
      </div>
    </label>

    <label class="flex items-start gap-3 px-3 py-2.5 rounded-lg cursor-pointer hover:bg-[#2d2d2d] transition-colors">
      <input
        type="radio"
        name="saveMode"
        value="custom"
        checked={saveMode === "custom"}
        onchange={() => onModeChange("custom")}
        class="mt-0.5 accent-blue-600"
      />
      <div>
        <div class="text-sm font-medium">Custom Folder</div>
        <div class="text-xs text-gray-500 mt-0.5">Choose a specific folder</div>
      </div>
    </label>

    {#if saveMode === "custom"}
      <div class="flex items-center gap-3 ml-9 mt-1 px-3 py-2 bg-[#2d2d2d] rounded-md">
        <span class="text-[13px] text-gray-400 flex-1 overflow-hidden text-ellipsis whitespace-nowrap">
          {customPath || "(not selected)"}
        </span>
        <button
          class="px-3.5 py-1.5 bg-blue-600 hover:bg-blue-500 text-white text-[13px] rounded-md cursor-pointer whitespace-nowrap border-none"
          onclick={selectFolder}
        >
          Browse...
        </button>
      </div>
    {/if}
  </section>
</div>
