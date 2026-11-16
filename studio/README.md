# studio

An activation that holds others. `studio.kiln` is the kiln, held in this process (`#[child] fn kiln(&self) -> &Kiln`); `studio.glaze` is the glaze activation, held across a channel (`#[child] fn glaze(&self) -> Box<dyn Activation>`): the `glaze` binary, started as the studio's child and spoken to over its stdin and stdout (`plexus::Remote::child`). A caller cannot tell the two apart. `main` is one line.

Native only: a component holds no children yet.

```sh
cargo build -p studio -p glaze --bins                  # glaze is found beside the studio, or where STUDIO_GLAZE says
cargo run -p studio -- --ws 127.0.0.1:0                # no flag: stdin and stdout
cargo test -p studio                                   # tests/held.rs: each way of holding, in memory; tests/process.rs: the process, glaze killed under it
```

When the glaze process is gone, `studio.glaze.*` fails `unavailable`; the studio and its kiln answer as before.

Its IR is not kept in `../ir` beside the others: `plexus-gen`'s corpus test counts the IRs it reads, so it joins when that count does.
