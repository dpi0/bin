# Build stage
FROM rust:1-alpine AS builder

RUN apk add --no-cache musl-dev

WORKDIR /build

COPY . .

RUN cargo build --release

# Run stage
FROM alpine:latest

WORKDIR /app

COPY --from=builder /build/target/release/bin ./bin

RUN mkdir -p ./pastes

RUN chown -R 1000:1000 ./pastes

USER 1000

EXPOSE 8000

ENTRYPOINT ["./bin", "0.0.0.0:8000", "--paste-dir", "/app/pastes"]