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
  import { formatVersionTimestamp } from "$lib/history/format";

  let editorRef = $state(<HistoryEditor | null>null);

  const roomId = page.data.roomInfo?.room_id;
  const versions: VersionEntry[] = page.data.versions;

  const versionCache = new Map<number, string>();
  if (page.data.initialSeq !== null) {
    versionCache.set(page.data.initialSeq, page.data.initialVersion);
  }
  if (
    page.data.initialPreviousSeq !== null &&
    page.data.initialPreviousVersion !== null
  ) {
    versionCache.set(
      page.data.initialPreviousSeq,
      page.data.initialPreviousVersion,
    );
  }

  let selectedSeq = $state<number | null>(page.data.initialSeq);
  let displayedSeq = $state<number | null>(page.data.initialSeq);
  let content = $state(page.data.initialVersion);
  let previousContent = $state<string | null>(page.data.initialPreviousVersion);
  let loading = $state(false);
  let error = $state<string | null>(null);
  let latestRequest = 0;

  const displayedIndex = $derived(
    versions.findIndex((version) => version.seq === displayedSeq),
  );
  const previousVersion = $derived(
    displayedIndex >= 0 ? (versions[displayedIndex + 1] ?? null) : null,
  );

  async function loadVersion(seq: number): Promise<string> {
    const cached = versionCache.get(seq);
    if (cached !== undefined) return cached;

    if (!roomId) throw new Error("Missing room id");

    const version = await getVersion(fetch, roomId, seq);
    versionCache.set(seq, version);
    return version;
  }

  async function selectVersion(seq: number) {
    if (!roomId || seq === selectedSeq) return;

    const index = versions.findIndex((version) => version.seq === seq);
    const previous = index >= 0 ? (versions[index + 1] ?? null) : null;

    selectedSeq = seq;
    const request = ++latestRequest;
    loading = true;
    error = null;

    try {
      const [version, previousVersionContent] = await Promise.all([
        loadVersion(seq),
        previous ? loadVersion(previous.seq) : Promise.resolve(null),
      ]);
      if (request !== latestRequest) return;

      content = version;
      previousContent = previousVersionContent;
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

<Header title="{page.data.roomInfo?.room_name} - history">
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

        {#if displayedSeq !== null}
          <p class="mb-2 text-xs text-gray-500">
            {#if previousVersion}
              Changes since
              <time datetime={previousVersion.createdAt}>
                {formatVersionTimestamp(previousVersion.createdAt)}
              </time>
            {:else}
              First recorded version - no previous version to compare.
            {/if}
          </p>
        {/if}

        <HistoryEditor bind:this={editorRef} {content} {previousContent} />
      </div>
    </div>
  </main>
</div>
