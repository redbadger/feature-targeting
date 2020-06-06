## TodoMVC Demo

This is a [Yew](https://yew.rs/docs/) implementation of [TodoMVC](http://todomvc.com/) app.

Stores the full state of the model, including all entries, entered text and chosen filter in local storage.

### Building

If you haven't already, install [`wasm-pack`](https://rustwasm.github.io/wasm-pack/):

```sh
cargo install wasm-pack

// or

curl https://rustwasm.github.io/wasm-pack/installer/init.sh -sSf | sh
```

Then:

```sh
wasm-pack build --target web
```

### Running

You can use your own local webserver or, for example, install the VSCode extension [Live Server](https://marketplace.visualstudio.com/items?itemName=ritwickdey.LiveServer) and simply right-click [`static/index.html`](static/index.html) and choose "Open with Live Server".
