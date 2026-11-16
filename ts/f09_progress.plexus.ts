// Generated from the IR of `export` (interface bbfb7901dbd4100f). Not edited by hand.

import { Connection, Server, each } from "./plexus-rt.ts";

export interface Step { done: number; of: number }

const toStep = (v: Step): any => ({ "done": v.done, "of": v.of });
const fromStep = (j: any): Step => ({ done: j["done"], of: j["of"] });

/// The IR this file was generated from: what `_ir` answers.
export const exportIr = {"ir": "plexus/0", "activation": "export", "hash": "bbfb7901dbd4100f", "doc": "Progress: what a method reports while the caller waits. Never part of\nthe answer, and not a parameter.", "types": [{"name": "step", "record": [{"name": "done", "type": "u32"}, {"name": "of", "type": "u32"}]}], "methods": [{"name": "archive", "doc": "Write every ticket to an archive; says how far along it is.", "params": [{"name": "under", "type": "string"}], "progress": "step", "returns": "string"}]};

/// The caller's side of `export`: every method, typed, sent as frames.
export class ExportClient {
  /// The interface this client was generated from.
  static readonly interface = "bbfb7901dbd4100f";
  private conn: Connection;
  private at: string;
  /// `at` is where the activation is mounted: its own name unless a parent holds it.
  constructor(conn: Connection, at: string = "export") {
    this.conn = conn;
    this.at = at;
  }
  /// Ask what is served at `at`, once, and refuse if it is not this interface.
  async bind(): Promise<this> {
    const served = await this.conn.call(this.at + "._hash", {});
    if (served !== ExportClient.interface) throw new Error(`${this.at} serves interface ${served}; this client was generated from ${ExportClient.interface}`);
    return this;
  }
  /// Write every ticket to an archive; says how far along it is.
  async archive(under: string): Promise<string> {
    const j = await this.conn.call(this.at + ".archive", { "under": under });
    return j;
  }
}

/// What `export` must offer to be served: a transpiled component does.
export interface ExportActivation {
  archive(under: string): string | Promise<string>;
}

/// The activation's side: each `open` frame becomes a call into `activation`.
/// `at` is where it is mounted: its own name unless a parent holds it.
export function serveExport(activation: ExportActivation, server: Server, at: string = "export"): void {
  server.method(at + ".archive", (p: any) => activation.archive(p["under"]));
  server.describe(at, exportIr);
}
