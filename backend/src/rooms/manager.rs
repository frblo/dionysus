use std::{collections::HashMap, sync::Arc};

use tokio::sync::RwLock;
use yrs::sync::Awareness;
use yrs::updates::decoder::Decode;
use yrs::{Doc, Transact};
use yrs_axum::{AwarenessRef, broadcast::BroadcastGroup};

use crate::rooms::error::Error;
use crate::rooms::storage::{self, Storage};

#[derive(Clone)]
pub struct RoomManager {
    storage: Arc<dyn Storage>,
    live: Arc<RwLock<HashMap<String, Arc<BroadcastGroup>>>>,
    bcast_capacity: usize,
}

impl RoomManager {
    pub fn new(storage: Arc<dyn Storage>, bcast_capacity: usize) -> Self {
        Self {
            storage,
            live: Arc::new(RwLock::new(HashMap::new())),
            bcast_capacity,
        }
    }

    // TODO: Better name, or at least consider naming.
    pub async fn connect(&self, room_id: &str) -> Result<Arc<BroadcastGroup>, Error> {
        // Check if it exists live
        if let Some(g) = self.get_live(room_id).await {
            return Ok(g);
        }

        // Attempt to create group
        self.create_group(room_id).await
    }

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

    /// Gets the [`BroadcastGroup`] for the room if it exists in memory
    async fn get_live(&self, room_id: &str) -> Option<Arc<BroadcastGroup>> {
        self.live.read().await.get(room_id).cloned()
    }

    /// Creates a [`BroadcastGroup`] for the room, if it already exists return the existing
    /// [`BroadcastGroup`]
    async fn create_group(&self, room_id: &str) -> Result<Arc<BroadcastGroup>, Error> {
        let mut guard = self.live.write().await;
        if let Some(g) = guard.get(room_id).cloned() {
            return Ok(g);
        }

        let awareness = self.make_awareness(room_id).await?;
        let bcast = Arc::new(BroadcastGroup::new(awareness, self.bcast_capacity).await);

        guard.insert(room_id.to_string(), bcast.clone());
        Ok(bcast)
    }

    async fn make_awareness(&self, room_id: &str) -> Result<AwarenessRef, Error> {
        let updates = self
            .storage
            .load_updates(
                room_id,
                storage::LoadUpdatesOptions {
                    ..Default::default()
                },
            )
            .await?;

        let doc = Doc::new();
        {
            let mut txn = doc.transact_mut();
            for u in updates {
                let update = yrs::Update::decode_v1(&u.bytes)?;
                txn.apply_update(update);
            }
        }

        Ok(Arc::new(RwLock::new(Awareness::new(doc))))
    }
}
