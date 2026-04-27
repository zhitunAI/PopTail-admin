FROM rust:1.88-bookworm AS builder

WORKDIR /app

COPY Cargo.toml Cargo.lock ./
COPY src ./src
COPY tests ./tests

RUN cargo build --release

FROM debian:bookworm-slim

RUN apt-get update \
    && apt-get install -y --no-install-recommends ca-certificates \
    && rm -rf /var/lib/apt/lists/*

WORKDIR /app

COPY --from=builder /app/target/release/pop-tail-auth /usr/local/bin/pop-tail-auth

ENV RUST_LOG=info \
    POP_TAIL_BIND_ADDR=0.0.0.0:8888

EXPOSE 8888

CMD ["pop-tail-auth"]
