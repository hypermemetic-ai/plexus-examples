// Generated from the IR of `tickets` (interface 61fb6bd3d5cf344a). Not edited by hand.

import { Connection, Server, each } from "./plexus-rt.ts";

export interface Ticket { id: string; title: string }

const toTicket = (v: Ticket): any => ({ "id": v.id, "title": v.title });
const fromTicket = (j: any): Ticket => ({ id: j["id"], title: j["title"] });

/// The IR this file was generated from: what `_ir` answers.
export const ticketsIr = {"ir": "plexus/0", "activation": "tickets", "hash": "61fb6bd3d5cf344a", "doc": "An `async fn` may wait: for a disk, for another activation. The IR does not\nsay which methods wait, because over a wire every call is awaited. WIT\ndoes: a caller in the same process gets a promise, or a plain value.", "types": [{"name": "ticket", "record": [{"name": "id", "type": "string"}, {"name": "title", "type": "string"}]}], "methods": [{"name": "get", "doc": "Waits for the store.", "async": true, "params": [{"name": "id", "type": "string"}], "returns": {"option": "ticket"}}, {"name": "count", "doc": "Answers at once.", "params": [], "returns": "u32"}]};

/// The caller's side of `tickets`: every method, typed, sent as frames.
export class TicketsClient {
  /// The interface this client was generated from.
  static readonly interface = "61fb6bd3d5cf344a";
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
  /// Waits for the store.
  async get(id: string): Promise<Ticket | undefined> {
    const j = await this.conn.call(this.at + ".get", { "id": id });
    return (j == null ? undefined : fromTicket(j));
  }
  /// Answers at once.
  async count(): Promise<number> {
    const j = await this.conn.call(this.at + ".count", {  });
    return j;
  }
}

/// What `tickets` must offer to be served: a transpiled component does.
export interface TicketsActivation {
  get(id: string): (Ticket | undefined) | Promise<(Ticket | undefined)>;
  count(): number | Promise<number>;
}

/// The activation's side: each `open` frame becomes a call into `activation`.
/// `at` is where it is mounted: its own name unless a parent holds it.
export function serveTickets(activation: TicketsActivation, server: Server, at: string = "tickets"): void {
  server.method(at + ".get", async (p: any) => { const v = await activation.get(p["id"]); return (v == null ? undefined : toTicket(v)); });
  server.method(at + ".count", (p: any) => activation.count());
  server.describe(at, ticketsIr);
}
