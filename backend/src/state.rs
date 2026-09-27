use std::sync::Arc;

use crate::auth::AuthManager;
use crate::authz::{AuthzManager, PgAuthz};
use crate::db::Db;
use crate::rooms;
use crate::rooms::RoomManager;

#[derive(Clone)]
pub struct AppState {
    pub db: Db,
    pub auth: AuthManager,
    pub authz: AuthzManager,
    pub rooms: RoomManager,
}

impl AppState {
    pub async fn new(db: Db, auth: AuthManager) -> Self {
        // let storage = rooms::InMemoryStorage::new().await;
        let storage = rooms::DatabaseStorage::new(db.clone()).await;
        let pg_authz = Arc::new(PgAuthz::new(db.clone()));
        Self {
            db,
            auth,
            authz: AuthzManager::new(pg_authz.clone(), Some(pg_authz.clone()), pg_authz),
            rooms: RoomManager::new(Arc::new(storage), 32, 100),
        }
    }
}
