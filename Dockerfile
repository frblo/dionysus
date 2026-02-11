# == WASM Build Stage ==
FROM rust:1.93 as wasm_builder
WORKDIR /usr/src/dionysus/frontend/wasm

COPY frontend/src/lib/converter .
RUN cargo install wasm-pack
RUN wasm-pack build --target web

# == Frontend Builder Stage ==
FROM node:25.6.0-alpine AS front-builder
WORKDIR /usr/src/dionysus/frontend

COPY frontend/package*.json ./
RUN npm ci

COPY frontend .

RUN rm -rf src/lib/converter
COPY --from=wasm_builder /usr/src/dionysus/frontend/wasm/pkg src/lib/converter/pkg/

RUN npm run build

# == Backend Build stage ==
FROM rust:1.93 as back-builder

WORKDIR /usr/src/dionysus/backend

COPY backend .

RUN cargo build --release

# == Running stage ==
FROM debian:trixie-slim as runtime

RUN apt-get update && apt-get install -y ca-certificates && rm -rf /var/lib/apt/lists/*

WORKDIR /usr/local/bin/
COPY --from=back-builder /usr/src/dionysus/backend/target/release/backend .
COPY --from=front-builder /usr/src/dionysus/frontend/build ./build

EXPOSE 8000
CMD [ "./backend", "serve" ]
