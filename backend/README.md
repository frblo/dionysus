# Dionysus Backend

See root `README.md` for info on how to run.

## Running the backend for local development

Start just the database from the root of the repo and wait for it to be healthy:

```sh
docker compose up -d --wait postgres
```

Copy `.env.example` to `.env` in this directory (or export it yourself):

```sh
DATABASE_URL=postgres://dionysus:dionysus@localhost:5432/dionysus
```

`DATABASE_URL` is used by `sqlx-cli` (e.g. `sqlx migrate run`, `cargo sqlx prepare`) and by `cargo test`'s `#[sqlx::test]` cases.

`SQLX_OFFLINE=true` is specified in `.cargo/config.toml`. This means that `cargo build` / `cargo check` / `rust-analyzer` will read the committed `.sqlx/` cache instead of connecting to Postgres. 

**Note**: `cargo test` still needs Postgres at runtime for `#[sqlx::test]`, if the database is unreachable, each case hangs ~30s before failing with `PoolTimedOut`.

With the server up and running and variables set run:
```sh
sqlx migrate run
cargo test
```

### Updating the offline query cache

After adding or changing a `sqlx::query!` or `query_scalar!` call, run `cargo sqlx prepare -- --tests` to update the `.slqx/` offline cache (Postgres must be up and migrated).

### Updating frontend TypeScript bindings

Types shared with the frontend derive [`ts-rs`](https://github.com/Aleph-Alpha/ts-rs). To stay out of production binary they are behind `#[cfg_attr(test, ...))]` flags. Any changes to the Rust types should regenerate the `.ts` files.

Run `cargo test` (or `cargo test export_bindings` for only bindings), which writes the `.ts` files to `frontend/src/lib/api/generated/`. Output path is set via `TS_RS_EXPORT_DIR` specified in `.cargo/config.toml`.
