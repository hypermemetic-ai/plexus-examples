// Generated from the IR of `tickets` (interface 502d5253ad9b5599). Not edited by hand.

import { Connection, Server, each } from "./plexus-rt.ts";

export type NotFound =
  | { tag: "no-such-project"; val: string };

const toNotFound = (v: NotFound): any => v.tag === "no-such-project" ? { "no-such-project": v.val } : (v as any).tag;
const fromNotFound = (j: any): NotFound => typeof j === "string" ? { tag: j } as NotFound : "no-such-project" in j ? { tag: "no-such-project", val: j["no-such-project"] } : j;

/// The IR this file was generated from: what `_ir` answers.
export const ticketsIr = {"ir": "plexus/0", "activation": "tickets", "hash": "502d5253ad9b5599", "doc": "Children: activations held under this one. One that is always there, and\none per key, with the method that lists the keys.", "types": [{"name": "not-found", "variant": [{"name": "no-such-project", "type": "string"}]}], "methods": [{"name": "projects", "doc": "The names `project` answers to.", "params": [], "returns": {"list": "string"}}], "children": [{"name": "commits", "activation": "commits", "doc": "Always there: `tickets.commits.*`"}, {"name": "project", "activation": "project", "key": {"name": "name", "type": "string"}, "list": "projects", "fails": "not-found", "doc": "One per name: `tickets.project.<name>.*`"}]};

/// The caller's side of `tickets`: every method, typed, sent as frames.
export class TicketsClient {
  /// The interface this client was generated from.
  static readonly interface = "502d5253ad9b5599";
  private conn: Connection;
  private at: string;
  /// `at` is where the activation is mounted: its own name unless a parent holds it.
  constructor(conn: Connection, at: string = "tickets") {
    this.conn = conn;
    this.at = at;
  }
  /// Ask what is served at `at`, once, and refuse if it is not this interface.
  async bind(): Promise<this> {
    const served = await this.conn.call(this.at + "._hash", {});
    if (served !== TicketsClient.interface) throw new Error(`${this.at} serves interface ${served}; this client was generated from ${TicketsClient.interface}`);
    return this;
  }
  /// The names `project` answers to.
  async projects(): Promise<Array<string>> {
    const j = await this.conn.call(this.at + ".projects", {  });
    return j;
  }
}

/// What `tickets` must offer to be served: a transpiled component does.
export interface TicketsActivation {
  projects(): Array<string> | Promise<Array<string>>;
}

/// The activation's side: each `open` frame becomes a call into `activation`.
/// `at` is where it is mounted: its own name unless a parent holds it.
export function serveTickets(activation: TicketsActivation, server: Server, at: string = "tickets"): void {
  server.method(at + ".projects", (p: any) => activation.projects());
  server.describe(at, ticketsIr);
}
