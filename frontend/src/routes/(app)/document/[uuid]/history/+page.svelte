<script lang="ts">
  import { page } from "$app/state";

  import { ArrowLeft, Download } from "svelte-bootstrap-icons";

  import HistoryEditor from "$lib/editor/HistoryEditor.svelte";
  import Settings from "$lib/editor/Settings.svelte";
  import ExportMenu from "$lib/export/ExportMenu.svelte";

  import { editorViewSettings } from "$lib/state/settings.svelte";
  import Header from "$lib/common/Header.svelte";
  import Sidebar from "$lib/common/Sidebar.svelte";
  import HistoryPanel from "$lib/history/HistoryPanel.svelte";
  import { getVersion, type VersionEntry } from "$lib/history/api";

  let editorRef = $state(<HistoryEditor | null>null);

  const roomId = page.data.roomInfo?.room_id;
  const versions: VersionEntry[] = page.data.versions;

  let selectedSeq = $state<number | null>(page.data.initialSeq);
  let displayedSeq = $state<number | null>(page.data.initialSeq);
  let content = $state(page.data.initialVersion);
  let loading = $state(false);
  let error = $state<string | null>(null);
  let latestRequest = 0;

  async function selectVersion(seq: number) {
    if (!roomId || seq === selectedSeq) return;

    selectedSeq = seq;
    const request = ++latestRequest;
    loading = true;
    error = null;

    try {
      const version = await getVersion(fetch, roomId, seq);
      if (request !== latestRequest) return;
      content = version;
      displayedSeq = seq;
    } catch (e) {
      if (request !== latestRequest) return;
      error = e instanceof Error ? e.message : "Failed to load version";
      selectedSeq = displayedSeq;
    } finally {
      if (request === latestRequest) loading = false;
    }
  }
</script>

<svelte:head>
  <title>{page.data.roomInfo?.room_name} history - Dionysus</title>
</svelte:head>

<Header title="{page.data.roomInfo?.room_name} history">
  {#snippet leftChildren()}
    {#if roomId}
      <a
        href="/document/{roomId}"
        class="px-3 py-1 rounded border border-gray-600 text-gray-400 text-xs font-medium hover:bg-[#3c3c3c] transition h-8 flex items-center gap-1.5"
        title="Back to editor"
      >
        <ArrowLeft />
        Editor
      </a>
    {/if}
    <p class="text-sm font-medium text-gray-400 truncate">READ ONLY</p>
  {/snippet}

  <button
    class="px-3 py-1 rounded border border-gray-600 text-gray-400 text-xs font-medium hover:bg-[#3c3c3c] transition h-8"
    onclick={() =>
      (editorViewSettings.exportMenuOpen = !editorViewSettings.exportMenuOpen)}
    title="Export this version"
  >
    <Download />
  </button>

  <ExportMenu {editorRef} />
</Header>

<div class="flex flex-1 h-[calc(100vh-64px)] overflow-hidden">
  <Sidebar />
  <HistoryPanel {versions} {selectedSeq} {loading} onSelect={selectVersion} />
  <Settings />

  <main class="flex flex-1 overflow-hidden bg-[#1e1e1e]">
    <div class="flex-1 overflow-auto">
      <div class="mx-auto w-full max-w-4xl px-6 py-4">
        {#if error}
          <div
            class="mb-3 rounded border border-red-800 bg-red-950/50 px-3 py-2 text-sm text-red-300"
          >
            {error}
          </div>
        {/if}

        <HistoryEditor bind:this={editorRef} {content} />
      </div>
    </div>
  </main>
</div>
