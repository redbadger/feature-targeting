# Feature Targeting for Istio 1.5+, using envoy filter (WASM).

1.  install Istio 1.5 to a cluster (see [readme](../1-istio/README.md))
1.  install the `echo-service` pod (see [readme](../../echo-service/README.md))
1.  test that you get a response (without feature header injection):

    ```sh
    curl --resolve echo.localhost:80:127.0.0.1 -vvv http://echo.localhost
    ```

1.  ensure you have the wasm target added (`rustup target add wasm32-unknown-unknown`).

1.  build the filter and copy it to `/tmp/envoy-filters`, so that it will be mounted in the sidecar container:

    ```sh
    cd ../../../adapter-proxy-wasm
    make build-image
    mkdir -p /tmp/envoy-filters
    cp feature_targeting.wasm /tmp/envoy-filters
    ```

1.  deploy the `EnvoyFilter` resource to the cluster:

    ```sh
    make
    ```

1.  test that you get a response (with default feature header injection):

    ```sh
    curl --resolve echo.localhost:80:127.0.0.1 -vvv http://echo.localhost
    ```
