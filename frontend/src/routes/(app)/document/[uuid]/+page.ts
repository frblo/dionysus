import type { PageLoad } from "./$types";

export const load: PageLoad = async ({ params, fetch }) => {
    const roomId = params.uuid;
    const response = await fetch(`/api/rooms/room_info/${roomId}`);
    if (!response.ok) throw new Error("Failed to fetch room info");
    const data = await response.json();

    return {
        roomInfo: data
    };
};
