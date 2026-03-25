FROM rust:1-slim AS builder

WORKDIR /app

RUN apt-get update && apt-get install -y pkg-config libssl-dev && rm -rf /var/lib/apt/lists/*

COPY Cargo.toml Cargo.lock ./
COPY crates/ crates/
COPY example/rust/ example/rust/

RUN cargo build --release --package flux-golden

FROM debian:bookworm-slim

RUN apt-get update && apt-get install -y ca-certificates && rm -rf /var/lib/apt/lists/*

COPY --from=builder /app/target/release/twitterd /usr/local/bin/twitterd

EXPOSE 3000

CMD ["twitterd"]
