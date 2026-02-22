# Dionysus

A web application for collaboratively writing screenplays in the [Fountain](https://fountain.io/syntax/) markup language. Allows for collaborative real time (CRT) editing on screenplays, and exporting them to multiple formats (such as HTML and PDF).

## NOTE
Currently the web page is protected by the simple password `manus27`, but actual authentication is on its way.

## Running

The recommended way to run Dionysus locally (e.g. during development) is via the compose file.

```sh
podman-compose --file compose.yml up
```
This starts both the application and a local `PostgreSQL` instance with sensible development defaults.

For production deployments, use the published Docker image and configure it as described in the [Configuration](#Configuration) section. Mount a configuration file to `/etc/dionysus/config.toml` or set `DIONYSUS_` environment variables as needed.

## Configuration

| Setting | Required | Format |
| --- | --- | --- |
| database.url | Yes | postgresql://USER:PWD@HOST:PORT/DB |
| listener.ip | No | Default: 0.0.0.0 (listen everywhere) |
| listener.port | No | Default: 8000 |

These settings can be configured either through a `TOML` file or `DIONYSUS_` environment variables.

The default configuration file location is `/etc/dionysus/config.toml`. To use a different path, set `DIONYSUS_CONFIG` environment variable to desired path.
When using the Docker image, mount your configuration file to `/etc/dionysus/config.toml` (recommended read only).

Example (Compose):
```yaml
services:
    dionysus:
        image: ghcr.io/frblo/dionysus:latest
        volumes:
            - ./dionysus.toml:/etc/dionysus/config.toml:ro
```

Environment variables are prefixed with `DIONYSUS_`. They use uppercase names and replace dots (`.`) with double underscores (`__`) to represent nested configuration keys (e.g. `DIONYSUS_DATABASE__URL`).

Environment variables override values from the configuration file.

For secrets, it is generally recommended to use environment variables rather than storing them in a file.

## Issues

Please make issues on this repository if you experience problems with the application. Please direct any issues concerning the screenplay exporting to the [Rustwell](https://github.com/frblo/rustwell/issues) repository instead, as that is the engine handling the exports.
