# == Build stage ==
FROM rust:1.93 as builder


WORKDIR /usr/src/dionysus/backend

COPY . .

RUN cargo build --release

# == Running stage ==
FROM debian:trixie-slim as runtime

WORKDIR /usr/local/bin/

COPY --from=builder /usr/src/dionysus/backend/target/release/backend .

CMD [ "./backend" ]
