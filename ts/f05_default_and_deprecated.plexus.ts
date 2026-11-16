// Generated from the IR of `tickets` (interface 2400600f8a1dd2ee). Not edited by hand.

import { Connection, Server, each } from "./plexus-rt.ts";

export interface Page { ids: Array<string>; more: boolean }

const toPage = (v: Page): any => ({ "ids": v.ids, "more": v.more });
const fromPage = (j: any): Page => ({ ids: j["ids"], more: j["more"] });

/// The IR this file was generated from: what `_ir` answers.
export const ticketsIr = {"ir": "plexus/0", "activation": "tickets", "hash": "2400600f8a1dd2ee", "doc": "A default is what an absent value is taken as. A deprecated parameter\nstill works, and says what to use.", "types": [{"name": "page", "record": [{"name": "ids", "type": {"list": "string"}}, {"name": "more", "type": "bool", "default": false}]}], "methods": [{"name": "list", "params": [{"name": "limit", "type": "u32", "default": 50}, {"name": "after", "type": "string", "default": ""}, {"name": "count", "type": {"option": "u32"}, "deprecated": {"since": "0.3", "note": "use `limit`"}}], "returns": "page"}]};

/// The caller's side of `tickets`: every method, typed, sent as frames.
export class TicketsClient {
  /// The interface this client was generated from.
  static readonly interface = "2400600f8a1dd2ee";
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
  async list(limit: number, after: string, count: number | undefined): Promise<Page> {
    const j = await this.conn.call(this.at + ".list", { "limit": limit, "after": after, "count": (count == null ? undefined : count) });
    return fromPage(j);
  }
}

/// What `tickets` must offer to be served: a transpiled component does.
export interface TicketsActivation {
  list(limit: number, after: string, count: number | undefined): Page | Promise<Page>;
}

/// The activation's side: each `open` frame becomes a call into `activation`.
/// `at` is where it is mounted: its own name unless a parent holds it.
export function serveTickets(activation: TicketsActivation, server: Server, at: string = "tickets"): void {
  server.method(at + ".list", async (p: any) => { const v = await activation.list((p["limit"] ?? 50), (p["after"] ?? ""), (p["count"] == null ? undefined : p["count"])); return toPage(v); });
  server.describe(at, ticketsIr);
}
