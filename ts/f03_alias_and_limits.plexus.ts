// Generated from the IR of `tickets` (interface b92b392f97cce81a). Not edited by hand.

import { Connection, Server, each } from "./plexus-rt.ts";

/// A ticket's id: `KILN-2`.
export type TicketId = string;
/// 1 is lowest, 5 is drop everything.
export type Priority = number;

const toTicketId = (v: TicketId): any => v;
const fromTicketId = (j: any): TicketId => j;
const toPriority = (v: Priority): any => v;
const fromPriority = (j: any): Priority => j;

/// The IR this file was generated from: what `_ir` answers.
export const ticketsIr = {"ir": "plexus/0", "activation": "tickets", "hash": "b92b392f97cce81a", "doc": "An alias names a type. Its limits are checked before the method runs.", "types": [{"name": "ticket-id", "doc": "A ticket's id: `KILN-2`.", "alias": "string", "limits": {"min-len": 3, "max-len": 64, "pattern": "^[A-Z]+-[0-9]+$"}}, {"name": "priority", "doc": "1 is lowest, 5 is drop everything.", "alias": "u8", "limits": {"min": 1, "max": 5}}], "methods": [{"name": "priority-of", "params": [{"name": "id", "type": "ticket-id"}], "returns": "priority"}]};

/// The caller's side of `tickets`: every method, typed, sent as frames.
export class TicketsClient {
  /// The interface this client was generated from.
  static readonly interface = "b92b392f97cce81a";
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
  async priorityOf(id: TicketId): Promise<Priority> {
    const j = await this.conn.call(this.at + ".priority-of", { "id": toTicketId(id) });
    return fromPriority(j);
  }
}

/// What `tickets` must offer to be served: a transpiled component does.
export interface TicketsActivation {
  priorityOf(id: TicketId): Priority | Promise<Priority>;
}

/// The activation's side: each `open` frame becomes a call into `activation`.
/// `at` is where it is mounted: its own name unless a parent holds it.
export function serveTickets(activation: TicketsActivation, server: Server, at: string = "tickets"): void {
  server.method(at + ".priority-of", async (p: any) => { const v = await activation.priorityOf(fromTicketId(p["id"])); return toPriority(v); });
  server.describe(at, ticketsIr);
}
