use std::sync::Arc;

use crate::rooms;
use crate::rooms::RoomManager;

#[derive(Clone)]
pub struct AppState {
    pub rooms: RoomManager,
}

impl AppState {
    pub async fn new() -> Self {
        let storage = rooms::InMemoryStorage::new().await;
        Self {
            rooms: RoomManager::new(Arc::new(storage), 32),
        }
    }
}
