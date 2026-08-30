export async function createRoom(name: String): Promise<String> {
  const response = await fetch(`/api/rooms/create/${name}`, {
    method: "POST",
    credentials: "include"
  });
  const roomId: string = await response.json();
  return roomId;
}

export async function renameRoom(id: String, name: String) {
  await fetch(`/api/rooms/rename/${id}/${name}`, {
    method: "POST",
    credentials: "include"
  });
}

export async function deleteRoom(id: String) {
  await fetch(`/api/rooms/delete/${id}`, {
    method: "POST",
    credentials: "include"
  });
}
