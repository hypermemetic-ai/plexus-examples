// Generated from the IR of `studio` (interface f308dc639d213ac1). Not edited by hand.

import { Connection, Server, each } from "./plexus-rt.ts";

export type NotFound =
  | { tag: "no-such"; val: string };

const toNotFound = (v: NotFound): any => v.tag === "no-such" ? { "no-such": v.val } : (v as any).tag;
const fromNotFound = (j: any): NotFound => typeof j === "string" ? { tag: j } as NotFound : "no-such" in j ? { tag: "no-such", val: j["no-such"] } : j;

/// The IR this file was generated from: what `_ir` answers.
export const studioIr = {"ir": "plexus/0", "activation": "studio", "hash": "f308dc639d213ac1", "doc": "The four ways an activation holds another, by what is known when the\nparent is compiled.", "types": [{"name": "not-found", "variant": [{"name": "no-such", "type": "string"}]}], "methods": [{"name": "shelves", "params": [], "returns": {"list": "u32"}}, {"name": "suppliers", "params": [], "returns": {"list": "string"}}, {"name": "plugins", "params": [], "returns": {"list": "string"}}], "children": [{"name": "kiln", "activation": "kiln", "doc": "1. Known, and always there. The compiler checks every call."}, {"name": "shelf", "activation": "kiln", "key": {"name": "number", "type": "u32"}, "list": "shelves", "fails": "not-found", "doc": "2. Known, one per key. Which keys exist is asked while running."}, {"name": "supplier", "interface": "glazes", "key": {"name": "name", "type": "string"}, "list": "suppliers", "fails": "not-found", "doc": "3. The interface is known; which activation fills it is not. It\nmay be here or on another machine. Checked once, when it is\nconnected, by its hash."}, {"name": "plugin", "any": true, "key": {"name": "name", "type": "string"}, "list": "plugins", "fails": "not-found", "doc": "4. Nothing is known. Its IR is read while running, and it is\ncalled by name, with JSON."}]};

/// The caller's side of `studio`: every method, typed, sent as frames.
export class StudioClient {
  /// The interface this client was generated from.
  static readonly interface = "f308dc639d213ac1";
  private conn: Connection;
  private at: string;
  /// `at` is where the activation is mounted: its own name unless a parent holds it.
  constructor(conn: Connection, at: string = "studio") {
    this.conn = conn;
    this.at = at;
  }
  /// Ask what is served at `at`, once, and refuse if it is not this interface.
  async bind(): Promise<this> {
    const served = await this.conn.call(this.at + "._hash", {});
    if (served !== StudioClient.interface) throw new Error(`${this.at} serves interface ${served}; this client was generated from ${StudioClient.interface}`);
    return this;
  }
  async shelves(): Promise<Array<number>> {
    const j = await this.conn.call(this.at + ".shelves", {  });
    return j;
  }
  async suppliers(): Promise<Array<string>> {
    const j = await this.conn.call(this.at + ".suppliers", {  });
    return j;
  }
  async plugins(): Promise<Array<string>> {
    const j = await this.conn.call(this.at + ".plugins", {  });
    return j;
  }
}

/// What `studio` must offer to be served: a transpiled component does.
export interface StudioActivation {
  shelves(): Array<number> | Promise<Array<number>>;
  suppliers(): Array<string> | Promise<Array<string>>;
  plugins(): Array<string> | Promise<Array<string>>;
}

/// The activation's side: each `open` frame becomes a call into `activation`.
/// `at` is where it is mounted: its own name unless a parent holds it.
export function serveStudio(activation: StudioActivation, server: Server, at: string = "studio"): void {
  server.method(at + ".shelves", (p: any) => activation.shelves());
  server.method(at + ".suppliers", (p: any) => activation.suppliers());
  server.method(at + ".plugins", (p: any) => activation.plugins());
  server.describe(at, studioIr);
}
