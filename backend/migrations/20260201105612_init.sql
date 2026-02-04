-- Rooms table: holds per-room metadata, including last_seq counter
CREATE TABLE IF NOT EXISTS rooms (
    room_id text PRIMARY KEY,
    last_seq bigint NOT NULL DEFAULT 0
);

-- Update log: deterministic ordering by (room_id, seq)
CREATE TABLE IF NOT EXISTS room_updates (
    room_id text NOT NULL REFERENCES rooms (room_id) ON DELETE CASCADE,
    seq bigint NOT NULL,
    bytes bytea NOT NULL,
    created_at timestamptz NOT NULL DEFAULT now(),
    PRIMARY KEY (room_id, seq)
);

CREATE INDEX IF NOT EXISTS room_updates_room_seq_idx ON room_updates (room_id, seq);

-- Snapshots: multiple per room, identified by covered_through
CREATE TABLE IF NOT EXISTS room_snapshots (
    room_id text NOT NULL REFERENCES rooms (room_id) ON DELETE CASCADE,
    covered_through bigint NOT NULL,
    bytes bytea NOT NULL,
    created_at timestamptz NOT NULL DEFAULT now(),
    PRIMARY KEY (room_id, covered_through)
);

CREATE INDEX IF NOT EXISTS room_snapshots_room_cov_idx ON room_snapshots (room_id, covered_through DESC);

