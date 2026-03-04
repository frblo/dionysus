use std::sync::Arc;

use crate::auth::AuthManager;
use crate::db::Db;
use crate::rooms;
use crate::rooms::RoomManager;

#[derive(Clone)]
pub struct AppState {
    pub db: Db,
    pub auth: AuthManager,
    pub rooms: RoomManager,
}

impl AppState {
    pub async fn new(db: Db, auth: AuthManager) -> Self {
        // let storage = rooms::InMemoryStorage::new().await;
        let storage = rooms::DatabaseStorage::new(db.clone()).await;
        Self {
            db,
            auth,
            rooms: RoomManager::new(Arc::new(storage), 32, 100, 1024),
        }
    }
}
