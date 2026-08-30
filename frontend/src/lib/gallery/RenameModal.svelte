<script>
  import Modal from "$lib/common/Modal.svelte";
  import { renameRoom } from "$lib/gallery/api";
  import { GalleryModals, galleryState } from "$lib/state/gallery.svelte";

  let renameName = $state(
    galleryState.roomList.get(galleryState.targetedId)?.name ?? "",
  );

  async function handleRename() {
    if (!renameName.trim()) return;
    await renameRoom(galleryState.targetedId, renameName);
    closeRenameModal();
  }

  function closeRenameModal() {
    renameName = "";
    galleryState.modalOpen = GalleryModals.None;
  }
</script>

{#if galleryState.modalOpen == GalleryModals.Rename}
  <Modal title="Rename screenplay" closeModal={closeRenameModal}>
    <input
      type="text"
      placeholder={galleryState.roomList.get(galleryState.targetedId)?.name}
      bind:value={renameName}
      class="w-full px-3 py-2 rounded border border-gray-600 bg-[#1e1e1e] text-gray-200 text-sm placeholder-gray-500 focus:outline-none focus:border-gray-400 mb-4"
      onkeydown={(e) => e.key === "Enter" && handleRename()}
    />
    <div class="flex justify-end gap-2">
      <button
        class="px-3 py-1.5 rounded border border-gray-600 text-gray-400 text-xs hover:bg-[#3c3c3c] transition"
        onclick={closeRenameModal}
      >
        Cancel
      </button>
      <button
        class="px-3 py-1.5 rounded bg-[#3c3c3c] text-gray-200 text-xs hover:bg-[#4a4a4a] transition border border-gray-600"
        onclick={handleRename}
      >
        Rename
      </button>
    </div>
  </Modal>
{/if}
