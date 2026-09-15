import { SvelteMap } from "svelte/reactivity";

import type { RoomInfo } from "$lib/api/generated/RoomInfo";

export enum GalleryModals {
  Remove,
  Rename,
  Create,
  None,
}

class GalleryState {
  hoveredRoomId = $state<string | null>(null);
  modalOpen = $state(GalleryModals.None);
  targetedId = $state("");
  roomList = $state<SvelteMap<string, RoomInfo>>(new SvelteMap);
}

export const galleryState = new GalleryState();

export async function loadRoomList(
  fetcher: (url: string, init?: RequestInit) => Promise<Response> = fetch
): Promise<void> {
  const response = await fetcher("/api/rooms/list", {
    method: "GET",
    credentials: "include"
  });
  if (!response.ok) throw new Error("Failed to fetch rooms");
  const json = await response.json();

  const data = new SvelteMap<string, RoomInfo>();
  for (const room of json as RoomInfo[]) {
    data.set(room.room_id, room);
  }

  galleryState.roomList = data;
}
