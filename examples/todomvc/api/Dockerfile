FROM rust:1.47 AS build

RUN rustup component add rustfmt clippy

COPY Cargo.toml Cargo.lock ./

RUN mkdir -p ./src/ && echo 'fn main() {}' >./src/main.rs && echo '' >./src/lib.rs
RUN cargo build --release && rm -rf ./target/release/.fingerprint/todomvc_api-*

COPY sql ./sql
COPY sqlx-data.json .
COPY src ./src

RUN cargo clippy --release -- -D warnings && \
    cargo test --release && \
    cargo build --release

# ~~~~~~~~~~~~~~~~~~~~~~
FROM debian:buster-slim as release

RUN apt-get update && apt-get install -y \
    openssl \
    tini \
    && rm -rf /var/lib/apt/lists/*

RUN useradd svc
USER svc

COPY --chown=svc --from=build \
    /target/release/todomvc_api \
    /

ENTRYPOINT ["/usr/bin/tini", "--"]

EXPOSE 3030
CMD ["/todomvc_api"]
