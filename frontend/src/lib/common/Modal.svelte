<script lang="ts">
  import { GalleryModals, galleryState } from "$lib/state/gallery.svelte";
  import type { Snippet } from "svelte";

  function defaultCloseModal() {
    galleryState.modalOpen = GalleryModals.None;
  }

  let {
    title = "",
    closeModal = defaultCloseModal,
    children,
  }: {
    title?: string;
    closeModal?: () => void;
    children?: Snippet;
  } = $props();
</script>

<div class="fixed inset-0 z-50 flex items-center justify-center">
  <div
    class="absolute inset-0 bg-black/50"
    role="presentation"
    onclick={closeModal}
  ></div>
  <div
    class="relative bg-[#252526] border border-gray-700 rounded-lg shadow-xl p-6 w-full max-w-sm z-10"
  >
    <h2 class="text-sm font-medium text-gray-200 mb-4">{title}</h2>
    {@render children?.()}
  </div>
</div>
