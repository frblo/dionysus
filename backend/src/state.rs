use std::sync::Arc;

use tokio::sync::broadcast;

use crate::auth::AuthManager;
use crate::db::Db;
use crate::rooms;
use crate::rooms::RoomManager;
use crate::rooms::routes::RoomDelta;

#[derive(Clone)]
pub struct AppState {
    pub db: Db,
    pub auth: AuthManager,
    pub rooms: RoomManager,
    pub gallary_tx: broadcast::Sender<RoomDelta>,
}

impl AppState {
    pub async fn new(db: Db, auth: AuthManager) -> Self {
        // let storage = rooms::InMemoryStorage::new().await;
        let storage = rooms::DatabaseStorage::new(db.clone()).await;
        let (tx, _) = broadcast::channel::<RoomDelta>(32);
        Self {
            db,
            auth,
            rooms: RoomManager::new(Arc::new(storage), 32, 100),
            gallary_tx: tx,
        }
    }
}
