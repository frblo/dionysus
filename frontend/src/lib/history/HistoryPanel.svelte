<script lang="ts">
  import { ClockHistory } from "svelte-bootstrap-icons";
  import type { VersionEntry } from "$lib/history/api";

  let {
    versions,
    selectedSeq,
    loading = false,
    onSelect,
  }: {
    versions: VersionEntry[];
    selectedSeq: number | null;
    loading?: boolean;
    onSelect: (seq: number) => void;
  } = $props();

  function formatTimestamp(value: string) {
    return new Date(value).toLocaleString(undefined, {
      dateStyle: "medium",
      timeStyle: "short",
    });
  }
</script>

<aside
  class="relative h-full w-72 shrink-0 bg-[#252526] border-r border-gray-700 flex flex-col z-30"
  aria-busy={loading}
>
  <div
    class="px-4 py-3 border-b border-gray-700 flex justify-between items-center"
  >
    <span class="text-xs font-bold uppercase tracking-wider text-gray-400"
      >Version history</span
    >
    <span class="text-xs text-gray-500">{versions.length}</span>
  </div>

  <div class="flex-1 overflow-y-auto custom-scrollbar">
    {#if versions.length === 0}
      <div class="flex flex-col items-center gap-2 p-6 text-center">
        <ClockHistory class="text-gray-600" width="24" height="24" />
        <p class="text-xs text-gray-500 italic">No versions recorded yet</p>
      </div>
    {:else}
      {#each versions as version, index (version.seq)}
        <button
          type="button"
          class="w-full text-left px-4 py-3 border-l-2 transition-colors {selectedSeq ===
          version.seq
            ? 'bg-[#37373d] border-blue-500'
            : 'border-transparent hover:bg-[#37373d]'}"
          onclick={() => onSelect(version.seq)}
          aria-current={selectedSeq === version.seq ? "true" : undefined}
        >
          <div class="flex items-center justify-between gap-2">
            <time
              datetime={version.createdAt}
              class="text-sm {selectedSeq === version.seq
                ? 'text-white'
                : 'text-gray-300'}"
            >
              {formatTimestamp(version.createdAt)}
            </time>

            {#if index === 0}
              <span
                class="shrink-0 rounded bg-blue-600/20 px-1.5 py-0.5 text-[10px] font-medium uppercase tracking-wide text-blue-300"
              >
                Latest
              </span>
            {/if}
          </div>
          <span class="mt-1 block text-[11px] text-gray-500"
            >Version {version.seq}</span
          >
        </button>
      {/each}
    {/if}
  </div>

  {#if loading}
    <div
      class="flex items-center gap-2 border-t border-gray-700 px-4 py-3 text-xs text-gray-400"
    >
      <span class="h-2 w-2 animate-pulse rounded-full bg-blue-400"></span>
      Loading version...
    </div>
  {/if}
</aside>
