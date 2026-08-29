<script>
  import Modal from "$lib/common/Modal.svelte";
  import { deleteRoom } from "$lib/gallary/api";
  import { GallaryModals, gallaryState } from "$lib/state/gallary.svelte";

  let deleteConfirmInput = $state("");

  function closeDeleteModal() {
    deleteConfirmInput = "";
    gallaryState.modalOpen = GallaryModals.None;
  }

  async function handleDelete() {
    if (
      deleteConfirmInput !==
      gallaryState.roomList.get(gallaryState.targetedId)?.name
    )
      return;
    await deleteRoom(gallaryState.targetedId);
    closeDeleteModal();
  }
</script>

{#if gallaryState.modalOpen == GallaryModals.Remove}
  <Modal title="Remove script" closeModal={closeDeleteModal}>
    <p class="text-xs text-gray-400 mb-4">
      Type
      <br />
      <span class="font-mono text-gray-300"
        >{gallaryState.roomList.get(gallaryState.targetedId)?.name}</span
      >
      <br />
      to confirm deletion.
    </p>
    <input
      type="text"
      placeholder="Type document ID to confirm"
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
          gallaryState.roomList.get(gallaryState.targetedId)?.name}
        class:text-white={deleteConfirmInput ===
          gallaryState.roomList.get(gallaryState.targetedId)?.name}
        class:border-red-600={deleteConfirmInput ===
          gallaryState.roomList.get(gallaryState.targetedId)?.name}
        class:hover:bg-red-600={deleteConfirmInput ===
          gallaryState.roomList.get(gallaryState.targetedId)?.name}
        class:bg-[#3c3c3c]={deleteConfirmInput !==
          gallaryState.roomList.get(gallaryState.targetedId)?.name}
        class:text-gray-500={deleteConfirmInput !==
          gallaryState.roomList.get(gallaryState.targetedId)?.name}
        class:border-gray-600={deleteConfirmInput !==
          gallaryState.roomList.get(gallaryState.targetedId)?.name}
        disabled={deleteConfirmInput !==
          gallaryState.roomList.get(gallaryState.targetedId)?.name}
        onclick={handleDelete}
      >
        Delete
      </button>
    </div>
  </Modal>
{/if}
