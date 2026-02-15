<script lang="ts">
  import { exportMenuState, ExportTypes } from "./export.svelte";

  let { exportFile } = $props();
</script>

<div class="relative inline-block text-left">
  {#if exportMenuState.isOpen}
    <!-- svelte-ignore a11y_click_events_have_key_events -->
    <!-- svelte-ignore a11y_no_noninteractive_element_interactions -->
    <div
      class="fixed inset-0 z-20"
      role="region"
      onclick={() => (exportMenuState.isOpen = false)}
    ></div>
    <div
      class="absolute right-0 mt-2 w-32 origin-top-right rounded-md bg-[#2d2d2d] border border-gray-600 shadow-lg ring-1 ring-black ring-opacity-5 z-50 focus:outline-none"
    >
      <div class="py-1">
        <button
          class="text-gray-300 block w-full px-4 py-2 text-left text-xs hover:bg-[#3c3c3c] transition"
          onclick={() => {
            exportFile(ExportTypes.Fountain);
            exportMenuState.isOpen = false;
          }}
        >
          Fountain
        </button>
        <button
          class="text-gray-300 block w-full px-4 py-2 text-left text-xs hover:bg-[#3c3c3c] transition"
          onclick={() => {
            exportFile(ExportTypes.Html);
            exportMenuState.isOpen = false;
          }}
        >
          HTML
        </button>
        <button
          class="text-gray-300 block w-full px-4 py-2 text-left text-xs hover:bg-[#3c3c3c] transition"
          onclick={() => {
            exportFile(ExportTypes.Pdf);
            exportMenuState.isOpen = false;
          }}
        >
          PDF
        </button>
      </div>
    </div>
  {/if}
</div>
