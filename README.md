# plexus-examples

Small activations written with `#[plexus::activation]`. Every part of Plexus v2 is proven on them.

| | |
|---|---|
| `kiln` | A kiln and its firing log: an `async fn`, an `impl Stream`, a typed error; the log kept in SQLite through `plexus::Store`. Builds to a wasm component, and natively to the `kiln` process. |
| `glaze` | Glaze recipes, kept in a `plexus::Store` of their own: a second activation, held beside the kiln. Builds to a wasm component, and natively to the `glaze` process. |
| `studio` | An activation that holds others: the kiln in its process (`&Kiln`), the glaze activation across a channel (`Box<dyn Activation>`), the `glaze` binary started as its child. Native only. |
| `showcase` | One small activation per thing the IR can say. Each compiles; none is served. |

```sh
cargo build -p showcase
cargo build --lib --profile web --target wasm32-unknown-unknown -p kiln -p glaze
                                           # the components: their store is the import plexus:store/sql, so plain cargo, no clang
scripts/web.sh --manifest-path ../plexus/engine/Cargo.toml
                                           # what still needs clang (SQLite's C), in a container: the one engine
                                           # (../plexus-rt/scripts/engine.sh runs this); prints the directory of the .wasm files
cargo run -p kiln -- --ws 127.0.0.1:0      # the kiln as a process: `main` is one line, plexus::main; no flag serves stdin and stdout
cargo run -p kiln -- --data DIR            # its log in DIR/kiln/kiln.db (the directory of the place it is served at), kept across a restart (no --data: in memory, said on stderr)
cargo test -p kiln -p glaze                # each process over stdio and a WebSocket, the kiln's log across a restart; the generated TypeScript client against it in Node
                                           # (ignored, with why, without Node 22.13+ or ../plexus-rt/dist: npm run build there)
cargo build -p studio -p glaze --bins && cargo run -p studio -- --ws 127.0.0.1:0   # the studio, which starts glaze beside it
cargo test -p studio                       # each way of holding, in memory; the studio process and its glaze child
```

Every example's IR is kept in `ir/`: a showcase file as `ir/<file>.json`, kiln and glaze as `ir/kiln.json` and `ir/glaze.json`. plexus-macros' `goldens` test fails, showing the difference, when the macro writes anything else; plexus-ir reads and writes each back. To keep what the macro writes now, from `../plexus-macros`:

```sh
BLESS=1 cargo test goldens
```

Each IR's TypeScript and WIT, as `plexus-gen` writes them from the kept IR, are in `ts/<name>.plexus.ts` and `wit/<name>.wit`. plexus-gen's `goldens` test fails, showing the difference, when it writes anything else; to keep what it writes now, from `../plexus-gen`:

```sh
BLESS=1 cargo test goldens
```

The Plexus crates are expected beside this checkout: `../plexus`, `../plexus-macros`, `../plexus-ir`. Tracker: LAYERS-9.
