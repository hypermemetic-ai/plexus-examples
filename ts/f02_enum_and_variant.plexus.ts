// Generated from the IR of `tickets` (interface 9bde84512db4bcf6). Not edited by hand.

import { Connection, Server, each } from "./plexus-rt.ts";

export type Status = "draft" | "ready" | "complete";
export type Change =
  | { tag: "created"; val: { id: string; title: string } }
  | { tag: "status-changed"; val: { id: string; status: Status } }
  | { tag: "deleted"; val: string }
  | { tag: "cleared" };

const toStatus = (v: Status): any => v;
const fromStatus = (j: any): Status => j;
const toChange = (v: Change): any => v.tag === "created" ? { "created": { "id": v.val.id, "title": v.val.title } } : v.tag === "status-changed" ? { "status-changed": { "id": v.val.id, "status": toStatus(v.val.status) } } : v.tag === "deleted" ? { "deleted": v.val } : (v as any).tag;
const fromChange = (j: any): Change => typeof j === "string" ? { tag: j } as Change : "created" in j ? { tag: "created", val: { id: j["created"]["id"], title: j["created"]["title"] } } : "status-changed" in j ? { tag: "status-changed", val: { id: j["status-changed"]["id"], status: fromStatus(j["status-changed"]["status"]) } } : "deleted" in j ? { tag: "deleted", val: j["deleted"] } : j;

/// The IR this file was generated from: what `_ir` answers.
export const ticketsIr = {"ir": "plexus/0", "activation": "tickets", "hash": "9bde84512db4bcf6", "doc": "An enum is one of several names. A variant's cases carry nothing, one\nvalue, or named fields.", "types": [{"name": "status", "enum": [{"name": "draft"}, {"name": "ready"}, {"name": "complete"}]}, {"name": "change", "variant": [{"name": "created", "record": [{"name": "id", "type": "string"}, {"name": "title", "type": "string"}], "doc": "A ticket was filed."}, {"name": "status-changed", "record": [{"name": "id", "type": "string"}, {"name": "status", "type": "status"}]}, {"name": "deleted", "type": "string"}, {"name": "cleared", "doc": "Everything went."}]}], "methods": [{"name": "last-change", "params": [], "returns": "change"}]};

/// The caller's side of `tickets`: every method, typed, sent as frames.
export class TicketsClient {
  /// The interface this client was generated from.
  static readonly interface = "9bde84512db4bcf6";
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
  async lastChange(): Promise<Change> {
    const j = await this.conn.call(this.at + ".last-change", {  });
    return fromChange(j);
  }
}

/// What `tickets` must offer to be served: a transpiled component does.
export interface TicketsActivation {
  lastChange(): Change | Promise<Change>;
}

/// The activation's side: each `open` frame becomes a call into `activation`.
/// `at` is where it is mounted: its own name unless a parent holds it.
export function serveTickets(activation: TicketsActivation, server: Server, at: string = "tickets"): void {
  server.method(at + ".last-change", async (p: any) => { const v = await activation.lastChange(); return toChange(v); });
  server.describe(at, ticketsIr);
}
