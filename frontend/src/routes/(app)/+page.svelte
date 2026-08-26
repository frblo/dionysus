<script lang="ts">
  import Header from "$lib/common/Header.svelte";
  import Sidebar from "$lib/common/Sidebar.svelte";
  import {
    FileEarmarkPlus,
    FileEarmarkText,
    Trash,
  } from "svelte-bootstrap-icons";
  import type { PageProps } from "./$types";
  import { HomeModals, homeModalSettings } from "$lib/state/settings.svelte";

  let { data }: PageProps = $props();

  let createName = $state("");
  let deleteTargetId = $state("");
  let deleteConfirmInput = $state("");
  let hoveredRoomId = $state<string | null>(null);

  function openCreateModal() {
    createName = "";
    toggleModal(HomeModals.Create);
  }

  function closeCreateModal() {
    toggleModal(HomeModals.None);
  }

  function handleCreate() {
    if (!createName.trim()) return;
    // TODO: call API to create new screenplay
    console.log("Create screenplay:", createName);
    closeCreateModal();
  }

  function openDeleteModal(id: string) {
    deleteTargetId = id;
    deleteConfirmInput = "";
    toggleModal(HomeModals.Delete);
  }

  function closeDeleteModal() {
    toggleModal(HomeModals.None);
    deleteTargetId = "";
    deleteConfirmInput = "";
  }

  function handleDelete() {
    if (deleteConfirmInput !== deleteTargetId) return;
    // TODO: call API to delete screenplay
    console.log("Delete screenplay:", deleteTargetId);
    closeDeleteModal();
  }

  function toggleModal(modal: HomeModals) {
    homeModalSettings.modalOpen = modal;
  }

  const roomList = data.roomList;
</script>

<Header title="Screenplays"></Header>

<div class="flex flex-1 h-[calc(100vh-64px)] overflow-hidden">
  <Sidebar>
    <button
      class="p-2 text-gray-400 hover:text-white transition-colors"
      title="Create new screenplay"
      onclick={openCreateModal}
    >
      <FileEarmarkPlus />
    </button>
  </Sidebar>

  <main class="flex flex-1 overflow-auto bg-[#1e1e1e] p-6">
    <div
      class="grid grid-cols-[repeat(auto-fill,minmax(200px,1fr))] auto-rows-min gap-4 content-start w-full"
    >
      {#each roomList ?? [] as room ((room.room_id, room.room_name))}
        <a
          href="/document/{room.room_id}"
          class="relative group block h-40 rounded-lg border border-gray-700 bg-[#252526] p-4 hover:border-gray-500 hover:bg-[#2d2d2d] transition-colors no-underline"
          onmouseenter={() => (hoveredRoomId = room.room_id)}
          onmouseleave={() => (hoveredRoomId = null)}
        >
          <div class="flex h-full flex-col items-center">
            <div class="flex flex-1 items-center justify-center">
              <FileEarmarkText
                width="48"
                height="48"
                class="text-gray-400 group-hover:text-gray-300 transition-colors"
              />
            </div>
            <span
              title={room.room_name}
              class="w-full text-center text-xs leading-4 h-12 font-mono break-all text-gray-400 overflow-hidden [mask-image:linear-gradient(to_bottom,black_65%,transparent_100%)]"
            >
              {room.room_name}
            </span>
          </div>

          {#if hoveredRoomId === room.room_id}
            <button
              class="absolute top-2 right-2 p-1.5 rounded bg-red-600/80 hover:bg-red-500 text-white text-xs transition-colors z-10"
              title="Delete screenplay"
              onclick={(e) => {
                e.preventDefault();
                e.stopPropagation();
                openDeleteModal(room.room_id);
              }}
              onmouseenter={(e) => e.stopPropagation()}
            >
              <Trash />
            </button>
          {/if}
        </a>
      {/each}
    </div>
  </main>
</div>

{#if homeModalSettings.modalOpen == HomeModals.Create}
  <div class="fixed inset-0 z-50 flex items-center justify-center">
    <div
      class="absolute inset-0 bg-black/50"
      role="presentation"
      onclick={closeCreateModal}
    ></div>
    <div
      class="relative bg-[#252526] border border-gray-700 rounded-lg shadow-xl p-6 w-full max-w-sm z-10"
    >
      <h2 class="text-sm font-medium text-gray-200 mb-4">New Screenplay</h2>
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
    </div>
  </div>
{/if}

{#if homeModalSettings.modalOpen == HomeModals.Delete}
  <div class="fixed inset-0 z-50 flex items-center justify-center">
    <div
      class="absolute inset-0 bg-black/50"
      role="presentation"
      onclick={closeDeleteModal}
    ></div>
    <div
      class="relative bg-[#252526] border border-gray-700 rounded-lg shadow-xl p-6 w-full max-w-sm z-10"
    >
      <h2 class="text-sm font-medium text-red-400 mb-2">Delete Screenplay</h2>
      <p class="text-xs text-gray-400 mb-4">
        Type <span class="font-mono text-gray-300">{deleteTargetId}</span> to confirm
        deletion.
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
          class:bg-red-700={deleteConfirmInput === deleteTargetId}
          class:text-white={deleteConfirmInput === deleteTargetId}
          class:border-red-600={deleteConfirmInput === deleteTargetId}
          class:hover:bg-red-600={deleteConfirmInput === deleteTargetId}
          class:bg-[#3c3c3c]={deleteConfirmInput !== deleteTargetId}
          class:text-gray-500={deleteConfirmInput !== deleteTargetId}
          class:border-gray-600={deleteConfirmInput !== deleteTargetId}
          disabled={deleteConfirmInput !== deleteTargetId}
          onclick={handleDelete}
        >
          Delete
        </button>
      </div>
    </div>
  </div>
{/if}
