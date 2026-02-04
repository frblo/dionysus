FROM rust:1.67

WORKDIR /backend
COPY . .

RUN cargo build --release

CMD [ "./target/release/backend" ]
