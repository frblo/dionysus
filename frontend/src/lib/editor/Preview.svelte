<script lang="ts">
  import { onMount, onDestroy } from "svelte";

  import { preview } from "$lib/state/preview.svelte";
  import init, { get_css } from "$lib/converter/pkg/converter";

  let styleEl: Element;

  onMount(async () => {
    styleEl = document.createElement("style");
    await init();
    styleEl.textContent = get_css();
    document.head.appendChild(styleEl);
  });

  onDestroy(() => styleEl?.remove());
</script>

{#if preview.html}
  <div class="w-full h-full border-none bg-white overflow-y-auto text-black">
    {@html preview.html}
  </div>
{:else}
  <div
    class="flex flex-col items-center justify-center h-full text-gray-500 text-center p-12"
  >
    <p class="italic mb-2">No content rendered.</p>
    <p class="text-xs opacity-70">
      No content found. Try writing something and then click "RUN PREVIEW",
      press
      <kbd class="bg-gray-800 px-1 rounded text-gray-300">CTRL+S</kbd> or run
      command
      <kbd class="bg-gray-800 px-1 rounded text-gray-300">:w</kbd> to see preview...
    </p>
  </div>
{/if}
