FROM rust:1.83-alpine AS builder

WORKDIR /app

RUN apk add --no-cache musl-dev openssl-dev pkgconfig

COPY Cargo.toml .

COPY . .

WORKDIR /app

RUN cargo build --release

FROM alpine:3.19

RUN apk add --no-cache openssl ca-certificates

WORKDIR /app

COPY --from=builder /app/target/release/miniwiki-backend /app/
COPY --from=builder /app/migrations ./migrations
COPY --from=builder /app/.env.example ./.env.example

EXPOSE 8080

CMD ["./miniwiki-backend"]
