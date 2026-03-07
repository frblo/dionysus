<script lang="ts">
  import { preview } from "$lib/state/preview.svelte";
</script>

{#if preview.html}
  <iframe
    title="Screenplay Preview"
    srcdoc={preview.html}
    class="w-full h-full border-none bg-white"
  ></iframe>
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
