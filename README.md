# Dionysus

## Running

The recommended way to run Dionysus is via the OCI compose file. Remember to set the `.env` variables which can be found in `.env.example`.

### Prepare database

Before launching the application the database needs to be prepared. This only has to be done when first installing the database. The instructions assume you'll use the compose file as it is.

#### Step 1

Open a terminal (we'll call this terminal `T1`) in the project directory and run. This will start the database on its own.

```sh
podman run \
  --name local-postgres \
  -e POSTGRES_USER=postgres \
  -e POSTGRES_PASSWORD=postgres \
  -e POSTGRES_DB=dionysus_db \
  -p 5432:5432 \
  -v dionysus_pgdata:/var/lib/postgresql \
  postgres:18
```

#### Step 2

Open another terminal (`T2`) and run the commands below. If you haven't already, you need to `cargo install sqlx-cli` to these commands. These commands prepare the database according to the schemas.

```sh
cd backend
cargo sqlx migrate run --database-url postgres://postgres:postgres@localhost:5432/dionysus_db
cargo sqlx prepare --database-url postgres://postgres:postgres@localhost:5432/dionysus_db
```

#### Step 3

In `T1` now run this command. This removes our temporary database container, but not the underlying data volume.

```sh
podman rm local-postgres
```

This concludes the preparations.

### Run

```sh
podman-compose --file compose.yml up
```
