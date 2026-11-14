use async_trait::async_trait;
use chrono::Duration;
use sqlx::{Executor, Postgres, Transaction};
use uuid::Uuid;

use crate::db::Db;
use crate::rooms::error::Error;
use crate::rooms::storage::{
    CreateRoomOptions, LoadUpdatesOptions, LogSeq, RoomInfo, Snapshot, SnapshotInfo, Storage,
    UpdateEntry, UpdateInfo,
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
    }

    async fn alloc_seq_range<'e, E>(ex: E, room_id: Uuid, n: i64) -> Result<(i64, i64), Error>
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
    async fn room_exists(&self, room_id: Uuid) -> Result<bool, Error> {
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

    async fn create_room(
        &self,
        room_name: &str,
        _opts: CreateRoomOptions,
    ) -> Result<RoomInfo, Error> {
        if room_name.is_empty() {
            return Err(Error::InvalidArgument(
                "Room name cannot be an empty string".to_string(),
            ));
        }

        let mut tx: Transaction<'_, Postgres> =
            self.db.pool().begin().await.map_err(Error::from)?;

        let row = sqlx::query!(
            r#"
            INSERT INTO rooms (room_name, last_seq)
                VALUES ($1, 0)
            RETURNING
                room_id,
                room_name,
                last_seq"#,
            room_name
        )
        .fetch_one(&mut *tx)
        .await
        .map_err(Error::from)?;

        tx.commit().await.map_err(Error::from)?;

        Ok(RoomInfo {
            room_id: row.room_id,
            room_name: row.room_name,
            last_seq: row.last_seq as u64,
            latest_snapshot: None,
        })
    }

    async fn delete_room(&self, room_id: Uuid) -> Result<(), Error> {
        // Assumes FK ON DELETE CASCADE to updates/snapshots.
        sqlx::query!(
            r#"DELETE FROM rooms
                        WHERE room_id = $1"#,
            room_id
        )
        .execute(self.db.pool())
        .await
        .map_err(Error::from)?;
        Ok(())
    }

    async fn rename_room(&self, room_id: Uuid, new_name: &str) -> Result<RoomInfo, Error> {
        let row = sqlx::query!(
            r#"UPDATE rooms
                SET room_name = $1
                WHERE room_id = $2
                RETURNING
                    room_id,
                    room_name,
                    last_seq"#,
            new_name,
            room_id,
        )
        .fetch_optional(self.db.pool())
        .await
        .map_err(Error::from)?;

        let Some(row) = row else {
            return Err(Error::NotFound);
        };

        Ok(RoomInfo {
            room_id: row.room_id,
            room_name: row.room_name,
            last_seq: row.last_seq as u64,
            latest_snapshot: None,
        })
    }

    async fn list_rooms(&self) -> Result<Vec<RoomInfo>, Error> {
        let rows = sqlx::query!(
            r#"
            SELECT
                r.room_id,
                r.room_name,
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
                room_name: r.room_name,
                last_seq: r.last_seq as u64,
                latest_snapshot: r.snap_covered.map(|ct| SnapshotInfo {
                    covered_through: ct as u64,
                    size_bytes: r.snap_size.unwrap_or(0) as u64,
                }),
            })
            .collect())
    }

    async fn get_room_info(&self, room_id: Uuid) -> Result<Option<RoomInfo>, Error> {
        let row = sqlx::query!(
            r#"
            SELECT
                r.room_id,
                r.room_name,
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
            room_name: r.room_name,
            last_seq: r.last_seq as u64,
            latest_snapshot: r.snap_covered.map(|ct| SnapshotInfo {
                covered_through: ct as u64,
                size_bytes: r.snap_size.unwrap_or(0) as u64,
            }),
        }))
    }

    async fn append_update(&self, room_id: Uuid, update: &[u8]) -> Result<LogSeq, Error> {
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
        room_id: Uuid,
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
        room_id: Uuid,
        opts: LoadUpdatesOptions,
    ) -> Result<Vec<UpdateEntry>, Error> {
        let from: i64 = opts.from.map_or(1, |v| v as i64);
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

    async fn store_snapshot(&self, room_id: Uuid, snapshot: Snapshot) -> Result<(), Error> {
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
        room_id: Uuid,
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
        room_id: Uuid,
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

    async fn list_snapshots(&self, room_id: Uuid) -> Result<Vec<SnapshotInfo>, Error> {
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

    async fn list_grouped_updates(
        &self,
        room_id: Uuid,
        interval: Duration,
    ) -> Result<Vec<UpdateInfo>, Error> {
        if interval <= Duration::zero() {
            return Err(Error::InvalidArgument(
                "interval must be positive".to_string(),
            ));
        }

        let pg_interval = sqlx::postgres::types::PgInterval::try_from(interval).map_err(|e| {
            Error::InvalidArgument(format!("interval cannot be represented in storage: {e}"))
        })?;

        let rows = sqlx::query!(
            r#"
            WITH latest AS (
                SELECT
                    max(seq) AS seq
                FROM
                    room_updates
                WHERE
                    room_id = $1
                GROUP BY
                    date_bin($2::interval, created_at, TIMESTAMPTZ 'epoch')
            )
            SELECT
                u.seq AS "covered_through!",
                u.created_at AS "created_at!"
            FROM
                room_updates u
                JOIN latest l ON l.seq = u.seq
            WHERE
                u.room_id = $1
            ORDER BY
                u.seq ASC"#,
            room_id,
            pg_interval
        )
        .fetch_all(self.db.pool())
        .await
        .map_err(Error::from)?;

        Ok(rows
            .into_iter()
            .map(|r| UpdateInfo {
                covered_through: r.covered_through as u64,
                created_at: r.created_at,
            })
            .collect())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use chrono::{DateTime, Utc};

    async fn make_room(storage: &DatabaseStorage) -> Uuid {
        storage
            .create_room("Test Room", CreateRoomOptions::default())
            .await
            .unwrap()
            .room_id
    }

    async fn insert_update(
        pool: &sqlx::PgPool,
        room_id: Uuid,
        seq: i64,
        created_at: DateTime<Utc>,
    ) {
        sqlx::query!(
            r#"
            INSERT INTO room_updates (room_id, seq, bytes, created_at)
                VALUES ($1, $2, $3, $4)"#,
            room_id,
            seq,
            b"update".as_slice(),
            created_at
        )
        .execute(pool)
        .await
        .unwrap();
    }

    fn at(timestamp: &str) -> DateTime<Utc> {
        timestamp.parse().unwrap()
    }

    #[sqlx::test]
    async fn list_grouped_updates_is_empty_for_room_without_updates(pool: sqlx::PgPool) {
        let storage = DatabaseStorage::new(Db::new(pool)).await;
        let room_id = make_room(&storage).await;

        let updates = storage
            .list_grouped_updates(room_id, Duration::hours(1))
            .await
            .unwrap();

        assert!(updates.is_empty());
    }

    #[sqlx::test]
    async fn list_grouped_updates_keeps_last_update_per_bucket(pool: sqlx::PgPool) {
        let storage = DatabaseStorage::new(Db::new(pool.clone())).await;
        let room_id = make_room(&storage).await;

        insert_update(&pool, room_id, 1, at("2026-01-01T10:01:00Z")).await;
        insert_update(&pool, room_id, 2, at("2026-01-01T10:30:00Z")).await;
        insert_update(&pool, room_id, 3, at("2026-01-01T10:59:59Z")).await;
        insert_update(&pool, room_id, 4, at("2026-01-01T11:00:00Z")).await;

        let updates = storage
            .list_grouped_updates(room_id, Duration::hours(1))
            .await
            .unwrap();

        assert_eq!(updates.len(), 2);
        assert_eq!(updates[0].covered_through, 3);
        assert_eq!(updates[0].created_at, at("2026-01-01T10:59:59Z"));
        assert_eq!(updates[1].covered_through, 4);
        assert_eq!(updates[1].created_at, at("2026-01-01T11:00:00Z"));
    }
}
