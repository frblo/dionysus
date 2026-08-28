use crate::rooms::Error;
use crate::rooms::storage::{
    CreateRoomOptions, LoadUpdatesOptions, LogSeq, RoomInfo, Snapshot, SnapshotInfo, Storage,
    UpdateEntry,
};

use std::collections::{BTreeMap, HashMap};

use async_trait::async_trait;

use tokio::sync::RwLock;

use uuid::Uuid;

pub struct InMemoryStorage {
    rooms: RwLock<HashMap<Uuid, RoomData>>,
}

pub struct RoomData {
    info: RoomInfo,
    updates: Vec<(LogSeq, Vec<u8>)>,
    snapshots: BTreeMap<LogSeq, Vec<u8>>,
}

impl InMemoryStorage {
    pub async fn new() -> Self {
        let storage = Self {
            rooms: RwLock::new(HashMap::new()),
        };

        // storage
        //     .create_room(
        //         "demo-room-1",
        //         CreateRoomOptions {
        //             ..Default::default()
        //         },
        //     )
        //     .await
        //     .expect("Initialization calls should work");
        // let _ = storage
        //     .append_update(
        //         "demo-room-1",
        //         &demo_doc()
        //             .transact()
        //             .encode_state_as_update_v1(&yrs::StateVector::default()),
        //     )
        //     .await
        //     .expect("Initialization calls should work");

        storage
    }
}

#[async_trait]
impl Storage for InMemoryStorage {
    async fn room_exists(&self, room_id: Uuid) -> Result<bool, Error> {
        Ok(self.rooms.read().await.contains_key(&room_id))
    }

    async fn load_updates(
        &self,
        room_id: Uuid,
        opts: LoadUpdatesOptions,
    ) -> Result<Vec<UpdateEntry>, Error> {
        let rooms = self.rooms.read().await;
        let room = rooms.get(&room_id).ok_or(Error::NotFound)?;

        let from = opts.from.unwrap_or(0);
        let to = opts.to.unwrap_or(room.info.last_seq);
        Ok(room
            .updates
            .iter()
            .skip_while(|(seq, _)| *seq < from)
            .take_while(|(seq, _)| *seq <= to)
            .map(|(seq, bytes)| UpdateEntry {
                seq: *seq,
                bytes: bytes.clone(),
            })
            .collect())
    }

    async fn create_room(
        &self,
        room_name: &str,
        _opts: CreateRoomOptions,
    ) -> Result<RoomInfo, Error> {
        if room_name.is_empty() {
            return Err(Error::InvalidArgument(
                "Room name cannot be an empty string".to_string(),
            ));
        }

        let mut rooms = self.rooms.write().await;

        let info = RoomInfo {
            room_id: Uuid::new_v4(),
            room_name: room_name.to_string(),
            last_seq: 0,
            latest_snapshot: None,
        };

        rooms.insert(
            info.room_id,
            RoomData {
                info: info.clone(),
                updates: Vec::new(),
                snapshots: BTreeMap::new(),
            },
        );

        Ok(info)
    }

    async fn delete_room(&self, room_id: Uuid) -> Result<(), Error> {
        let mut rooms = self.rooms.write().await;

        rooms.remove(&room_id);

        Ok(())
    }

    async fn rename_room(&self, room_id: Uuid, new_name: &str) -> Result<RoomInfo, Error> {
        let mut rooms = self.rooms.write().await;
        match rooms.get_mut(&room_id) {
            Some(room) => {
                room.info.room_name = new_name.to_string();
                Ok(room.info.clone())
            }
            None => Err(Error::NotFound),
        }
    }

    async fn list_rooms(&self) -> Result<Vec<RoomInfo>, Error> {
        let rooms = self.rooms.write().await;

        Ok(rooms
            .values()
            .map(|room_data| room_data.info.clone())
            .collect())
    }

    async fn get_room_info(&self, room_id: Uuid) -> Result<Option<RoomInfo>, Error> {
        let rooms = self.rooms.write().await;
        Ok(rooms.get(&room_id).map(|x| x.info.clone()))
    }

    async fn append_update(&self, room_id: Uuid, update: &[u8]) -> Result<LogSeq, Error> {
        let mut rooms = self.rooms.write().await;
        let room = rooms.get_mut(&room_id).ok_or(Error::NotFound)?;

        let new_seq = room.info.last_seq + 1;

        room.updates.push((new_seq, update.to_vec()));

        room.info.last_seq = new_seq;

        Ok(new_seq)
    }

    async fn append_updates(
        &self,
        room_id: Uuid,
        updates: &[Vec<u8>],
    ) -> Result<(LogSeq, LogSeq), Error> {
        let first = self.append_update(room_id, &updates[0]).await?;

        let mut last = first;
        for update in &updates[1..] {
            last = self.append_update(room_id, update).await?;
        }

        Ok((first, last))
    }

    async fn store_snapshot(&self, room_id: Uuid, snapshot: Snapshot) -> Result<(), Error> {
        let mut rooms = self.rooms.write().await;
        let room = rooms.get_mut(&room_id).ok_or(Error::NotFound)?;

        if room.snapshots.contains_key(&snapshot.covered_through) {
            return Ok(());
        }

        room.snapshots
            .insert(snapshot.covered_through, snapshot.bytes);

        Ok(())
    }

    async fn load_snapshot_at(
        &self,
        room_id: Uuid,
        covered_through: LogSeq,
    ) -> Result<Option<Snapshot>, Error> {
        let rooms = self.rooms.read().await;
        let room = rooms.get(&room_id).ok_or(Error::NotFound)?;

        Ok(room.snapshots.get(&covered_through).map(|bytes| Snapshot {
            covered_through,
            bytes: bytes.clone(),
        }))
    }

    async fn load_snapshot_best(
        &self,
        room_id: Uuid,
        max_covered_through: Option<LogSeq>,
    ) -> Result<Option<Snapshot>, Error> {
        let rooms = self.rooms.read().await;
        let room = rooms.get(&room_id).ok_or(Error::NotFound)?;

        let snapshot = match max_covered_through {
            None => room.snapshots.iter().next_back(),
            Some(m) => room.snapshots.range(..=m).next_back(),
        };

        Ok(snapshot.map(|(k, v)| Snapshot {
            covered_through: *k,
            bytes: v.clone(),
        }))
    }

    async fn list_snapshots(&self, room_id: Uuid) -> Result<Vec<SnapshotInfo>, Error> {
        let rooms = self.rooms.read().await;
        let room = rooms.get(&room_id).ok_or(Error::NotFound)?;

        Ok(room
            .snapshots
            .iter()
            .map(|(k, v)| SnapshotInfo {
                covered_through: *k,
                size_bytes: v.len().try_into().expect("Should fit in u64"),
            })
            .collect())
    }
}
