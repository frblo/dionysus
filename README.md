# Dionysus

A web application for collaboratively writing screenplays in the [Fountain](https://fountain.io/syntax/) markup language. Allows for collaborative real time (CRT) editing on screenplays, and exporting them to multiple formats (such as HTML and PDF).

## Running

The recommended way to run Dionysus is via the OCI compose file. Remember to set the `.env` variables which can be found in `.env.example`.

```sh
podman-compose --file compose.yml up
```

Currently the web page is protected by the simple password `manus27`, but actual authentication is on its way.

## Issues

Please make issues on this repository if you experience problems with the application. Please direct any issues concerning the screenplay exporting to the [Rustwell](https://github.com/frblo/rustwell/issues) repository instead, as that is the engine handling the exports.
