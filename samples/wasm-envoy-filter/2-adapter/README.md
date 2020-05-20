# Feature Targeting for Istio 1.5+, using envoy filter (WASM).

In this example, we will use `wasme` to install the filter into the envoy proxies in Istio. There are detailed instructions [here](https://docs.solo.io/web-assembly-hub/latest/tutorial_code/deploy_tutorials/deploying_with_istio/), or you can follow along below:

1.  install Istio 1.5 to a cluster (see [readme](../1-istio/README.md))
1.  install the `echo-service` pod (see [readme](../../echo-service/README.md))
1.  test that you get a response (without feature header injection):

    ```sh
    curl --resolve echo.localhost:80:127.0.0.1 -vvv http://echo.localhost
    ```

1.  ensure you have `wasme` installed:

    ```sh
    curl -sL https://run.solo.io/wasme/install | sh
    export PATH=$HOME/.wasme/bin:$PATH
    ```

1.  ensure you have the wasm target added (`rustup target add wasm32-unknown-unknown`).

1.  build the filter:

    ```sh
    cd ../../../adapter-proxy-wasm
    make build-image
    ```

1.  You should be able to see the built image (by running `wasme list`)

    ```
    NAME                                TAG   SIZE   SHA      UPDATED
    docker.io/library/feature-targeting 0.1.0 1.8 MB 8aedf1fd 20 May 20 13:54 BST
    ```

1.  Deploy to the cluster:

    ```sh
    cd ../../../adapter-proxy-wasm
    wasme deploy istio \
        docker.io/library/feature-targeting:0.1.0 \
        --id=featureFlagging \
        --namespace echo-service \
        --config="$(cat filter-config.json)"
    ```
