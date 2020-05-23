# An integration into Envoy, using a `proxy-wasm` based filter

TODO

## Get started

1. [install `wasme`](https://docs.solo.io/web-assembly-hub/latest/installation/)
1. Run `rustup target add wasm32-unknown-unknown`
1. Run `make build-image`
1. Run `make run-local`
1. open <http://localhost:8080/headers>. You should see logs from the filter in
   the console
