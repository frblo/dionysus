import type { PageLoad } from "./$types";

export const load: PageLoad = async ({ params, fetch }) => {
    const roomId = params.slug;
    const response = await fetch(`/rooms/api/room_info/${roomId}`);
    const data = await response.json();

    return {
        roomInfo: data
    };
};
