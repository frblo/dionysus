ARG RUST_VERSION=1.93
ARG NODE_VERSION=25.6.0

# ============================
# Rust toolchain stage
#
# Install build tools and keep them in a cache layer.
# These are used for both wasm and for the backend
# ============================
FROM rust:${RUST_VERSION}-slim AS rust-tools
WORKDIR /app
RUN --mount=type=cache,target=/usr/local/cargo/registry \
    --mount=type=cache,target=/usr/local/cargo/git \
    rustup target add wasm32-unknown-unknown && \
    cargo install --locked cargo-chef wasm-pack

# ============================
# WASM (converter) build stages
# ============================

# 1) Use cargo chef to prepare dependencies
FROM rust-tools AS wasm-planner
WORKDIR /app/wasm
COPY frontend/src/lib/converter/ ./
RUN cargo chef prepare --recipe-path recipe.json

# 2) Cook dependencies
FROM rust-tools AS wasm-builder
WORKDIR /app/wasm
ENV CARGO_TARGET_DIR=/app/target/wasm

COPY --from=wasm-planner /app/wasm/recipe.json recipe.json
RUN --mount=type=cache,target=/usr/local/cargo/registry \
    --mount=type=cache,target=/usr/local/cargo/git \
    cargo chef cook --release --target wasm32-unknown-unknown

# 3) Build the actual wasm pkg
COPY frontend/src/lib/converter/ ./
RUN --mount=type=cache,target=/usr/local/cargo/registry \
    --mount=type=cache,target=/usr/local/cargo/git \
    wasm-pack build --target web --release

# ============================
# Frontend Builder Stage
# ============================
FROM node:${NODE_VERSION}-alpine AS front-builder
WORKDIR /usr/src/dionysus/frontend

# Cache dependencies
COPY frontend/package*.json ./
RUN --mount=type=cache,target=/root/.npm \
    npm ci

COPY frontend .

# Replace converter source with prebuilt wasm pkg
RUN rm -rf src/lib/converter
COPY --from=wasm-builder /app/wasm/pkg src/lib/converter/pkg/

RUN npm run build

# ============================
# Backend Builder Stages
# ============================

# 1) Plan dependencies
FROM rust-tools AS back-planner
WORKDIR /app/backend
COPY backend/ ./
RUN cargo chef prepare --recipe-path recipe.json

# 2) Cook deps + build binary
FROM rust-tools AS back-builder
WORKDIR /app/backend

ENV CARGO_TARGET_DIR=/app/backend/target

COPY --from=back-planner /app/backend/recipe.json recipe.json
RUN --mount=type=cache,target=/usr/local/cargo/registry \
    --mount=type=cache,target=/usr/local/cargo/git \
    cargo chef cook --release --recipe-path recipe.json

COPY backend/ ./
RUN --mount=type=cache,target=/usr/local/cargo/registry \
    --mount=type=cache,target=/usr/local/cargo/git \
    cargo build --release

# ============================
# Runtime stage
# ============================
FROM debian:trixie-slim AS runtime
RUN apt-get update && apt-get install -y ca-certificates && rm -rf /var/lib/apt/lists/*

WORKDIR /usr/local/bin/
COPY --from=back-builder /app/backend/target/release/backend ./backend
COPY --from=front-builder /usr/src/dionysus/frontend/build ./build

# If configured to something else you should set it manually when used.
# e.g. by `-p` flag or ports in a compose file.
EXPOSE 8000

CMD [ "./backend" ]
