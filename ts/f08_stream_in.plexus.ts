// Generated from the IR of `media` (interface 98505b970aaba5a1). Not edited by hand.

import { Connection, Server, each, b64, unb64 } from "./plexus-rt.ts";

export interface Kept { sha256: string; bytes: bigint }

const toKept = (v: Kept): any => ({ "sha256": v.sha256, "bytes": Number(v.bytes) });
const fromKept = (j: any): Kept => ({ sha256: j["sha256"], bytes: BigInt(j["bytes"]) });

/// The IR this file was generated from: what `_ir` answers.
export const mediaIr = {"ir": "plexus/0", "activation": "media", "hash": "98505b970aaba5a1", "doc": "A stream in: `Input<T>` is what the caller sends while the call runs.\nIt is not one of the parameters.", "types": [{"name": "kept", "record": [{"name": "sha256", "type": "string"}, {"name": "bytes", "type": "u64"}]}], "methods": [{"name": "put", "doc": "Keep a file. The caller streams its bytes in.", "params": [{"name": "name", "type": "string"}], "input": "bytes", "returns": "kept"}, {"name": "tokens", "doc": "Count a run's tokens: a stream in of a type of its own.", "params": [{"name": "run", "type": "string"}], "input": "string", "returns": "u32"}]};

/// The caller's side of `media`: every method, typed, sent as frames.
export class MediaClient {
  /// The interface this client was generated from.
  static readonly interface = "98505b970aaba5a1";
  private conn: Connection;
  private at: string;
  /// `at` is where the activation is mounted: its own name unless a parent holds it.
  constructor(conn: Connection, at: string = "media") {
    this.conn = conn;
    this.at = at;
  }
  /// Ask what is served at `at`, once, and refuse if it is not this interface.
  async bind(): Promise<this> {
    const served = await this.conn.call(this.at + "._hash", {});
    if (served !== MediaClient.interface) throw new Error(`${this.at} serves interface ${served}; this client was generated from ${MediaClient.interface}`);
    return this;
  }
  /// Keep a file. The caller streams its bytes in.
  async put(name: string): Promise<Kept> {
    const j = await this.conn.call(this.at + ".put", { "name": name });
    return fromKept(j);
  }
  /// Count a run's tokens: a stream in of a type of its own.
  async tokens(run: string): Promise<number> {
    const j = await this.conn.call(this.at + ".tokens", { "run": run });
    return j;
  }
}

/// What `media` must offer to be served: a transpiled component does.
export interface MediaActivation {
  put(name: string): Kept | Promise<Kept>;
  tokens(run: string): number | Promise<number>;
}

/// The activation's side: each `open` frame becomes a call into `activation`.
/// `at` is where it is mounted: its own name unless a parent holds it.
export function serveMedia(activation: MediaActivation, server: Server, at: string = "media"): void {
  server.method(at + ".put", async (p: any) => { const v = await activation.put(p["name"]); return toKept(v); });
  server.method(at + ".tokens", (p: any) => activation.tokens(p["run"]));
  server.describe(at, mediaIr);
}
