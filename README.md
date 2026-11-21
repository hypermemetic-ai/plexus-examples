# plexus-examples

Small activations written with `#[plexus::activation]`. Every part of Plexus v2 is proven on them.

| | |
|---|---|
| `kiln` | A kiln and its firing log: an `async fn`, an `impl Stream`, a typed error; the log kept in SQLite through `plexus::Store`. Builds to a wasm component, and natively to the `kiln` process. |
| `glaze` | Glaze recipes, kept in a `plexus::Store` of their own: a second activation, held beside the kiln. Builds to a wasm component, and natively to the `glaze` process. |
| `studio` | An activation that holds others: the kiln in its process (`&Kiln`), the glaze activation across a channel (`Box<dyn Activation>`), the `glaze` binary started as its child. Native only. |
| `showcase` | One small activation per thing the IR can say. The input-stream fixture runs as a native activation. |
| `grant` | A sketch of who is calling and what they may call. An activation is opened with a context, a struct whose fields the server fills or the caller sends as keys of the path (`studio[tenant=acme].kiln[3].fire`); opening answers the application's own proof, remembered per connection, and each method takes it. Roles are expressions over methods (`studio.kiln.** && group:read`) compiled to bitsets, grants are by method alone, and a delegation only narrows. Plain Rust, no Plexus dependency; the studio is called by hand as `#[activation]` would. |

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
cargo test -p grant                        # the grant sketch: paths with keys, a context, openings, roles, grants, delegation
```

### Stream a file into a native activation

The `media-input` example serves the same `studio.media` address in both modes:

```sh
cargo run --locked -p showcase --example media-input -- local
cargo run --locked -p showcase --example media-input -- remote
```

Each prints the WebSocket address it bound. In remote mode the server holds a second media process over stdio. The generated client in `ts/f08_stream_in.plexus.ts` works with either address:

```ts
const media = await new MediaClient(conn, "studio.media").bind()
const kept = await media.put(file.name, file.stream(), abortController.signal)
console.log(kept.bytes, kept.sha256)
```

The native handler hashes and writes each chunk to a temporary file as it arrives. A completed upload replaces the file kept under that name for the lifetime of the activation. Failed or cancelled uploads release their temporary files. `npm run test:input` in the sibling `plexus-rt` checkout exercises both mounts in Chromium.

Every example's IR is kept in `ir/`: a showcase file as `ir/<file>.json`, kiln and glaze as `ir/kiln.json` and `ir/glaze.json`. plexus-macros' `goldens` test fails, showing the difference, when the macro writes anything else; plexus-ir reads and writes each back. To keep what the macro writes now, from `../plexus-macros`:

```sh
BLESS=1 cargo test goldens
```

Each IR's TypeScript and WIT, as `plexus-gen` writes them from the kept IR, are in `ts/<name>.plexus.ts` and `wit/<name>.wit`. plexus-gen's `goldens` test fails, showing the difference, when it writes anything else; to keep what it writes now, from `../plexus-gen`:

```sh
BLESS=1 cargo test goldens
```

The Plexus crates are expected beside this checkout: `../plexus`, `../plexus-macros`, `../plexus-ir`. Tracker: LAYERS-9.
