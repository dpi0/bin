# Build stage
FROM rust:1-alpine AS builder

RUN apk add --no-cache musl-dev

WORKDIR /build

COPY . .

RUN cargo build --release

RUN chown nobody:nogroup /build/target/release/bin

# Set ownership and create the pastes directory with correct permissions
RUN mkdir -p /build/pastes \
  && chown nobody:nogroup /build/pastes

# Run stage
FROM scratch

WORKDIR /app

COPY --from=builder /build/target/release/bin ./bin

COPY --from=builder /etc/passwd /etc/passwd

# Copy the pastes directory with the correct permissions
COPY --from=builder --chown=nobody:nogroup /build/pastes ./pastes

USER nobody

EXPOSE 8000

ENTRYPOINT ["./bin", "0.0.0.0:8000", "--paste-dir", "/app/pastes"]