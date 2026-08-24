use std::sync::atomic::{AtomicU64, AtomicUsize, Ordering};
use std::{collections::HashMap, sync::Arc};

use tokio::sync::{RwLock, mpsc, watch};
use tracing::Instrument;
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
    enqueued: Arc<AtomicU64>,
    persisted: watch::Receiver<u64>,
}

impl LiveRoom {
    fn inc(&self) {
        self.conn_count.fetch_add(1, Ordering::Relaxed);
    }

    fn dec(&self) -> usize {
        self.conn_count.fetch_sub(1, Ordering::Relaxed) - 1
    }

    /// Waits until the persist task has caught up processing everything
    /// enqueued as of this call.
    async fn wait_for_drain(&self) {
        let target = self.enqueued.load(Ordering::Relaxed);
        let mut persisted = self.persisted.clone();
        while *persisted.borrow() < target {
            if persisted.changed().await.is_err() {
                // Persist task ended, which means there is nothing more
                // to wait for.
                break;
            }
        }
    }
}

#[derive(Clone)]
pub struct RoomManager {
    storage: Arc<dyn Storage>,
    live: Arc<RwLock<HashMap<String, Arc<LiveRoom>>>>,
    bcast_capacity: usize,
    snapshot_every_n_updates: u64,
}

impl RoomManager {
    pub fn new(
        storage: Arc<dyn Storage>,
        bcast_capacity: usize,
        snapshot_every_n_updates: u64,
    ) -> Self {
        Self {
            storage,
            live: Arc::new(RwLock::new(HashMap::new())),
            bcast_capacity,
            snapshot_every_n_updates,
        }
    }

    /// Aquire the [`LiveRoom`] for the `room_id` or attempt to cretate it
    /// if it doesn't already exist.
    #[tracing::instrument(skip_all, fields(room_id = %room_id))]
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
    #[tracing::instrument(skip_all, fields(room_id = %room_id))]
    pub async fn disconnect(&self, room_id: &str) {
        let Some(room) = self.live.read().await.get(room_id).cloned() else {
            return;
        };

        if room.dec() == 0 {
            room.wait_for_drain().await;

            let mut guard = self.live.write().await;

            // Re-check so no one else has changed it (e.g. a reconnect
            // during the drain wait above, or another eviction).
            if let Some(current) = guard.get(room_id)
                && Arc::ptr_eq(current, &room)
                && current.conn_count.load(Ordering::Relaxed) == 0
            {
                tracing::info!("evicting room from memory (no active connections)");
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

    /// Lists all rooms with metadata.
    pub async fn list_rooms(&self) -> Result<Vec<storage::RoomInfo>, Error> {
        self.storage.list_rooms().await
    }

    /// Gets room info
    pub async fn room_info(&self, room_id: &str) -> Result<Option<storage::RoomInfo>, Error> {
        self.storage.get_room_info(room_id).await
    }

    /// Gets the [`LiveRoom`] for the room if it exists in memory
    async fn get_live(&self, room_id: &str) -> Option<Arc<LiveRoom>> {
        self.live.read().await.get(room_id).cloned()
    }

    /// Creates a [`LiveRoom`] for the room, if it already exists return the existing
    /// [`LiveRoom`]
    #[tracing::instrument(skip_all, fields(room_id = %room_id))]
    async fn create_room_live(&self, room_id: &str) -> Result<Arc<LiveRoom>, Error> {
        let mut guard = self.live.write().await;
        if let Some(r) = guard.get(room_id).cloned() {
            return Ok(r);
        }

        let (awareness, sub, enqueued, persisted) =
            self.make_awareness_and_persitence(room_id).await?;
        let bcast = Arc::new(BroadcastGroup::new(awareness.clone(), self.bcast_capacity).await);

        let room = Arc::new(LiveRoom {
            bcast,
            awareness,
            // Needs to be stored. Unsubsribes when dropped.
            _sub: sub,
            conn_count: AtomicUsize::new(0),
            enqueued,
            persisted,
        });

        guard.insert(room_id.to_string(), room.clone());
        Ok(room)
    }

    async fn make_awareness_and_persitence(
        &self,
        room_id: &str,
    ) -> Result<
        (
            AwarenessRef,
            Subscription,
            Arc<AtomicU64>,
            watch::Receiver<u64>,
        ),
        Error,
    > {
        let doc = self.load_doc(room_id).await?;

        let (tx, mut rx) = mpsc::unbounded_channel::<Vec<u8>>();
        let storage = self.storage.clone();
        let room_id_owned = room_id.to_string();

        let doc_for_snapshots = doc.clone();
        let snapshot_every = self.snapshot_every_n_updates;

        let (persisted_tx, persisted_rx) = watch::channel(0u64);

        // The persistence task outlives the `create_room_live` span that spawns
        // it, so give it its own root span rather than letting it inherit.
        let persist_span = tracing::info_span!("room_persistence_task", room_id = %room_id_owned);

        tokio::spawn(
            async move {
                let mut since_snapshot = 0;
                let mut last_seq;
                let mut processed: u64 = 0;

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
                                match storage.store_snapshot(&room_id_owned, snap).await {
                                    Ok(()) => {
                                        since_snapshot = 0;
                                        tracing::debug!(
                                            covered_through = last_seq,
                                            "snapshot stored"
                                        );
                                    }
                                    Err(e) => {
                                        tracing::warn!(
                                            error = %e,
                                            "failed to store snapshot, will retry on next threshold"
                                        );
                                    }
                                }
                            }
                        }
                        Err(e) => {
                            // Attempt to store change. On error just log and continue in-memory doc.
                            tracing::error!(
                                error = %e,
                                "failed to persist update to storage; in-memory doc continues but change may be lost on restart"
                            );
                        }
                    }

                    // Mark as processed even on error: the error is already logged above and
                    // there's no retry, so this is what "drained" means for a room eviction wait.
                    processed += 1;
                    let _ = persisted_tx.send(processed);
                }
            }
            .instrument(persist_span),
        );

        let enqueued = Arc::new(AtomicU64::new(0));
        let enqueued_for_sub = enqueued.clone();
        let room_id_for_sub = room_id.to_string();
        let sub = doc
            .observe_update_v1(move |_txn, e| match tx.send(e.update.clone()) {
                Ok(()) => {
                    enqueued_for_sub.fetch_add(1, Ordering::Relaxed);
                }
                Err(e) => {
                    tracing::error!(
                        room_id = %room_id_for_sub,
                        error = %e,
                        "persist task is gone, update dropped"
                    );
                }
            })
            .expect("Subscription function should work.");

        Ok((
            Arc::new(RwLock::new(Awareness::new(doc))),
            sub,
            enqueued,
            persisted_rx,
        ))
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
