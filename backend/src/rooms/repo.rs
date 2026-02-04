use async_trait::async_trait;
use sqlx::{Executor, Postgres, Transaction};

use crate::db::Db;
use crate::rooms::error::Error;
use crate::rooms::storage::{
    CreateRoomOptions, LoadUpdatesOptions, LogSeq, RoomInfo, Snapshot, SnapshotInfo, Storage,
    UpdateEntry,
};

impl From<sqlx::Error> for Error {
    fn from(value: sqlx::Error) -> Self {
        Error::Backend {
            source: Box::new(value),
        }
    }
}

#[derive(Clone)]
pub struct DatabaseStorage {
    db: Db,
}

impl DatabaseStorage {
    pub async fn new(db: Db) -> Self {
        let storage = Self { db };
        storage
            .create_room(
                "demo-room-1",
                CreateRoomOptions {
                    ..Default::default()
                },
            )
            .await
            .expect("Initialization calls should work");

        storage
    }

    async fn alloc_seq_range<'e, E>(ex: E, room_id: &str, n: i64) -> Result<(i64, i64), Error>
    where
        E: Executor<'e, Database = Postgres>,
    {
        // Should always allocate one or more updates
        debug_assert!(n > 0);
        // Allocate contiguous [first..last] by incrementing rooms.last_seq.
        // If previous last_seq was L, after +n it becomes L+n:
        // first = (L+n) - (n-1) = last_seq - n + 1, last = last_seq
        let row = sqlx::query!(
            r#"
            UPDATE
                rooms
            SET
                last_seq = last_seq + $2
            WHERE
                room_id = $1
            RETURNING (last_seq - $2 + 1) AS "first_seq!",
            last_seq AS "last_seq!""#,
            room_id,
            n
        )
        .fetch_optional(ex)
        .await
        .map_err(Error::from)?;

        let Some(r) = row else {
            return Err(Error::NotFound);
        };

        Ok((r.first_seq, r.last_seq))
    }
}

#[async_trait]
impl Storage for DatabaseStorage {
    async fn room_exists(&self, room_id: &str) -> Result<bool, Error> {
        let r = sqlx::query!(
            r#"SELECT
                   EXISTS (
                       SELECT
                           1
                       FROM
                           rooms
                       WHERE
                           room_id = $1) AS "exists!""#,
            room_id
        )
        .fetch_one(self.db.pool())
        .await
        .map_err(Error::from)?;

        Ok(r.exists)
    }

    async fn create_room(&self, room_id: &str, opts: CreateRoomOptions) -> Result<(), Error> {
        let res = sqlx::query!(
            r#"
            INSERT INTO rooms (room_id, last_seq)
                VALUES ($1, 0)
            ON CONFLICT (room_id)
                DO NOTHING"#,
            room_id
        )
        .execute(self.db.pool())
        .await
        .map_err(Error::from)?;

        if opts.fail_if_exists && res.rows_affected() == 0 {
            return Err(Error::AlreadyExists);
        }
        Ok(())
    }

    async fn delete_room(&self, room_id: &str) -> Result<(), Error> {
        // Assumes FK ON DELETE CASCADE to updates/snapshots.
        sqlx::query!(r#"DELETE FROM rooms
                        WHERE room_id = $1"#, room_id)
            .execute(self.db.pool())
            .await
            .map_err(Error::from)?;
        Ok(())
    }

    async fn list_rooms(&self) -> Result<Vec<RoomInfo>, Error> {
        let rows = sqlx::query!(
            r#"
            SELECT
                r.room_id,
                r.last_seq,
                s.covered_through AS snap_covered,
                s.size_bytes AS snap_size
            FROM
                rooms r
                LEFT JOIN LATERAL (
                    SELECT
                        covered_through,
                        octet_length(bytes)::bigint AS size_bytes
                    FROM
                        room_snapshots
                    WHERE
                        room_id = r.room_id
                    ORDER BY
                        covered_through DESC
                    LIMIT 1) s ON TRUE
            ORDER BY
                r.room_id ASC"#
        )
        .fetch_all(self.db.pool())
        .await
        .map_err(Error::from)?;

        Ok(rows
            .into_iter()
            .map(|r| RoomInfo {
                room_id: r.room_id,
                last_seq: r.last_seq as u64,
                latest_snapshot: r.snap_covered.map(|ct| SnapshotInfo {
                    covered_through: ct as u64,
                    size_bytes: r.snap_size.unwrap_or(0) as u64,
                }),
            })
            .collect())
    }

    async fn get_room_info(&self, room_id: &str) -> Result<Option<RoomInfo>, Error> {
        let row = sqlx::query!(
            r#"
            SELECT
                r.room_id,
                r.last_seq,
                s.covered_through AS "snap_covered?",
                s.size_bytes AS snap_size
            FROM
                rooms r
                LEFT JOIN LATERAL (
                    SELECT
                        covered_through,
                        octet_length(bytes)::bigint AS size_bytes
                    FROM
                        room_snapshots
                    WHERE
                        room_id = r.room_id
                    ORDER BY
                        covered_through DESC
                    LIMIT 1) s ON TRUE
            WHERE
                r.room_id = $1"#,
            room_id
        )
        .fetch_optional(self.db.pool())
        .await
        .map_err(Error::from)?;

        Ok(row.map(|r| RoomInfo {
            room_id: r.room_id,
            last_seq: r.last_seq as u64,
            latest_snapshot: r.snap_covered.map(|ct| SnapshotInfo {
                covered_through: ct as u64,
                size_bytes: r.snap_size.unwrap_or(0) as u64,
            }),
        }))
    }

    async fn append_update(&self, room_id: &str, update: &[u8]) -> Result<LogSeq, Error> {
        let mut tx: Transaction<'_, Postgres> =
            self.db.pool().begin().await.map_err(Error::from)?;

        let (first, last) = DatabaseStorage::alloc_seq_range(&mut *tx, room_id, 1).await?;
        debug_assert_eq!(first, last);

        sqlx::query!(
            r#"
            INSERT INTO room_updates (room_id, seq, bytes)
                VALUES ($1, $2, $3)"#,
            room_id,
            first,
            update
        )
        .execute(&mut *tx)
        .await
        .map_err(Error::from)?;

        tx.commit().await.map_err(Error::from)?;
        Ok(first as u64)
    }

    async fn append_updates(
        &self,
        room_id: &str,
        updates: &[Vec<u8>],
    ) -> Result<(LogSeq, LogSeq), Error> {
        if updates.is_empty() {
            return Err(Error::InvalidArgument(
                "updates must be non-empty".to_string(),
            ));
        }
        let n: i64 = updates
            .len()
            .try_into()
            .map_err(|_| Error::InvalidArgument("too many updates".to_string()))?;

        let mut tx: Transaction<'_, Postgres> =
            self.db.pool().begin().await.map_err(Error::from)?;

        let (first, last) = DatabaseStorage::alloc_seq_range(&mut *tx, room_id, n).await?;

        // Insert all updates; preserve caller order.
        sqlx::query!(
            r#"
            INSERT INTO room_updates (room_id, seq, bytes)
            SELECT
                $1 AS room_id,
                ($2 + gs.i) AS seq,
                u.bytes AS bytes
            FROM
                generate_series(0::bigint, $3 - 1::bigint) AS gs (i)
                JOIN unnest($4::bytea[])
                WITH ORDINALITY AS u (bytes, ord) ON u.ord = gs.i + 1
            ORDER BY
                gs.i"#,
            room_id,
            first,
            n,
            updates as &[Vec<u8>]
        )
        .execute(&mut *tx)
        .await
        .map_err(Error::from)?;

        tx.commit().await.map_err(Error::from)?;
        Ok((first as u64, last as u64))
    }

    async fn load_updates(
        &self,
        room_id: &str,
        opts: LoadUpdatesOptions,
    ) -> Result<Vec<UpdateEntry>, Error> {
        let from: i64 = opts.from.map(|v| v as i64).unwrap_or(1);
        let to: Option<i64> = opts.to.map(|v| v as i64);

        let rows = sqlx::query!(
            r#"
                SELECT
                    seq,
                    bytes
                FROM
                    room_updates
                WHERE
                    room_id = $1
                    AND seq >= $2
                    AND ($3::bigint IS NULL
                        OR seq <= $3)
                ORDER BY
                    seq ASC"#,
            room_id,
            from,
            to
        )
        .fetch_all(self.db.pool())
        .await
        .map_err(Error::from)?;

        Ok(rows
            .into_iter()
            .map(|r| UpdateEntry {
                seq: r.seq as u64,
                bytes: r.bytes,
            })
            .collect())
    }

    async fn store_snapshot(&self, room_id: &str, snapshot: Snapshot) -> Result<(), Error> {
        // Upsert snapshot by (room_id, covered_through).
        // You may want to enforce snapshot.covered_through <= rooms.last_seq in a later iteration.
        sqlx::query!(
            r#"
            INSERT INTO room_snapshots (room_id, covered_through, bytes)
                VALUES ($1, $2, $3)
            ON CONFLICT (room_id, covered_through)
                DO UPDATE SET
                    bytes = excluded.bytes"#,
            room_id,
            snapshot.covered_through as i64,
            snapshot.bytes
        )
        .execute(self.db.pool())
        .await
        .map_err(Error::from)?;

        Ok(())
    }

    async fn load_snapshot_at(
        &self,
        room_id: &str,
        covered_through: LogSeq,
    ) -> Result<Option<Snapshot>, Error> {
        let row = sqlx::query!(
            r#"
            SELECT
                covered_through,
                bytes
            FROM
                room_snapshots
            WHERE
                room_id = $1
                AND covered_through = $2"#,
            room_id,
            covered_through as i64
        )
        .fetch_optional(self.db.pool())
        .await
        .map_err(Error::from)?;

        Ok(row.map(|r| Snapshot {
            covered_through: r.covered_through as u64,
            bytes: r.bytes,
        }))
    }

    async fn load_snapshot_best(
        &self,
        room_id: &str,
        max_covered_through: Option<LogSeq>,
    ) -> Result<Option<Snapshot>, Error> {
        let row = sqlx::query!(
            r#"
                SELECT
                    covered_through,
                    bytes
                FROM
                    room_snapshots
                WHERE
                    room_id = $1
                    AND ($2::bigint IS NULL
                        OR covered_through <= $2)
                ORDER BY
                    covered_through DESC
                LIMIT 1"#,
            room_id,
            max_covered_through.map(|max| max as i64)
        )
        .fetch_optional(self.db.pool())
        .await
        .map_err(Error::from)?;

        Ok(row.map(|r| Snapshot {
            covered_through: r.covered_through as u64,
            bytes: r.bytes,
        }))
    }

    async fn list_snapshots(&self, room_id: &str) -> Result<Vec<SnapshotInfo>, Error> {
        let rows = sqlx::query!(
            r#"
            SELECT
                covered_through,
                octet_length(bytes)::bigint AS "size_bytes!"
            FROM
                room_snapshots
            WHERE
                room_id = $1
            ORDER BY
                covered_through ASC"#,
            room_id
        )
        .fetch_all(self.db.pool())
        .await
        .map_err(Error::from)?;

        Ok(rows
            .into_iter()
            .map(|r| SnapshotInfo {
                covered_through: r.covered_through as u64,
                size_bytes: r.size_bytes as u64,
            })
            .collect())
    }
}
