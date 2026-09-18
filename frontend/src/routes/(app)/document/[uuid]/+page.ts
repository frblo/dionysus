import type { PageLoad } from "./$types";
import type { RoomInfo } from "$lib/api/generated/RoomInfo";

export const load: PageLoad = async ({ params, fetch }) => {
    const roomId = params.uuid;
    const response = await fetch(`/api/rooms/room_info/${roomId}`);
    if (!response.ok) throw new Error("Failed to fetch room info");
    const data: RoomInfo = await response.json();

    return {
        roomInfo: data
    };
};
