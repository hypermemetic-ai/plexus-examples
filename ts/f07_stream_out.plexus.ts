// Generated from the IR of `tickets` (interface 1bbc1163a752a95b). Not edited by hand.

import { Connection, Server, each } from "./plexus-rt.ts";

export interface Change { id: string; status: string }

const toChange = (v: Change): any => ({ "id": v.id, "status": v.status });
const fromChange = (j: any): Change => ({ id: j["id"], status: j["status"] });

/// The IR this file was generated from: what `_ir` answers.
export const ticketsIr = {"ir": "plexus/0", "activation": "tickets", "hash": "1bbc1163a752a95b", "doc": "A stream out: many items, one at a time, until there are no more.\n`impl Stream` may wait between items; `impl Iterator` is accepted for\none that never does.", "types": [{"name": "change", "record": [{"name": "id", "type": "string"}, {"name": "status", "type": "string"}]}], "methods": [{"name": "watch", "doc": "Every change, as it happens.", "params": [{"name": "under", "type": "string"}], "stream": "change"}, {"name": "ids", "doc": "Every id under a ticket: nothing to wait for.", "params": [{"name": "under", "type": "string"}], "stream": "string"}]};

/// The caller's side of `tickets`: every method, typed, sent as frames.
export class TicketsClient {
  /// The interface this client was generated from.
  static readonly interface = "1bbc1163a752a95b";
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
  /// Every change, as it happens.
  watch(under: string): AsyncIterable<Change> {
    return each(this.conn.stream(this.at + ".watch", { "under": under }), (j: any) => fromChange(j));
  }
  /// Every id under a ticket: nothing to wait for.
  ids(under: string): AsyncIterable<string> {
    return each(this.conn.stream(this.at + ".ids", { "under": under }), (j: any) => j);
  }
}

/// What `tickets` must offer to be served: a transpiled component does.
export interface TicketsActivation {
  watch(under: string): AsyncIterable<Change> | Promise<AsyncIterable<Change>>;
  ids(under: string): AsyncIterable<string> | Promise<AsyncIterable<string>>;
}

/// The activation's side: each `open` frame becomes a call into `activation`.
/// `at` is where it is mounted: its own name unless a parent holds it.
export function serveTickets(activation: TicketsActivation, server: Server, at: string = "tickets"): void {
  server.stream(at + ".watch", (p: any) => each(activation.watch(p["under"]), (v: any) => toChange(v)));
  server.stream(at + ".ids", (p: any) => each(activation.ids(p["under"]), (v: any) => v));
  server.describe(at, ticketsIr);
}
