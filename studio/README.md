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

## A tenant gate with a local or TLS kiln

`studio-kiln` is a standalone example with an authenticated Studio listener.
Studio opens the `acme` tenant for Alice. Both Alice and Bob can connect, and
Studio applies that membership check before dispatching a call to its kiln.
One startup choice serves the kiln directly in Studio's process. The other
mounts a separate process over WebSocket with mutual Ed25519 TLS pins.
Studio composes the child at startup. Discovery lists it in `_tree` and
serves its own schema at `studio[tenant=acme].kiln._ir`.
The client uses the same address and parameters in both modes.

```text
studio[tenant=acme].kiln.heat {"cone":6,"hours":1.0}
```

Run the end-to-end example from the workspace root on Unix.

```sh
cargo test -p studio --features tls --test tls_kiln
```

The test launches real processes on ephemeral loopback ports and uses
disposable Ed25519 identities. It compares all 13 temperature readings and
the kiln's discovery hash across both modes, checks that Bob's denial leaves
the kiln's invocation counter at zero, and observes the kiln drop an unfinished
stream when Alice cancels it. The local kiln records Alice's authenticated
principal. The remote kiln records Studio's authenticated principal, which
the test reads from the separate kiln process. A wrong kiln pin prevents
Studio from opening its listener.

To start the services manually, build the example and create key files in a
private temporary directory. Each invocation prints its loopback WSS URL.

```sh
studio_kiln() {
  cargo run --quiet -p studio --features tls --bin studio-kiln -- "$@"
}
keys=$(mktemp -d)
umask 077
for name in studio kiln alice bob; do
  openssl rand -out "$keys/$name.key" 32
done
studio_pin=$(studio_kiln principal "$keys/studio.key")
kiln_pin=$(studio_kiln principal "$keys/kiln.key")
alice_pin=$(studio_kiln principal "$keys/alice.key")
bob_pin=$(studio_kiln principal "$keys/bob.key")
```

For a local kiln, start Studio on its own.

```sh
studio_kiln studio "$keys/studio.key" "$alice_pin" "$bob_pin"
```

For a remote kiln, start the kiln in one terminal and use the URL it prints
as `KILN_URL` when starting Studio. Use the same shell function, key
directory and public pins in both terminals.

```sh
studio_kiln --wss 127.0.0.1:0 --key-file "$keys/kiln.key" --allow-key "$studio_pin"
studio_kiln studio "$keys/studio.key" "$alice_pin" "$bob_pin" "$KILN_URL" "$kiln_pin"
```

To connect a Rust client, load the chosen user key with
`plexus::tls::Identity::from_key_file`, pass that identity and `studio_pin` to
`plexus::tls::Dial::new`, and call `dial.connect(studio_url)`. Use the returned
channel to create a `plexus::Connection` and spawn its driver, then call
`connection.stream("studio[tenant=acme].kiln.heat", params)`. The test is a
complete client for both modes.
