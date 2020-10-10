FROM rust:1.47 AS build

RUN rustup component add rustfmt clippy

COPY Cargo.toml Cargo.lock ./

RUN mkdir -p ./src/ && echo 'fn main() {}' >./src/main.rs && echo '' >./src/lib.rs
RUN cargo build --release && rm -rf ./target/release/.fingerprint/feature_targeting_operator-*

COPY src ./src

RUN cargo clippy --release -- -D warnings && \
    cargo test --release && \
    cargo build --release

# ~~~~~~~~~~~~~~~~~~~~~~
FROM debian:buster-slim as release

RUN apt-get update && apt-get install -y \
    openssl \
    ca-certificates \
    tini \
    ;

RUN useradd svc
USER svc

COPY --chown=svc --from=build \
    /target/release/feature_targeting_operator \
    /

ENTRYPOINT ["/usr/bin/tini", "--"]

EXPOSE 8080
ENV RUSTLOG=roperator=debug
CMD ["/feature_targeting_operator"]
