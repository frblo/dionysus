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
  const previous = versions[1] ?? null;
  const [initialVersion, initialPreviousVersion] = await Promise.all([
    latest ? getVersion(fetch, roomId, latest.seq) : Promise.resolve(""),
    previous ? getVersion(fetch, roomId, previous.seq) : Promise.resolve(null),
  ]);

  return {
    roomInfo,
    versions,
    initialSeq: latest?.seq ?? null,
    initialVersion,
    initialPreviousSeq: previous?.seq ?? null,
    initialPreviousVersion,
  };
};
