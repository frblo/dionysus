import { SvelteMap } from "svelte/reactivity";
import type { RoomInfo } from "../../routes/(app)/+page";

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
