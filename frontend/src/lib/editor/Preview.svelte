<script lang="ts">
  import { onMount, onDestroy } from "svelte";

  import { preview } from "$lib/state/preview.svelte";
  import init, { get_css } from "$lib/converter/pkg/converter";
  import { scrollPreviewToLine } from "$lib/editor/scroll";

  let styleEl: Element;
  let previewEl = $state(<HTMLElement | null>null);

  onMount(async () => {
    styleEl = document.createElement("style");
    await init();
    styleEl.textContent = get_css();
    document.head.appendChild(styleEl);
  });

  onDestroy(() => styleEl?.remove());

  $effect(() => {
    if (previewEl) {
      preview.scrollTick;
      scrollPreviewToLine(
        preview.targetLine,
        previewEl,
        preview.scrollBehavior,
      );
    }
  });
</script>

<div
  bind:this={previewEl}
  class="w-full h-full border-none bg-white overflow-y-auto text-black"
>
  {@html preview.html}
</div>
