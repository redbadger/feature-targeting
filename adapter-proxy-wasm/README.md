# An integration into Envoy, using a `proxy-wasm` based filter

TODO

## Get started

1. [install `wasme`](https://docs.solo.io/web-assembly-hub/latest/installation/)
1. Run `rustup target add wasm32-unknown-unknown`
1. Run `make build-image`
1. Run `make run-local`
1. open <http://localhost:8080/headers>. You should see logs from the filter in
   the console

## Install the adapter into Istio using the feature targeting operator

The feature targeting operator relies on the built adapter being mounted
into the pod using a persistent volume. You can look at the [echo service example](../examples/echo-service/README.md)
to see how the volume is configured. It expects the built envoy filter to be
present in `/tmp/envoy-filters` on the host machine. To build and install it
run

```sh
make install
```

Now you can [deploy the echo service example](../examples/echo-service/README.md)
and follow the steps in the [operator readme](../feature-targeting-operator/README.md#installation-and-testing)
to install the filter in the echo service's sidecar.
