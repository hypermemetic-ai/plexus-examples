// Generated from the IR of `tickets` (interface 06d54522f2da0ae8). Not edited by hand.

import { Connection, Server, each } from "./plexus-rt.ts";

/// One ticket.
export interface Ticket { id: string; title: string; open: boolean }

const toTicket = (v: Ticket): any => ({ "id": v.id, "title": v.title, "open": v.open });
const fromTicket = (j: any): Ticket => ({ id: j["id"], title: j["title"], open: j["open"] });

/// The IR this file was generated from: what `_ir` answers.
export const ticketsIr = {"ir": "plexus/0", "activation": "tickets", "hash": "06d54522f2da0ae8", "doc": "A record: named fields.", "types": [{"name": "ticket", "doc": "One ticket.", "record": [{"name": "id", "type": "string"}, {"name": "title", "type": "string", "doc": "What it is about."}, {"name": "open", "type": "bool"}]}], "methods": [{"name": "newest", "doc": "The newest ticket.", "params": [], "returns": "ticket"}]};

/// The caller's side of `tickets`: every method, typed, sent as frames.
export class TicketsClient {
  /// The interface this client was generated from.
  static readonly interface = "06d54522f2da0ae8";
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
  /// The newest ticket.
  async newest(): Promise<Ticket> {
    const j = await this.conn.call(this.at + ".newest", {  });
    return fromTicket(j);
  }
}

/// What `tickets` must offer to be served: a transpiled component does.
export interface TicketsActivation {
  newest(): Ticket | Promise<Ticket>;
}

/// The activation's side: each `open` frame becomes a call into `activation`.
/// `at` is where it is mounted: its own name unless a parent holds it.
export function serveTickets(activation: TicketsActivation, server: Server, at: string = "tickets"): void {
  server.method(at + ".newest", async (p: any) => { const v = await activation.newest(); return toTicket(v); });
  server.describe(at, ticketsIr);
}
