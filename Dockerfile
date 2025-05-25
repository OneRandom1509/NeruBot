FROM rust:1.70 as builder

RUN rustup update stable && rustup default stable

WORKDIR /usr/src/app

COPY Cargo.toml Cargo.lock ./

COPY src ./src

RUN cargo build --release

FROM debian:bookworm-slim

WORKDIR /usr/src/app

COPY --from=builder /usr/src/app/target/release/NeruBot ./

RUN apt-get update && apt-get install -y libssl-dev && rm -rf /var/lib/apt/lists/*

EXPOSE 8080

CMD ["./NeruBot"]
