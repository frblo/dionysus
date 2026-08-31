-- Add column room_name and migrate old room_id as room_name
ALTER TABLE rooms ADD COLUMN IF NOT EXISTS room_name text;
UPDATE rooms SET room_name = room_id WHERE room_name IS NULL;
ALTER TABLE rooms ALTER COLUMN room_name SET NOT NULL;

-- Change room_id to uuid instead of text, and generate new ids
ALTER TABLE rooms ADD COLUMN new_room_id uuid;
UPDATE rooms SET new_room_id = gen_random_uuid();

-- Replace old room_ids with new ones
ALTER TABLE room_updates DROP CONSTRAINT room_updates_room_id_fkey;
ALTER TABLE room_snapshots DROP CONSTRAINT room_snapshots_room_id_fkey;

UPDATE room_updates u SET room_id = r.new_room_id::text
    FROM rooms r
    WHERE r.room_id = u.room_id;

UPDATE room_snapshots s SET room_id = r.new_room_id::text
    FROM rooms r
    WHERE r.room_id = s.room_id;

ALTER TABLE rooms ALTER COLUMN room_id TYPE uuid USING new_room_id;
ALTER TABLE rooms ALTER COLUMN room_id SET DEFAULT gen_random_uuid();
ALTER TABLE rooms DROP COLUMN new_room_id;

ALTER TABLE room_updates ALTER COLUMN room_id TYPE uuid USING room_id::uuid;
ALTER TABLE room_snapshots ALTER COLUMN room_id TYPE uuid USING room_id::uuid;

ALTER TABLE room_updates
    ADD CONSTRAINT room_updates_room_id_fkey
    FOREIGN KEY (room_id) REFERENCES rooms (room_id) ON DELETE CASCADE;

ALTER TABLE room_snapshots
    ADD CONSTRAINT room_snapshots_room_id_fkey
    FOREIGN KEY (room_id) REFERENCES rooms (room_id) ON DELETE CASCADE;
