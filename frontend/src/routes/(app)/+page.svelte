<script lang="ts">
  import Header from "$lib/common/Header.svelte";
  import Sidebar from "$lib/common/Sidebar.svelte";
  import {
    FileEarmarkPlus,
    FileEarmarkTextFill,
    PencilSquare,
    Trash,
  } from "svelte-bootstrap-icons";
  import { GalleryModals } from "$lib/state/gallery.svelte";
  import type { RoomInfo } from "./+page";
  import { onMount } from "svelte";
  import CreateModal from "$lib/gallery/CreateModal.svelte";
  import RemoveModal from "$lib/gallery/RemoveModal.svelte";
  import RenameModal from "$lib/gallery/RenameModal.svelte";
  import { galleryState } from "$lib/state/gallery.svelte";

  function openCreateModal() {
    galleryState.modalOpen = GalleryModals.Create;
  }

  function openRenameModal() {
    galleryState.targetedId = galleryState.hoveredRoomId?.toString() || "";
    galleryState.modalOpen = GalleryModals.Rename;
  }

  function openDeleteModal() {
    galleryState.targetedId = galleryState.hoveredRoomId?.toString() || "";
    galleryState.modalOpen = GalleryModals.Remove;
  }

  function stringToColor(str: string): string {
    // djb2 hash function
    let hash = 5381;

    for (let i = 0; i < str.length; i++) {
      hash = (hash << 5) + hash + str.charCodeAt(i);
      hash |= 0;
    }

    const r = (hash & 0xff0000) >> 16;
    const g = (hash & 0x00ff00) >> 8;
    const b = hash & 0x0000ff;

    const toHex = (n: number) => n.toString(16).padStart(2, "0");

    return `#${toHex(r)}${toHex(g)}${toHex(b)}`;
  }

  onMount(() => {
    const eventSource = new EventSource("/api/rooms/sse");

    eventSource.addEventListener("room-added", (event) => {
      const room = JSON.parse(event.data);
      const roomInfo: RoomInfo = { id: room.room_id, name: room.room_name };
      galleryState.roomList.set(roomInfo.id, roomInfo);
    });

    eventSource.addEventListener("room-updated", (event) => {
      const room = JSON.parse(event.data);
      galleryState.roomList.set(room.room_id, {
        id: room.room_id,
        name: room.room_name,
      });
    });

    eventSource.addEventListener("room-removed", (event) => {
      galleryState.roomList.delete(event.data);
    });

    return () => {
      eventSource.close();
    };
  });
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
      {#each galleryState.roomList as [_, room]}
        <a
          href="/document/{room.id}"
          class="relative group block h-40 rounded-lg border border-gray-700 bg-[#252526] p-4 hover:border-gray-500 hover:bg-[#2d2d2d] transition-colors no-underline"
          onmouseenter={() => (galleryState.hoveredRoomId = room.id)}
          onmouseleave={() => (galleryState.hoveredRoomId = null)}
        >
          <div class="flex h-full flex-col items-center">
            <div class="flex flex-1 items-center justify-center">
              <FileEarmarkTextFill
                width="48"
                height="48"
                class="text-gray-400 group-hover:text-gray-300 transition-colors"
                fill={stringToColor(room.id)}
              />
            </div>
            <span
              title={room.name}
              class="w-full text-center text-xs leading-4 h-12 font-mono break-all text-gray-400 overflow-hidden [mask-image:linear-gradient(to_bottom,black_65%,transparent_100%)]"
            >
              {room.name}
            </span>
          </div>

          {#if galleryState.hoveredRoomId === room.id}
            <button
              class="absolute top-2 left-2 p-1.5 rounded bg-blue-600/80 hover:bg-blue-500 text-white text-xs transition-colors z-10"
              title="Rename screenplay"
              onclick={(e) => {
                e.preventDefault();
                e.stopPropagation();
                openRenameModal();
              }}
              onmouseenter={(e) => e.stopPropagation()}
            >
              <PencilSquare />
            </button>
            <button
              class="absolute top-2 right-2 p-1.5 rounded bg-red-600/80 hover:bg-red-500 text-white text-xs transition-colors z-10"
              title="Delete screenplay"
              onclick={(e) => {
                e.preventDefault();
                e.stopPropagation();
                openDeleteModal();
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

<CreateModal />
<RemoveModal />
<RenameModal />
