# plexus-examples

Small activations written with `#[plexus::activation]`. Every part of Plexus v2 is proven on them.

| | |
|---|---|
| `kiln` | A kiln and its firing log: an `async fn`, an `impl Stream`, a typed error. Builds to a wasm component. |
| `glaze` | Glaze recipes: a second activation, held beside the kiln. Builds to a wasm component. |
| `showcase` | One small activation per thing the IR can say. Each compiles; none is served. |

```sh
cargo build -p showcase
cargo build -p kiln -p glaze --profile web --target wasm32-unknown-unknown
```

The Plexus crates are expected beside this checkout: `../plexus`, `../plexus-macros`. Tracker: LAYERS-9.
