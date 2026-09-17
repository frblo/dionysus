-- A non-null email is always verified.
-- Unique so it can be used to detect account collisions.
ALTER TABLE users ADD COLUMN email text UNIQUE;

ALTER TABLE users ADD COLUMN last_login_at timestamptz NOT NULL DEFAULT now();
