import { SvelteMap } from "svelte/reactivity";

export interface RoomInfo {
  id: string,
  name: string,
}

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
  for (const room of json) {
    data.set(room.room_id, {
      id: room.room_id,
      name: room.room_name,
    });
  }

  galleryState.roomList = data;
}
