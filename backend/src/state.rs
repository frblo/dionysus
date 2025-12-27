use std::{collections::HashMap, sync::Arc};

use tokio::sync::RwLock;
use yrs::sync::Awareness;
use yrs::Doc;
use yrs_axum::AwarenessRef;
use yrs_axum::broadcast::BroadcastGroup;

use crate::rooms;

#[derive(Clone)]
pub struct AppState {
    inner: Arc<AppStateInner>,
}

struct AppStateInner {
    rooms: RwLock<HashMap<String, Arc<BroadcastGroup>>>,
}

impl AppState {
    pub async fn new() -> Self {
        Self {
            inner: Arc::new(AppStateInner {
                rooms: RwLock::new(HashMap::new()),
            }),
        }
    }

    /// Ensure a room exists; if missing, create it.
    pub async fn ensure_room(&self, room_id: &str) -> Arc<BroadcastGroup> {
        // fast path: read lock
        if let Some(room) = self.inner.rooms.read().await.get(room_id).cloned() {
            return room;
        }

        // slow path: write lock + double-check
        let mut rooms_guard = self.inner.rooms.write().await;
        if let Some(room) = rooms_guard.get(room_id).cloned() {
            return room;
        }

        let awareness = make_awareness_for_room(room_id);
        let bcast = Arc::new(BroadcastGroup::new(awareness, 32).await);

        rooms_guard.insert(room_id.to_string(), bcast.clone());
        bcast
    }
}

fn make_awareness_for_room(room_id: &str) -> AwarenessRef {
    // room-specific initial doc content lives in rooms module
    let doc: Doc = rooms::initial_doc(room_id);

    Arc::new(RwLock::new(Awareness::new(doc)))
}
