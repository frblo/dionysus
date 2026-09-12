# Dionysus Backend

See root `README.md` for info on how to run.

## Running the backend for local development

Start just the database from the root of the repo and wait for it to be healthy:

```sh
docker compose up -d --wait postgres
```

Copy `.env.example` to `.env` in this directory (or export the same two variables yourself):

```sh
DATABASE_URL=postgres://dionysus:dionysus@localhost:5432/dionysus
SQLX_OFFLINE=true
```

These two variables are used by different things:

- `DATABASE_URL` — a real connection, used by `sqlx-cli` (`sqlx migrate run`, `cargo sqlx prepare`) and, at runtime, by `cargo test`'s `#[sqlx::test]` cases and the server itself on startup.
- `SQLX_OFFLINE=true` — only affects the `sqlx::query!` macros' *compile-time* checks. It's what lets `cargo build` / `cargo check` / rust-analyzer read the committed `.sqlx/` cache instead of connecting to a database.

So `cargo build` and `cargo check` never need Postgres running, but `cargo test` still does. No matter `SQLX_OFFLINE` `#[sqlx::test]` connects to a real database at runtime to execute the tests. If Postgres isn't reachable `#[sqlx::test]` doesn't skip or fail fast, each one hangs for ~30s before failing with `PoolTimedOut`.

Then:
```sh
sqlx migrate run
cargo test
```

### Updating the offline query cache

After adding or changing a `sqlx::query!` or `query_scalar!` call, run `cargo sqlx prepare -- --tests`. Before doing so make sure Postgres is up and migrated (in accordance to the above instructions).
