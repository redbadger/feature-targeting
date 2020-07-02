# Todo MVC Web UI

A client-side [Todo MVC][todomvc] Web UI written in Rust. 

Built with:

- [`seed`][seed]
- [`graphql-client`][graphql-client]
- [`wasm-pack`][wasm-pack]

Currently, this is configured against a private Google GSuite OIDC Identity Provider. We will make this more configurable, but for now you may need to point it at your own IDP in order for the authentication bit to work.

We will also be updating this soon to demonstrate feature targeting capabilities.

### To get started

- you should build and run the [API](../api/README.md) first

- install `wasm-pack`, either directly:

    ```sh
    curl https://rustwasm.github.io/wasm-pack/installer/init.sh -sSf | sh 
    ```

    or with Cargo:

    ```sh
    cargo install wasm-pack
    ```

- if you don't have a static webserver:
  
    ```sh
    brew install httpserver
    ```

- build and serve:

    ```sh
    wasm-pack build --target web && httpserver
    ```

[graphql-client]: https://github.com/graphql-rust/graphql-client
[seed]: https://github.com/seed-rs/seed
[todomvc]: http://todomvc.com/
[wasm-pack]: https://rustwasm.github.io/wasm-pack/