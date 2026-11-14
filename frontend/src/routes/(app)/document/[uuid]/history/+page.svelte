<script lang="ts">
  import { goto } from "$app/navigation";
  import { page } from "$app/state";

  import {
    ArrowCounterclockwise,
    ArrowLeft,
    Download,
  } from "svelte-bootstrap-icons";

  import Modal from "$lib/common/Modal.svelte";
  import HistoryEditor from "$lib/editor/HistoryEditor.svelte";
  import Settings from "$lib/editor/Settings.svelte";
  import ExportMenu from "$lib/export/ExportMenu.svelte";

  import { editorViewSettings } from "$lib/state/settings.svelte";
  import { sessionState } from "$lib/state/session.svelte";
  import Header from "$lib/common/Header.svelte";
  import Sidebar from "$lib/common/Sidebar.svelte";
  import HistoryPanel from "$lib/history/HistoryPanel.svelte";
  import {
    getVersion,
    restoreVersion,
    type VersionEntry,
  } from "$lib/history/api";
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

  let restoreTarget = $state<VersionEntry | null>(null);
  let restoring = $state(false);
  let restoreError = $state<string | null>(null);

  const canRestore = $derived(
    page.data.roomInfo?.role !== "viewer" ||
      sessionState.current?.global_role === "admin",
  );

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

  function openRestoreModal() {
    if (displayedSeq === null) return;

    const version = versions.find((entry) => entry.seq === displayedSeq);
    if (!version) return;

    restoreTarget = version;
    restoreError = null;
  }

  function closeRestoreModal() {
    if (restoring) return;

    restoreTarget = null;
    restoreError = null;
  }

  async function confirmRestore() {
    if (!roomId || !restoreTarget) return;

    restoring = true;
    restoreError = null;

    try {
      await restoreVersion(fetch, roomId, restoreTarget.seq);
      restoreTarget = null;
      restoring = false;
      await goto(`/document/${roomId}`);
    } catch (e) {
      restoreError =
        e instanceof Error ? e.message : "Failed to restore document version";
      restoring = false;
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

  {#if canRestore}
    <button
      class="px-3 py-1 rounded border border-gray-600 text-gray-400 text-xs font-medium hover:bg-[#3c3c3c] transition h-8 flex items-center gap-1.5 disabled:cursor-not-allowed disabled:opacity-50 disabled:hover:bg-transparent"
      onclick={openRestoreModal}
      disabled={loading || restoring || displayedSeq === null}
      title="Restore the document to this version"
    >
      <ArrowCounterclockwise />
      Restore
    </button>
  {/if}

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

{#if restoreTarget}
  <Modal title="Restore document" closeModal={closeRestoreModal}>
    <p class="mb-4 text-xs text-gray-400">
      Restore the document to the version from
      <span class="text-gray-200">
        {formatVersionTimestamp(restoreTarget.createdAt)}
      </span>
      ? This replaces the current document for everyone editing it. The version history
      is kept and the restore is recorded as a new version.
    </p>

    {#if restoreError}
      <p
        class="mb-4 rounded border border-red-800 bg-red-950/50 px-3 py-2 text-xs text-red-300"
      >
        {restoreError}
      </p>
    {/if}

    <div class="flex justify-end gap-2">
      <button
        class="px-3 py-1.5 rounded border border-gray-600 text-gray-400 text-xs hover:bg-[#3c3c3c] transition disabled:cursor-not-allowed disabled:opacity-50"
        onclick={closeRestoreModal}
        disabled={restoring}
      >
        Cancel
      </button>
      <button
        class="px-3 py-1.5 rounded bg-[#3c3c3c] text-gray-200 text-xs hover:bg-[#4a4a4a] transition border border-gray-600 disabled:cursor-not-allowed disabled:opacity-50"
        onclick={confirmRestore}
        disabled={restoring}
      >
        {restoring ? "Restoring..." : "Restore"}
      </button>
    </div>
  </Modal>
{/if}
