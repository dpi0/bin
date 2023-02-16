FROM rust:1-slim AS builder

RUN apt update && apt install -y libclang-dev

COPY . /sources
WORKDIR /sources
RUN cargo build --release
RUN chown nobody:nogroup /sources/target/release/bin


FROM debian:bullseye-slim
COPY --from=builder /sources/target/release/bin /pastebin

RUN mkdir /srv/pastes
RUN chown 1000:1000 /srv/pastes
USER 1000
EXPOSE 8000
ENTRYPOINT ["/pastebin", "0.0.0.0:8000", "--paste-dir", "/srv/pastes"]
