import { SvelteMap } from 'svelte/reactivity';
import type { PageLoad } from './$types';

export interface RoomInfo {
  id: string,
  name: string,
}

export const load: PageLoad = async ({ fetch }) => {
  const response = await fetch("/rooms/api/list", {
    method: "GET",
    credentials: "include"
  });
  const json = await response.json();
  // const bogus = [
  //   {
  //     room_id: "id_lolboll",
  //     room_name: "Bunguskons seger",
  //   }
  // ]

  let data = new SvelteMap<string, RoomInfo>();
  for (const room of json) {
    const roomInfo = {
      id: room.room_id,
      name: room.room_name
    };
    data.set(roomInfo.id, roomInfo);
  }

  return {
    roomList: data
  };
};
