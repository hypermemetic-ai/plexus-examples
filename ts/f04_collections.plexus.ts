// Generated from the IR of `tickets` (interface ce0507a52e271141). Not edited by hand.

import { Connection, Server, each, b64, unb64 } from "./plexus-rt.ts";

export interface Ticket { labels: Array<string>; parent?: string; cell: [number, number]; counts: Array<[string, number]>; avatar: Uint8Array; extra: string }

const toTicket = (v: Ticket): any => ({ "labels": v.labels, "parent": (v.parent == null ? undefined : v.parent), "cell": [v.cell[0], v.cell[1]], "counts": Object.fromEntries(v.counts.map(([k, x]: [string, any]) => [k, x])), "avatar": b64(v.avatar), "extra": JSON.parse(v.extra) });
const fromTicket = (j: any): Ticket => ({ labels: j["labels"], parent: (j["parent"] == null ? undefined : j["parent"]), cell: [j["cell"][0], j["cell"][1]], counts: Object.entries(j["counts"]).map(([k, x]: [string, any]) => [k, x]), avatar: unb64(j["avatar"]), extra: JSON.stringify(j["extra"]) });

/// The IR this file was generated from: what `_ir` answers.
export const ticketsIr = {"ir": "plexus/0", "activation": "tickets", "hash": "ce0507a52e271141", "doc": "Lists, options, tuples, string-keyed maps, bytes, and any JSON.", "types": [{"name": "ticket", "record": [{"name": "labels", "type": {"list": "string"}}, {"name": "parent", "type": {"option": "string"}}, {"name": "cell", "type": {"tuple": ["u32", "u32"]}, "doc": "Row and column on the board."}, {"name": "counts", "type": {"map": "u32"}}, {"name": "avatar", "type": "bytes"}, {"name": "extra", "type": "json", "doc": "Whatever an integration stored."}]}], "methods": [{"name": "get", "params": [{"name": "id", "type": "string"}], "returns": {"option": "ticket"}}]};

/// The caller's side of `tickets`: every method, typed, sent as frames.
export class TicketsClient {
  /// The interface this client was generated from.
  static readonly interface = "ce0507a52e271141";
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
  async get(id: string): Promise<Ticket | undefined> {
    const j = await this.conn.call(this.at + ".get", { "id": id });
    return (j == null ? undefined : fromTicket(j));
  }
}

/// What `tickets` must offer to be served: a transpiled component does.
export interface TicketsActivation {
  get(id: string): (Ticket | undefined) | Promise<(Ticket | undefined)>;
}

/// The activation's side: each `open` frame becomes a call into `activation`.
/// `at` is where it is mounted: its own name unless a parent holds it.
export function serveTickets(activation: TicketsActivation, server: Server, at: string = "tickets"): void {
  server.method(at + ".get", async (p: any) => { const v = await activation.get(p["id"]); return (v == null ? undefined : toTicket(v)); });
  server.describe(at, ticketsIr);
}
