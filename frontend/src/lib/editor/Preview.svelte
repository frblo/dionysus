<script lang="ts">
  import { onMount, onDestroy } from "svelte";

  import { preview } from "$lib/state/preview.svelte";
  import init, { get_css } from "$lib/converter/pkg/converter";
  import { scrollPreviewToLine } from "$lib/editor/scroll";
  import { editor } from "$lib/state/editor.svelte";

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

<!-- Not a critical function, more important to look clean. -->
<!-- svelte-ignore a11y_click_events_have_key_events -->
<!-- svelte-ignore a11y_no_static_element_interactions -->
<div
  bind:this={previewEl}
  class="w-full h-full border-none bg-white overflow-y-auto text-black"
  onclick={(e) => {
    const el = (e.target as HTMLElement).closest("[data-start-line]");
    if (!el) return;
    const line = parseInt((el as HTMLElement).dataset.startLine!);
    editor.scrollToLine(line);
  }}
>
  {@html preview.html}
</div>
