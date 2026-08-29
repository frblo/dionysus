import { SvelteMap } from "svelte/reactivity";
import type { RoomInfo } from "../../routes/(app)/+page";

export enum GallaryModals {
  Remove,
  Rename,
  Create,
  None,
}

class GallaryState {
  hoveredRoomId = $state<string | null>(null);
  modalOpen = $state(GallaryModals.None);
  targetedId = $state("");
  roomList = $state<SvelteMap<string, RoomInfo>>(new SvelteMap);
}

export const gallaryState = new GallaryState();
