pub mod error;
pub mod manager;
mod storage;

use async_trait::async_trait;
pub use error::Error;
pub use manager::RoomManager;

use std::{collections::HashMap, sync::Arc};

use tokio::sync::RwLock;
use yrs::{Doc, ReadTxn, Text, Transact};

use crate::rooms::{error::StorageError, storage::Storage};

pub struct InMemoryStorage {
    storage: Arc<RwLock<HashMap<String, Vec<Vec<u8>>>>>,
}

impl InMemoryStorage {
    pub fn new() -> Self {
        let mut hm = HashMap::new();
        hm.insert(
            "demo-room-1".to_string(),
            vec![
                demo_doc()
                    .transact()
                    .encode_state_as_update_v1(&yrs::StateVector::default()),
            ],
        );

        Self {
            storage: Arc::new(RwLock::new(hm)),
        }
    }
}

#[async_trait]
impl Storage for InMemoryStorage {
    async fn room_exists(&self, room_id: &str) -> Result<bool, StorageError> {
        Ok(self.storage.read().await.contains_key(room_id))
    }

    async fn load_updates(&self, room_id: &str) -> Result<Option<Vec<Vec<u8>>>, StorageError> {
        let res = self.storage.read().await.get(room_id).cloned();
        Ok(res)
    }
}

fn demo_doc() -> Doc {
    let doc = Doc::new();
    {
        let txt = doc.get_or_insert_text("codemirror");
        let mut txn = doc.transact_mut();
        txt.push(
            &mut txn,
            r#"EXT. BRICK'S PATIO - DAY

A gorgeous day.  The sun is shining.  But BRICK BRADDOCK, retired police detective, is sitting quietly, contemplating -- something.

The SCREEN DOOR slides open and DICK STEEL, his former partner and fellow retiree, emerges with two cold beers.

STEEL
Beer's ready!

BRICK
Are they cold?"#,
        );
    }
    doc
}
