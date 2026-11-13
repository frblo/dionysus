<script lang="ts">
  import { afterNavigate } from "$app/navigation";
  import { page } from "$app/state";

  import { Download, FileText } from "svelte-bootstrap-icons";

  import Editor from "$lib/editor/HistoryEditor.svelte";
  import Outline from "$lib/editor/Outline.svelte";
  import Settings from "$lib/editor/Settings.svelte";
  import ExportMenu from "$lib/export/ExportMenu.svelte";

  import {
    editorViewSettings,
    SidebarMenus,
    PanelFocus,
  } from "$lib/state/settings.svelte";
  import { preview } from "$lib/state/preview.svelte";
  import Header from "$lib/common/Header.svelte";
  import Sidebar from "$lib/common/Sidebar.svelte";

  let editorRef = $state(<Editor | null>null);

  const roomId = page.data.roomInfo?.room_id;

  const editorWidth = $derived(
    editorViewSettings.panelFocus === PanelFocus.EditorOnly
      ? "100%"
      : editorViewSettings.panelFocus === PanelFocus.PreviewOnly
        ? "0%"
        : "50%",
  );

  function toggleSidebarMenu(menu: SidebarMenus) {
    editorViewSettings.sidebarMenuOpen =
      editorViewSettings.sidebarMenuOpen === menu ? SidebarMenus.None : menu;
  }

  afterNavigate(() => preview.reset());
</script>

<svelte:head>
  <title>{page.data.roomInfo?.room_name} history - Dionysus</title>
</svelte:head>

<Header title="{page.data.roomInfo?.room_name} history">
  {#snippet leftChildren()}
    <p class="text-sm font-medium text-gray-400 truncate">READ ONLY</p>
  {/snippet}

  <select
    class="px-3 py-1 rounded border border-gray-600 text-gray-400 text-xs font-medium bg-[#252526] hover:bg-[#3c3c3c] transition h-8 cursor-pointer"
    bind:value={editorViewSettings.panelFocus}
  >
    <option value={PanelFocus.Both}>Split</option>
    <option value={PanelFocus.EditorOnly}>Editor only</option>
    <option value={PanelFocus.PreviewOnly}>Preview only</option>
  </select>

  <button
    class="px-3 py-1 rounded border border-gray-600 text-gray-400 text-xs font-medium hover:bg-[#3c3c3c] transition h-8"
    onclick={() =>
      (editorViewSettings.exportMenuOpen = !editorViewSettings.exportMenuOpen)}
    title="Export"
  >
    <Download />
  </button>

  <ExportMenu {editorRef} />
</Header>

<div class="flex flex-1 h-[calc(100vh-64px)] overflow-hidden">
  <Sidebar>
    <button
      class="p-2 text-gray-400 hover:text-white transition-colors"
      class:text-white={editorViewSettings.sidebarMenuOpen ===
        SidebarMenus.Outline}
      title="Document outline"
      onclick={() => toggleSidebarMenu(SidebarMenus.Outline)}
    >
      <FileText />
    </button>
  </Sidebar>
  <Outline {editorRef} />
  <Settings />

  <main class="flex flex-1 overflow-hidden bg-[#1e1e1e]">
    <section
      class="flex flex-col overflow-hidden transition-all duration-200 border-r border-gray-700"
      style="width: {editorWidth}"
    >
      <div class="flex-1 overflow-auto">
        <Editor bind:this={editorRef} room={roomId} />
      </div>
    </section>
  </main>
</div>
