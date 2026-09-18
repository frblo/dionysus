CREATE TABLE users (
    user_id      uuid PRIMARY KEY DEFAULT gen_random_uuid(),
    display_name text NOT NULL,
    created_at   timestamptz NOT NULL DEFAULT now(),
    updated_at   timestamptz NOT NULL DEFAULT now()
);

CREATE TABLE user_identities (
    provider_id text NOT NULL,
    subject     text NOT NULL,
    user_id     uuid NOT NULL REFERENCES users (user_id) ON DELETE CASCADE,
    created_at  timestamptz NOT NULL DEFAULT now(),
    PRIMARY KEY (provider_id, subject)
);

CREATE INDEX user_identities_user_idx ON user_identities (user_id);
