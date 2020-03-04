# Setup rust build environment
FROM rust:1.41.1 AS build-context

RUN rustup component add rustfmt

WORKDIR /usr/src/adapter-istio

COPY Cargo.toml .
COPY Cargo.lock .

# Layer hack: Build an empty program to compile dependencies and place on their own layer.
# This cuts down build time

# it was borrowed from here: 
# https://github.com/deislabs/krustlet/blob/master/Dockerfile#L7 
RUN mkdir -p ./src/ && \
  echo 'fn main() {}' > ./src/main.rs && \
  echo '' > ./src/lib.rs

RUN cargo fetch

RUN cargo build --release && \
  rm -rf ./target/release/.fingerprint/adapter-istio-*

# Setup debian release environment
FROM debian:buster-slim AS release-context

RUN apt-get update && apt-get install -y \
  tini \
  ;

RUN useradd svc

ENV PORT=50051

# Build real binaries now, as late as possible
FROM build-context AS build

COPY Makefile .
COPY ./src ./src
COPY ./template ./template
COPY ./build.rs .

RUN make release

# Create the release
FROM release-context AS release

COPY --from=build /usr/src/adapter-istio/target/release/adapter-istio /

RUN chown -R svc /adapter-istio

USER svc

ENTRYPOINT ["/usr/bin/tini", "--"]

CMD ["/adapter-istio"]
