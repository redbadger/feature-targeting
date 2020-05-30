# Feature targeting samples

This set of samples should demonstrate the capabilities feature targeting
can provide.

## Getting started with Istio 1.6, and above

This example uses a WASM Envoy filter

First, create a cluster and install Istio 1.6 (see the [readme](./wasm-envoy-filter/1-istio/README.md)).

Then, either submit an `EnvoyFilter` resource directly (see the [readme](./wasm-envoy-filter/2-adapter/README.md)), or configure the `feature-targeting-operator` (see the [readme](../feature-targeting-operator/README.md)) and submit a (much simpler) `FeatureTargetingConfig` CRD.

## Getting started with Istio 1.4.x, and below

This example uses an out-of-process Istio Mixer Adapter

- create a cluster and install Istio: [`init-cluster`](./mixer-adapter/1-init-cluster/README.md)
- install the feature targeting Istio adapter [`enable-adapter`](./mixer-adapter/2-enable-adapter/README.md)

## Samples

- [echo service](./echo-service/README.md) - A simple echo service that shows
  feature targeting header injection.
