FROM rust:1.75-alpine AS builder

WORKDIR /app

RUN apk add --no-cache musl-dev openssl-dev pkgconfig

COPY Cargo.toml Cargo.lock ./
COPY backend ./backend

WORKDIR /app/backend

RUN cargo build --release --locked

FROM alpine:3.19

RUN apk add --no-cache openssl ca-certificates

WORKDIR /app

COPY --from=builder /app/backend/target/release/miniwiki-backend /app/
COPY --from=builder /app/backend/migrations ./migrations
COPY --from=builder /app/backend/.env.example ./.env.example

EXPOSE 8080

CMD ["./miniwiki-backend"]
