FROM rust:1.92-bookworm AS builder

WORKDIR /app

RUN apt-get update && apt-get install -y libssl-dev pkg-config

COPY Cargo.toml .

COPY . .

WORKDIR /app

RUN cargo build --release

FROM debian:bookworm-slim

RUN apt-get update && apt-get install -y ca-certificates openssl && rm -rf /var/lib/apt/lists/*

WORKDIR /app

COPY --from=builder /app/target/release/miniwiki-backend /app/
COPY --from=builder /app/migrations ./migrations


EXPOSE 8080

CMD ["./miniwiki-backend"]
