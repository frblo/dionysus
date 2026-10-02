use std::sync::Arc;

use crate::auth::{AuthManager, IdentityStore};
use crate::authz::{AuthzManager, HiveRoleSource, PgAuthz};
use crate::config::Hive;
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
    pub async fn new(db: Db, auth: AuthManager, hive: Option<&Hive>) -> anyhow::Result<Self> {
        // let storage = rooms::InMemoryStorage::new().await;
        let storage = rooms::DatabaseStorage::new(db.clone()).await;
        let pg_authz = Arc::new(PgAuthz::new(db.clone()));

        let authz = match hive {
            Some(hive) => {
                let roles =
                    HiveRoleSource::new(hive, IdentityStore::new(db.clone()), auth.events())?;
                tracing::info!(base_url = %hive.base_url, "using Hive as the role source");
                AuthzManager::new(Arc::new(roles), None, pg_authz)
            }
            None => AuthzManager::new(pg_authz.clone(), Some(pg_authz.clone()), pg_authz),
        };

        Ok(Self {
            db,
            auth,
            authz,
            rooms: RoomManager::new(Arc::new(storage), 32, 100),
        })
    }
}
