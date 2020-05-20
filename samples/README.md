# Feature targeting samples

This set of samples should demonstrate the capabilities feature targeting
can provide.

## Getting started with Istio 1.5, and above

This example uses a WASM Envoy filter

- create a cluster and install Istio: [`init-cluster`](./wasm-envoy-filter/1-init-cluster/README.md)
- install the feature targeting Istio adapter [`enable-adapter`](./wasm-envoy-filter/2-enable-adapter/README.md)

## Getting started with Istio 1.4.x, and below

This example uses an out-of-process Istio Mixer Adapter

- create a cluster and install Istio: [`init-cluster`](./mixer-adapter/1-init-cluster/README.md)
- install the feature targeting Istio adapter [`enable-adapter`](./mixer-adapter/2-enable-adapter/README.md)

## Samples

- [Hello World](./hello-world/README.md) - A simple hello world that shows
  feature state injection happening with an Echo server.
