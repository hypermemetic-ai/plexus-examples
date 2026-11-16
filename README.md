# plexus-examples

Small activations written with `#[plexus::activation]`. Every part of Plexus v2 is proven on them.

| | |
|---|---|
| `kiln` | A kiln and its firing log: an `async fn`, an `impl Stream`, a typed error; the log kept in SQLite through `plexus::Store`. Builds to a wasm component, and natively to the `kiln` process. |
| `glaze` | Glaze recipes: a second activation, held beside the kiln. Builds to a wasm component, and natively to the `glaze` process. |
| `showcase` | One small activation per thing the IR can say. Each compiles; none is served. |

```sh
cargo build -p showcase
scripts/web.sh -p kiln -p glaze           # the components (wasm32, the web profile) in a container: SQLite to wasm needs clang;
                                           # prints the directory of the .wasm files
cargo run -p kiln -- --ws 127.0.0.1:0      # the kiln as a process: `main` is one line, plexus::main; no flag serves stdin and stdout
cargo run -p kiln -- --data DIR            # its log in DIR/kiln.db, kept across a restart (no --data: in memory, said on stderr)
cargo test -p kiln -p glaze                # each process over stdio and a WebSocket, the kiln's log across a restart; the generated TypeScript client against it in Node
                                           # (ignored, with why, without Node 22.13+ or ../plexus-rt/dist: npm run build there)
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
