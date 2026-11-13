import type { PageLoad } from "./$types";
import type { RoomInfo } from "$lib/api/generated/RoomInfo";
import { getVersion, listVersions } from "$lib/history/api";

export const load: PageLoad = async ({ params, fetch }) => {
  const roomId = params.uuid;

  const [roomInfoResponse, versions] = await Promise.all([
    fetch(`/api/rooms/room_info/${roomId}`),
    listVersions(fetch, roomId),
  ]);
  if (!roomInfoResponse.ok) throw new Error("Failed to fetch room info");

  const roomInfo: RoomInfo = await roomInfoResponse.json();
  const latest = versions[0] ?? null;
  const initialVersion = latest
    ? await getVersion(fetch, roomId, latest.seq)
    : "";

  return {
    roomInfo,
    versions,
    initialSeq: latest?.seq ?? null,
    initialVersion,
  };
};
