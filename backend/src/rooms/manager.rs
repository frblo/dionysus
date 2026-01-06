use std::sync::atomic::AtomicUsize;
use std::{collections::HashMap, sync::Arc};

use tokio::sync::{RwLock, mpsc};
use yrs::sync::Awareness;
use yrs::updates::decoder::Decode;
use yrs::{Doc, ReadTxn, Subscription, Transact};
use yrs_axum::{AwarenessRef, broadcast::BroadcastGroup};

use crate::rooms::error::Error;
use crate::rooms::storage::{self, LoadUpdatesOptions, Storage};

pub struct LiveRoom {
    pub bcast: Arc<BroadcastGroup>,
    pub awareness: AwarenessRef,
    _sub: Subscription,
    conn_count: AtomicUsize,
}

impl LiveRoom {
    fn inc(&self) {
        self.conn_count
            .fetch_add(1, std::sync::atomic::Ordering::Relaxed);
    }

    fn dec(&self) -> usize {
        self.conn_count
            .fetch_sub(1, std::sync::atomic::Ordering::Relaxed)
            - 1
    }
}

#[derive(Clone)]
pub struct RoomManager {
    storage: Arc<dyn Storage>,
    live: Arc<RwLock<HashMap<String, Arc<LiveRoom>>>>,
    bcast_capacity: usize,
    snapshot_every_n_updates: u64,
    persist_queue_capacity: usize,
}

impl RoomManager {
    pub fn new(
        storage: Arc<dyn Storage>,
        bcast_capacity: usize,
        snapshot_every_n_updates: u64,
        persist_queue_capacity: usize,
    ) -> Self {
        Self {
            storage,
            live: Arc::new(RwLock::new(HashMap::new())),
            bcast_capacity,
            snapshot_every_n_updates,
            persist_queue_capacity,
        }
    }

    /// Aquire the [`LiveRoom`] for the `room_id` or attempt to cretate it
    /// if it doesn't already exist.
    pub async fn connect(&self, room_id: &str) -> Result<Arc<LiveRoom>, Error> {
        // Check if it exists live
        let r = if let Some(r) = self.get_live(room_id).await {
            r
        } else {
            // Attempt to create group
            self.create_room_live(room_id).await?
        };

        r.inc();
        Ok(r)
    }

    /// Release one connection. If it's the last we evict the room from memory
    pub async fn disconnect(&self, room_id: &str) {
        let Some(room) = self.live.read().await.get(room_id).cloned() else {
            return;
        };

        if room.dec() == 0 {
            let mut guard = self.live.write().await;

            // Re-check so no one else has changed it.
            if let Some(current) = guard.get(room_id)
                && current
                    .conn_count
                    .load(std::sync::atomic::Ordering::Relaxed)
                    == 0
            {
                println!("Evicting room {room_id}");
                guard.remove(room_id);
            }
        }
    }

    /// Create a new room in the storage so a [`LiveRoom`] can be created later.
    pub async fn create_room(&self, room_id: &str) -> Result<(), Error> {
        let exists = self.storage.room_exists(room_id).await?;
        if exists {
            return Err(Error::AlreadyExists);
        }
        self.storage
            .create_room(
                room_id,
                storage::CreateRoomOptions {
                    ..Default::default()
                },
            )
            .await?;
        Ok(())
    }

    /// Gets the [`LiveRoom`] for the room if it exists in memory
    async fn get_live(&self, room_id: &str) -> Option<Arc<LiveRoom>> {
        self.live.read().await.get(room_id).cloned()
    }

    /// Creates a [`LiveRoom`] for the room, if it already exists return the existing
    /// [`LiveRoom`]
    async fn create_room_live(&self, room_id: &str) -> Result<Arc<LiveRoom>, Error> {
        let mut guard = self.live.write().await;
        if let Some(r) = guard.get(room_id).cloned() {
            return Ok(r);
        }

        let (awareness, sub) = self.make_awareness_and_persitence(room_id).await?;
        let bcast = Arc::new(BroadcastGroup::new(awareness.clone(), self.bcast_capacity).await);

        let room = Arc::new(LiveRoom {
            bcast,
            awareness,
            // Needs to be stored. Unsubsribes when dropped.
            _sub: sub,
            conn_count: AtomicUsize::new(0),
        });

        guard.insert(room_id.to_string(), room.clone());
        Ok(room)
    }

    async fn make_awareness_and_persitence(
        &self,
        room_id: &str,
    ) -> Result<(AwarenessRef, Subscription), Error> {
        let doc = self.load_doc(room_id).await?;

        let (tx, mut rx) = mpsc::channel::<Vec<u8>>(self.persist_queue_capacity);
        let storage = self.storage.clone();
        let room_id_owned = room_id.to_string();

        let doc_for_snapshots = doc.clone();
        let snapshot_every = self.snapshot_every_n_updates;

        tokio::spawn(async move {
            let mut since_snapshot = 0;
            let mut last_seq = 0;

            while let Some(update_bytes) = rx.recv().await {
                match storage.append_update(&room_id_owned, &update_bytes).await {
                    Ok(seq) => {
                        last_seq = seq;
                        since_snapshot += 1;

                        if since_snapshot >= snapshot_every {
                            // Encode full doc state as an update (v1) and store snapshot.
                            let bytes = doc_for_snapshots
                                .transact()
                                .encode_state_as_update_v1(&yrs::StateVector::default());

                            let snap = storage::Snapshot {
                                covered_through: last_seq,
                                bytes,
                            };

                            // Attempt to store snapshot. On error just log and continue.
                            if let Err(e) = storage.store_snapshot(&room_id_owned, snap).await {
                                eprintln!("snapshot failed room={room_id_owned}: {e:?}");
                            } else {
                                since_snapshot = 0;
                            }
                        }
                    }
                    Err(e) => {
                        // Attempt to store change. On error just log and continue in-memory doc.
                        eprintln!("append_update failed room={room_id_owned}: {e:?}");
                    }
                }
            }
        });

        let sub = doc
            .observe_update_v1(move |_txn, e| {
                // Attempt to send if full we drop.
                let _ = tx.try_send(e.update.clone());
            })
            .expect("Subscription function should work.");

        Ok((Arc::new(RwLock::new(Awareness::new(doc))), sub))
    }

    async fn load_doc(&self, room_id: &str) -> Result<Doc, Error> {
        let doc = Doc::new();
        let snap = self.storage.load_snapshot_best(room_id, None).await?;

        let start_from = if let Some(s) = snap {
            // Apply snapshot
            let mut txn = doc.transact_mut();
            let update = yrs::Update::decode_v1(&s.bytes)?;
            txn.apply_update(update);

            s.covered_through + 1
        } else {
            1
        };

        let updates = self
            .storage
            .load_updates(
                room_id,
                LoadUpdatesOptions {
                    from: Some(start_from),
                    ..Default::default()
                },
            )
            .await?;

        {
            let mut txn = doc.transact_mut();
            for u in updates {
                let update = yrs::Update::decode_v1(&u.bytes)?;
                txn.apply_update(update);
            }
        }

        Ok(doc)
    }
}
