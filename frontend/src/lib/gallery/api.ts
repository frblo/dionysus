export async function createRoom(name: String): Promise<String> {
  const response = await fetch(`/api/rooms/create/${name}`, {
    method: "POST",
    credentials: "include"
  });
  if (!response.ok) throw new Error("Failed to create room");
  const roomId: string = await response.json();
  return roomId;
}

export async function renameRoom(id: String, name: String) {
  const response = await fetch(`/api/rooms/rename/${id}`, {
    method: "POST",
    credentials: "include",
    headers: { "Content-Type": "application/json" },
    body: JSON.stringify({ name })
  });
  if (!response.ok) throw new Error("Failed to rename room");
}

export async function deleteRoom(id: String) {
  const response = await fetch(`/api/rooms/delete/${id}`, {
    method: "DELETE",
    credentials: "include"
  });
  if (!response.ok) throw new Error("Failed to delete room");
}
