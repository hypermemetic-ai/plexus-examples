# plexus-examples

Small services written with `#[plexus::service]`. Every part of Plexus v2 is proven on them.

| | |
|---|---|
| `kiln` | A kiln and its firing log: an `async fn`, an `impl Stream`, a typed error. Builds to a wasm component. |
| `glaze` | Glaze recipes: a second service, held beside the kiln. Builds to a wasm component. |
| `showcase` | One small service per thing the IR can say. Each compiles; none is served. |

```sh
cargo build -p showcase
cargo build -p kiln -p glaze --profile web --target wasm32-unknown-unknown
```

The Plexus crates are expected beside this checkout: `../plexus`, `../plexus-macros`. Tracker: LAYERS-9.
