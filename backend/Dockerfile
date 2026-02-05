# == Build stage ==
FROM rust:1.93 as builder

WORKDIR /usr/src/dionysus/backend

COPY . .

RUN cargo build --release

# == Running stage ==
FROM debian:trixie-slim as runtime

RUN apt-get update && apt-get install -y ca-certificates && rm -rf /var/lib/apt/lists/*

WORKDIR /usr/local/bin/
COPY --from=builder /usr/src/dionysus/backend/target/release/backend .

EXPOSE 8000
CMD [ "./backend" ]
