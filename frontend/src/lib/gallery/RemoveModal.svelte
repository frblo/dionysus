<script>
  import Modal from "$lib/common/Modal.svelte";
  import { deleteRoom } from "$lib/gallery/api";
  import { GalleryModals, galleryState } from "$lib/state/gallery.svelte";

  let deleteConfirmInput = $state("");

  function closeDeleteModal() {
    deleteConfirmInput = "";
    galleryState.modalOpen = GalleryModals.None;
  }

  async function handleDelete() {
    if (
      deleteConfirmInput !==
      galleryState.roomList.get(galleryState.targetedId)?.name
    )
      return;
    await deleteRoom(galleryState.targetedId);
    closeDeleteModal();
  }
</script>

{#if galleryState.modalOpen == GalleryModals.Remove}
  <Modal title="Remove screenplay" closeModal={closeDeleteModal}>
    <p class="text-xs text-gray-400 mb-4">
      Type
      <br />
      <span class="font-mono text-gray-300"
        >{galleryState.roomList.get(galleryState.targetedId)?.name}</span
      >
      <br />
      to confirm deletion.
    </p>
    <input
      type="text"
      placeholder="Type document name to confirm"
      bind:value={deleteConfirmInput}
      class="w-full px-3 py-2 rounded border border-gray-600 bg-[#1e1e1e] text-gray-200 text-sm placeholder-gray-500 focus:outline-none focus:border-red-500 mb-4"
      onkeydown={(e) => e.key === "Enter" && handleDelete()}
    />
    <div class="flex justify-end gap-2">
      <button
        class="px-3 py-1.5 rounded border border-gray-600 text-gray-400 text-xs hover:bg-[#3c3c3c] transition"
        onclick={closeDeleteModal}
      >
        Cancel
      </button>
      <button
        class="px-3 py-1.5 rounded text-xs transition border"
        class:bg-red-700={deleteConfirmInput ===
          galleryState.roomList.get(galleryState.targetedId)?.name}
        class:text-white={deleteConfirmInput ===
          galleryState.roomList.get(galleryState.targetedId)?.name}
        class:border-red-600={deleteConfirmInput ===
          galleryState.roomList.get(galleryState.targetedId)?.name}
        class:hover:bg-red-600={deleteConfirmInput ===
          galleryState.roomList.get(galleryState.targetedId)?.name}
        class:bg-[#3c3c3c]={deleteConfirmInput !==
          galleryState.roomList.get(galleryState.targetedId)?.name}
        class:text-gray-500={deleteConfirmInput !==
          galleryState.roomList.get(galleryState.targetedId)?.name}
        class:border-gray-600={deleteConfirmInput !==
          galleryState.roomList.get(galleryState.targetedId)?.name}
        disabled={deleteConfirmInput !==
          galleryState.roomList.get(galleryState.targetedId)?.name}
        onclick={handleDelete}
      >
        Delete
      </button>
    </div>
  </Modal>
{/if}
