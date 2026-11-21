// Generated from the IR of `media` (interface 1ce984f4f99c1ba2). Not edited by hand.

import { Connection, Server, each, b64, unb64 } from "./plexus-rt.ts";

export interface Kept { sha256: string; bytes: bigint }
export interface UploadStatus { received: bigint; active: number }
export type UploadError =
  | { tag: "invalid-input"; val: string }
  | { tag: "storage"; val: string };

const toKept = (v: Kept): any => ({ "sha256": v.sha256, "bytes": Number(v.bytes) });
const fromKept = (j: any): Kept => ({ sha256: j["sha256"], bytes: BigInt(j["bytes"]) });
const toUploadStatus = (v: UploadStatus): any => ({ "received": Number(v.received), "active": v.active });
const fromUploadStatus = (j: any): UploadStatus => ({ received: BigInt(j["received"]), active: j["active"] });
const toUploadError = (v: UploadError): any => v.tag === "invalid-input" ? { "invalid-input": v.val } : v.tag === "storage" ? { "storage": v.val } : (v as any).tag;
const fromUploadError = (j: any): UploadError => typeof j === "string" ? { tag: j } as UploadError : "invalid-input" in j ? { tag: "invalid-input", val: j["invalid-input"] } : "storage" in j ? { tag: "storage", val: j["storage"] } : j;

/// The IR this file was generated from: what `_ir` answers.
export const mediaIr = {"ir": "plexus/0", "activation": "media", "hash": "1ce984f4f99c1ba2", "doc": "A stream in: `Input<T>` is what the caller sends while the call runs.\nIt is not one of the parameters.", "types": [{"name": "kept", "record": [{"name": "sha256", "type": "string"}, {"name": "bytes", "type": "u64"}]}, {"name": "upload-status", "record": [{"name": "received", "type": "u64"}, {"name": "active", "type": "u32"}]}, {"name": "upload-error", "variant": [{"name": "invalid-input", "type": "string"}, {"name": "storage", "type": "string"}]}], "methods": [{"name": "put", "doc": "Keep a file. The caller streams its bytes in.", "async": true, "params": [{"name": "name", "type": "string"}], "input": "bytes", "returns": "kept", "fails": "upload-error"}, {"name": "tokens", "doc": "Count a run's tokens: a stream in of a type of its own.", "async": true, "params": [{"name": "run", "type": "string"}], "input": "string", "returns": "u32", "fails": "upload-error"}, {"name": "status", "doc": "Active uploads and bytes read so far, including unfinished uploads.", "params": [], "returns": "upload-status"}, {"name": "receipt", "doc": "The receipt of the last completed upload to this name.", "params": [{"name": "name", "type": "string"}], "returns": {"option": "kept"}}]};

/// The caller's side of `media`: every method, typed, sent as frames.
export class MediaClient {
  /// The interface this client was generated from.
  static readonly interface = "1ce984f4f99c1ba2";
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
  async put(name: string, chunks: AsyncIterable<Uint8Array>, signal?: AbortSignal): Promise<Kept> {
    const j = await this.conn.call(this.at + ".put", { "name": name }, (e: any) => fromUploadError(e), each(chunks, (v) => b64(v)), signal);
    return fromKept(j);
  }
  /// Count a run's tokens: a stream in of a type of its own.
  async tokens(run: string, chunks: AsyncIterable<string>, signal?: AbortSignal): Promise<number> {
    const j = await this.conn.call(this.at + ".tokens", { "run": run }, (e: any) => fromUploadError(e), each(chunks, (v) => v), signal);
    return j;
  }
  /// Active uploads and bytes read so far, including unfinished uploads.
  async status(): Promise<UploadStatus> {
    const j = await this.conn.call(this.at + ".status", {  });
    return fromUploadStatus(j);
  }
  /// The receipt of the last completed upload to this name.
  async receipt(name: string): Promise<Kept | undefined> {
    const j = await this.conn.call(this.at + ".receipt", { "name": name });
    return (j == null ? undefined : fromKept(j));
  }
}

/// What `media` must offer to be served: a transpiled component does.
export interface MediaActivation {
  put(name: string, chunks: AsyncIterable<Uint8Array>): Kept | Promise<Kept>;
  tokens(run: string, chunks: AsyncIterable<string>): number | Promise<number>;
  status(): UploadStatus | Promise<UploadStatus>;
  receipt(name: string): (Kept | undefined) | Promise<(Kept | undefined)>;
}

/// The activation's side: each `open` frame becomes a call into `activation`.
/// `at` is where it is mounted: its own name unless a parent holds it.
export function serveMedia(activation: MediaActivation, server: Server, at: string = "media"): void {
  server.methodWithInput(at + ".put", async (p: any, input) => { const v = await activation.put(p["name"], each(input, (j: any) => unb64(j))); return toKept(v); }, (e: any) => toUploadError(e));
  server.methodWithInput(at + ".tokens", (p: any, input) => activation.tokens(p["run"], each(input, (j: any) => j)), (e: any) => toUploadError(e));
  server.method(at + ".status", async (p: any) => { const v = await activation.status(); return toUploadStatus(v); });
  server.method(at + ".receipt", async (p: any) => { const v = await activation.receipt(p["name"]); return (v == null ? undefined : toKept(v)); });
  server.describe(at, mediaIr);
}
