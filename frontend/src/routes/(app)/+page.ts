import { SvelteMap } from 'svelte/reactivity';
import type { PageLoad } from './$types';
import { galleryState } from '$lib/state/gallery.svelte';

export interface RoomInfo {
  id: string,
  name: string,
}

export const load: PageLoad = async ({ fetch }) => {
  const response = await fetch("/api/rooms/list", {
    method: "GET",
    credentials: "include"
  });
  const json = await response.json();

  let data = new SvelteMap<string, RoomInfo>();
  for (const room of json) {
    const roomInfo = {
      id: room.room_id,
      name: room.room_name
    };
    data.set(roomInfo.id, roomInfo);
  }

  galleryState.roomList = data;
};
