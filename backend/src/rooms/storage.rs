use async_trait::async_trait;

use crate::rooms::error::Error;

/// Monotonically increasing, per-room sequence number assigned by storage on append.
/// Defines the canonical ordering of the room's update log.
///
/// The first update has [`LogSeq`] = 1, allowing 0 to represent a document with no updates.
pub type LogSeq = u64;

/// Record for one update entry in the log.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct UpdateEntry {
    pub seq: LogSeq,
    pub bytes: Vec<u8>,
}

/// Snapshot of a room at a specific log position.
///
/// That is applying updates with [`LogSeq`] <= `covered_through` is already reflected in `bytes`.
/// Snapshot with `covered_through` = 0 is thus the same thing as an empty document.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Snapshot {
    pub covered_through: LogSeq,
    pub bytes: Vec<u8>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SnapshotInfo {
    pub covered_through: LogSeq,
    pub size_bytes: u64,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RoomInfo {
    pub room_id: String,

    /// If there are no updates yet, this should be 0.
    pub last_seq: LogSeq,

    pub latest_snapshot: Option<SnapshotInfo>,
}

#[derive(Debug, Clone, Default, PartialEq, Eq)]
#[non_exhaustive]
pub struct CreateRoomOptions {
    pub fail_if_exists: bool,
}

#[derive(Debug, Clone, Default, PartialEq, Eq)]
#[non_exhaustive]
pub struct LoadUpdatesOptions {
    /// Inclusive lower bound. If None, start from the earliest available.
    pub from: Option<LogSeq>,
    /// Inclusive upper bound. If None, read through the latest available.
    pub to: Option<LogSeq>,
}

/// A trait defining the Room Storage interface.
#[async_trait]
pub trait Storage: Send + Sync + 'static {
    /// Check if a room with the given id exists.
    async fn room_exists(&self, room_id: &str) -> Result<bool, Error>;

    /// Creates a room record if absent.
    async fn create_room(&self, room_id: &str, opts: CreateRoomOptions) -> Result<(), Error>;

    /// Removes all data for the room (metadata, updates, snapshot).
    async fn delete_room(&self, room_id: &str) -> Result<(), Error>;

    /// Lists rooms with metadata.
    async fn list_rooms(&self) -> Result<Vec<RoomInfo>, Error>;

    /// Loads metadata for one room. `None` if missing.
    async fn get_room_info(&self, room_id: &str) -> Result<Option<RoomInfo>, Error>;

    /// Appends one update and returns its assigned seq.
    async fn append_update(&self, room_id: &str, update: &[u8]) -> Result<LogSeq, Error>;

    /// Appends multiple updates, preserving order, and returns the assigned seq range (inclusive).
    async fn append_updates(
        &self,
        room_id: &str,
        updates: &[Vec<u8>],
    ) -> Result<(LogSeq, LogSeq), Error>;

    /// Loads updates in the given range as entries
    /// (seq + bytes) for deterministic replay.
    async fn load_updates(
        &self,
        room_id: &str,
        opts: LoadUpdatesOptions,
    ) -> Result<Vec<UpdateEntry>, Error>;

    /// Store a snapshot for the given room.
    async fn store_snapshot(&self, room_id: &str, snapshot: Snapshot) -> Result<(), Error>;

    async fn load_snapshot_at(
        &self,
        room_id: &str,
        covered_through: LogSeq,
    ) -> Result<Option<Snapshot>, Error>;

    /// Load the newest snapshot with `covered_through` <= `max_covered_through`.
    /// If `max_covered_through` is None, returns the latest snapshot overall.
    async fn load_snapshot_best(
        &self,
        room_id: &str,
        max_covered_through: Option<LogSeq>,
    ) -> Result<Option<Snapshot>, Error>;

    /// List available snapshots for the room.
    async fn list_snapshots(&self, room_id: &str) -> Result<Vec<SnapshotInfo>, Error>;
}
