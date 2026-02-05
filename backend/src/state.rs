use std::sync::Arc;

use crate::auth::SessionStore;
use crate::db::Db;
use crate::rooms;
use crate::rooms::RoomManager;

#[derive(Clone)]
pub struct AppState {
    pub db: Db,
    pub sessions: SessionStore,
    pub rooms: RoomManager,
}

impl AppState {
    pub async fn new(db: Db) -> Self {
        // let storage = rooms::InMemoryStorage::new().await;
        let storage = rooms::DatabaseStorage::new(db.clone()).await;
        Self {
            db,
            sessions: SessionStore::new(),
            rooms: RoomManager::new(Arc::new(storage), 32, 100, 1024),
        }
    }
}
