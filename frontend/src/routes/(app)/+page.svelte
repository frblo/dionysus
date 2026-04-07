<script lang="ts">
  import { page } from "$app/state";

  import {
    Download,
    ExclamationCircle,
    FileText,
    ArrowLeftRight,
  } from "svelte-bootstrap-icons";

  import Editor from "$lib/editor/Editor.svelte";
  import Outline from "$lib/editor/Outline.svelte";
  import ExportMenu from "$lib/export/ExportMenu.svelte";
  import Preview from "$lib/editor/Preview.svelte";

  import {
    editorViewSettings,
    userSettings,
    SidebarMenus,
    PanelFocus,
  } from "$lib/state/settings.svelte";
  import { preview } from "$lib/state/preview.svelte";

  const COLORS = [
    "#e83d84", // cerise
    "#4fc4cf", // cyan
    "#f4a261", // warm orange
    "#7bed9f", // mint green
    "#a29bfe", // lavender
    "#ffd32a", // yellow
    "#ff6b6b", // coral red
    "#48dbfb", // sky blue
    "#ff9ff3", // light pink
    "#54a0ff", // blue
    "#5f27cd", // purple
    "#00d2d3", // teal
    "#ff9f43", // peach
    "#c8d6e5", // steel blue-grey
    "#01abc2", // deep cyan
    "#fd79a8", // rose
  ];

  let editorRef = $state(<Editor | null>null);

  let name = page.data.me?.display_name;
  let color = COLORS[Math.floor(Math.random() * COLORS.length)];

  const editorWidth = $derived(
    editorViewSettings.panelFocus === PanelFocus.EditorOnly
      ? "100%"
      : editorViewSettings.panelFocus === PanelFocus.PreviewOnly
        ? "0%"
        : "50%",
  );

  const previewWidth = $derived(
    editorViewSettings.panelFocus === PanelFocus.PreviewOnly
      ? "100%"
      : editorViewSettings.panelFocus === PanelFocus.EditorOnly
        ? "0%"
        : "50%",
  );

  function toggleSidebarMenu(menu: SidebarMenus) {
    editorViewSettings.sidebarMenuOpen =
      editorViewSettings.sidebarMenuOpen === menu ? SidebarMenus.None : menu;
  }
</script>

<header
  class="flex items-center justify-between px-4 py-3 bg-[#252526] border-b border-gray-700 shadow-md z-10"
>
  <div class="flex items-center gap-2 mt-5 ml-auto">
    <button
      class="px-3 py-1 rounded border border-gray-600 text-gray-400 text-xs font-medium hover:bg-[#3c3c3c] transition h-8"
      onclick={() => (userSettings.vimEnabled = !userSettings.vimEnabled)}
    >
      {userSettings.vimEnabled ? "Vim ON" : "Vim OFF"}
    </button>

    <select
      class="px-3 py-1 rounded border border-gray-600 text-gray-400 text-xs font-medium bg-[#252526] hover:bg-[#3c3c3c] transition h-8 cursor-pointer"
      bind:value={editorViewSettings.panelFocus}
    >
      <option value={PanelFocus.Both}>Split</option>
      <option value={PanelFocus.EditorOnly}>Editor only</option>
      <option value={PanelFocus.PreviewOnly}>Preview only</option>
    </select>

    <button
      class="px-4 py-1 rounded bg-blue-600 text-white text-xs font-bold hover:bg-blue-500 shadow-lg transition h-8 uppercase tracking-tight"
      onclick={() => {
        preview.jumpToLine(editorRef ? editorRef.getCursorLine() : 0);
      }}
      title="Sync cursor position"
    >
      <ArrowLeftRight />
    </button>

    <button
      class="px-3 py-1 rounded border border-gray-600 text-gray-400 text-xs font-medium hover:bg-[#3c3c3c] transition h-8"
      onclick={() =>
        (editorViewSettings.exportMenuOpen =
          !editorViewSettings.exportMenuOpen)}
      title="Export"
    >
      <Download />
    </button>

    <ExportMenu {editorRef} />
  </div>
</header>

<div class="flex flex-1 h-[calc(100vh-64px)] overflow-hidden">
  <!-- Side bar -->
  <aside
    class="w-12 bg-[#333333] border-r border-gray-700 flex flex-col items-center py-4 gap-4"
  >
    <button
      class="p-2 text-gray-400 hover:text-white transition-colors"
      class:text-white={editorViewSettings.sidebarMenuOpen ===
        SidebarMenus.Outline}
      title="Document outline"
      onclick={() => toggleSidebarMenu(SidebarMenus.Outline)}
    >
      <FileText />
    </button>

    <div class="mt-auto flex flex-col items-center gap-4">
      <a href="https://github.com/frblo/dionysus/issues" target="_blank">
        <button
          class="p-2 text-gray-400 hover:text-white transition-colors"
          title="Report issue"
        >
          <ExclamationCircle />
        </button>
      </a>
    </div>
  </aside>
  <Outline {editorRef} />

  <main class="flex flex-1 overflow-hidden bg-[#1e1e1e]">
    <section
      class="flex flex-col overflow-hidden transition-all duration-200 border-r border-gray-700"
      style="width: {editorWidth}"
    >
      <div class="flex-1 overflow-auto">
        <Editor bind:this={editorRef} user={{ name, color }} />
      </div>
    </section>

    <section
      class="flex flex-col overflow-hidden transition-all duration-200"
      style="width: {previewWidth}"
    >
      <Preview />
    </section>
  </main>
</div>
