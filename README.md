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

Every example's IR is kept in `ir/`: a showcase file as `ir/<file>.json`, kiln and glaze as `ir/kiln.json` and `ir/glaze.json`. plexus-macros' `goldens` test fails, showing the difference, when the macro writes anything else; plexus-ir reads and writes each back. To keep what the macro writes now, from `../plexus-macros`:

```sh
BLESS=1 cargo test goldens
```

Each IR's TypeScript and WIT, as `plexus-gen` writes them from the kept IR, are in `ts/<name>.plexus.ts` and `wit/<name>.wit`. plexus-gen's `goldens` test fails, showing the difference, when it writes anything else; to keep what it writes now, from `../plexus-gen`:

```sh
BLESS=1 cargo test goldens
```

The WIT in `wit/` says `func` where an example's own says `async func`: an IR's JSON does not say which methods wait.

The Plexus crates are expected beside this checkout: `../plexus`, `../plexus-macros`, `../plexus-ir`. Tracker: LAYERS-9.
