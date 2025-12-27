use crate::rooms::error::StorageError;
use async_trait::async_trait;

#[async_trait]
pub trait Storage: Send + Sync + 'static {
    async fn room_exists(&self, room_id: &str) -> Result<bool, StorageError>;
    async fn load_updates(&self, room_id: &str) -> Result<Option<Vec<Vec<u8>>>, StorageError>;
    // async fn create_room(&self, room_id: &str) -> Result<(), StorageError>;
}
