FROM rust:1.47 AS build

RUN rustup component add rustfmt clippy

COPY server/Cargo.toml server/Cargo.lock /

RUN mkdir -p ./src/ && echo 'fn main() {}' >./src/main.rs && echo '' >./src/lib.rs
RUN cargo build --release && rm -rf ./target/release/.fingerprint/todomvc_web_server-*

COPY server/src src

RUN cargo clippy --release -- -D warnings && \
    cargo test --release && \
    cargo build --release

# ~~~~~~~~~~~~~~~~~~~~~~
FROM debian:buster-slim as release

RUN apt-get update && apt-get install -y \
    curl \
    openssl \
    tini \
    && rm -rf /var/lib/apt/lists/*

RUN useradd svc
USER svc

COPY client/index.html /client/
COPY client/pkg/todomvc_web_client.js /client/pkg/
COPY client/pkg/todomvc_web_client_bg.wasm /client/pkg/
COPY client/public /client/public/

COPY --chown=svc --from=build \
    /target/release/todomvc_web_server \
    /server/

ENTRYPOINT ["/usr/bin/tini", "--"]

EXPOSE 8080
CMD ["/server/todomvc_web_server"]
