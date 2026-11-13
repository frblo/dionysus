import type { UpdateInfo } from "$lib/api/generated/UpdateInfo";

type Fetch = typeof globalThis.fetch;

export type VersionEntry = {
  seq: number;
  createdAt: string;
};

export async function listVersions(
  fetcher: Fetch,
  roomId: string,
): Promise<VersionEntry[]> {
  const response = await fetcher(`/api/rooms/list_updates/${roomId}`);
  if (!response.ok) throw new Error("Failed to fetch document history");

  const updates: UpdateInfo[] = await response.json();

  return updates
    .map(({ covered_through, created_at }) => ({
      seq: Number(covered_through),
      createdAt: created_at,
    }))
    .sort((a, b) => b.seq - a.seq);
}

export async function getVersion(
  fetcher: Fetch,
  roomId: string,
  seq: number,
): Promise<string> {
  const response = await fetcher(
    `/api/rooms/version_at/${roomId}?covered_through=${seq}`,
  );
  if (!response.ok) throw new Error("Failed to fetch document version");

  return response.json();
}
