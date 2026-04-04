# Dionysus

A web application for collaboratively writing screenplays in the [Fountain](https://fountain.io/syntax/) markup language. Allows for collaborative real time editing on screenplays, and exporting them to multiple formats (such as HTML and PDF).

## Running

The recommended way to run Dionysus locally (e.g. during development) is via the compose file.

```sh
podman-compose --file compose.yml up
```

This starts the application, a local `PostgreSQL` instance, and a mock OIDC provider with sensible development defaults.

For production deployments, use the published Docker image and configure it as described in the [Configuration](#configuration) section. Mount a configuration file to `/etc/dionysus/config.toml` or set `DIONYSUS_` environment variables as needed.

## Configuration

### General

| Setting | Required | Format |
| --- | --- | --- |
| database.url | Yes | postgresql://USER:PWD@HOST:PORT/DB |
| oidc.external_base_url | Yes | Base for externally reachable url. E.g. "https://your-domain.com" |
| oidc.providers.<name> | Yes | See [OIDC provider config](#oidc-providers-config) section |
| logging.filter | No | Default: "info,tower_http=debug" |
| logging.json | No | Default: False |
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

### OIDC providers config
Dionysus supports having multiple OIDC providers at the same time. Each one of these needs the following information.

| Setting | Required | Format |
| --- | --- | --- |
| oidc.providers.<name>.issuer | Yes | Base url of the issuer |
| oidc.providers.<name>.external_issuer | No | An external url to the issuer reachable from the browser |
| oidc.providers.<name>.client_id | Yes | Client id for issuer |
| oidc.providers.<name>.client_secret | Yes | Client secret for issuer |
| oidc.providers.<name>.scopes | Yes | A list of the scopes to request from the issuer |

These are configured in the same way as described above where <name> specifies the identifier for the issuer.

```toml
[oidc.providers.your_oidc]
issuer = "https://your-oidc-domain"
client_id = "oidc-client"
client_secret = "oidc-secret" # Should preferably be stored in an environment variable
scopes = ["openid", "profile", "email"]
```
or the same using environment variables

```sh
DIONYSUS_OIDC__PROVIDERS__YOUR_OIDC__ISSUER="https://your-oidc-domain"
DIONYSUS_OIDC__PROVIDERS__YOUR_OIDC__CLIENT_ID="oidc-client"
DIONYSUS_OIDC__PROVIDERS__YOUR_OIDC__CLIENT_SECRET="oidc-secret"
DIONYSUS_OIDC__PROVIDERS__YOUR_OIDC__SCOPES__0="openid"
DIONYSUS_OIDC__PROVIDERS__YOUR_OIDC__SCOPES__1="profile"
DIONYSUS_OIDC__PROVIDERS__YOUR_OIDC__SCOPES__2="email"
```

## Issues

Please make issues on this repository if you experience problems with the application. Please direct any issues concerning the screenplay exporting to the [Rustwell](https://github.com/frblo/rustwell/issues) repository instead, as that is the engine handling the exports.
