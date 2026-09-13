CREATE TABLE user_roles (
    user_id     uuid PRIMARY KEY REFERENCES users (user_id) ON DELETE CASCADE,
    global_role text NOT NULL CHECK (global_role IN ('guest', 'user', 'admin')),
    updated_at  timestamptz NOT NULL DEFAULT now()
);

CREATE TABLE room_members (
    room_id    uuid NOT NULL REFERENCES rooms (room_id) ON DELETE CASCADE,
    user_id    uuid NOT NULL REFERENCES users (user_id) ON DELETE CASCADE,
    role       text NOT NULL CHECK (role IN ('owner', 'editor', 'viewer')),
    created_at timestamptz NOT NULL DEFAULT now(),
    PRIMARY KEY (room_id, user_id)
);

CREATE INDEX room_members_user_idx ON room_members (user_id);
