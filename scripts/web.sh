#!/bin/sh
# Build for a page — wasm32, the web profile — in a container, for what
# compiles C and so needs clang: the engine that holds SQLite,
#
#   scripts/web.sh --manifest-path ../plexus/engine/Cargo.toml
#
# (an activation that stores imports plexus:store/sql and needs none of it).
# Arguments go to `cargo build --lib`. The image, plexus-web-build, is
# built on first use. The sibling checkouts (../plexus, …) are mounted at
# their own paths, and remapped in what is built (/src, its cargo /cargo), so
# no path of this machine is in the wasm; the container's cargo, a newer one, builds into
# $CARGO_TARGET_DIR/web-container (default target/web-container), and the
# directory holding the .wasm files is printed.
set -eu
here=$(cd "$(dirname "$0")/.." && pwd)
root=$(dirname "$here")
target=${CARGO_TARGET_DIR:-$here/target}/web-container
registry=${CARGO_HOME:-$HOME/.cargo}/registry
image=plexus-web-build
home=$target/.cargo-home   # the container's CARGO_HOME, writable by this user
mkdir -p "$home/registry" "$registry"
docker image inspect "$image" >/dev/null 2>&1 || docker build -q -t "$image" - >&2 <<'DOCKERFILE'
FROM rust:1-bookworm
RUN apt-get update && apt-get install -y --no-install-recommends clang && rm -rf /var/lib/apt/lists/* \
 && rustup target add wasm32-unknown-unknown
DOCKERFILE
# SELinux is enforcing on the house's machines: a bind mount is unreadable
# in a container without label=disable.
docker run --rm --security-opt label=disable --user "$(id -u):$(id -g)" \
  -v "$root:$root" -v "$target:$target" -v "$registry:$home/registry" \
  -e CARGO_HOME="$home" -e CARGO_TARGET_DIR="$target" -w "$here" \
  "$image" cargo build -q --lib --profile web --target wasm32-unknown-unknown \
  --config "target.wasm32-unknown-unknown.rustflags=[\"--remap-path-prefix=$home=/cargo\", \"--remap-path-prefix=$root=/src\"]" "$@" >&2
echo "$target/wasm32-unknown-unknown/web"
