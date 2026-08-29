<script>
  import Modal from "$lib/common/Modal.svelte";
  import { createRoom } from "$lib/gallary/api";
  import { goto } from "$app/navigation";
  import { GallaryModals, gallaryState } from "$lib/state/gallary.svelte";

  let createName = $state("");

  async function handleCreate() {
    if (!createName.trim()) return;
    const createdRoomId = await createRoom(createName);
    closeCreateModal();
    goto(`/document/${createdRoomId}`);
  }

  function closeCreateModal() {
    createName = "";
    gallaryState.modalOpen = GallaryModals.None;
  }
</script>

{#if gallaryState.modalOpen == GallaryModals.Create}
  <Modal title="Create" closeModal={closeCreateModal}>
    <input
      type="text"
      placeholder="Enter screenplay name"
      bind:value={createName}
      class="w-full px-3 py-2 rounded border border-gray-600 bg-[#1e1e1e] text-gray-200 text-sm placeholder-gray-500 focus:outline-none focus:border-gray-400 mb-4"
      onkeydown={(e) => e.key === "Enter" && handleCreate()}
    />
    <div class="flex justify-end gap-2">
      <button
        class="px-3 py-1.5 rounded border border-gray-600 text-gray-400 text-xs hover:bg-[#3c3c3c] transition"
        onclick={closeCreateModal}
      >
        Cancel
      </button>
      <button
        class="px-3 py-1.5 rounded bg-[#3c3c3c] text-gray-200 text-xs hover:bg-[#4a4a4a] transition border border-gray-600"
        onclick={handleCreate}
      >
        Create
      </button>
    </div>
  </Modal>
{/if}
