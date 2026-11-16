// Generated from the IR of `tickets` (interface 43acd59c587f3318). Not edited by hand.

import { Connection, Server, each } from "./plexus-rt.ts";

export type Status = "draft" | "in-review" | "complete";
export type Change =
  | { tag: "status-changed"; val: { id: string; status: Status } }
  | { tag: "deleted"; val: string }
  | { tag: "cleared" };

const toStatus = (v: Status): any => (({ "in-review": "In Review" } as any)[v] ?? v);
const fromStatus = (j: any): Status => (({ "In Review": "in-review" } as any)[j] ?? j);
const toChange = (v: Change): any => v.tag === "status-changed" ? { "Moved": { "id": v.val.id, "status": toStatus(v.val.status) } } : v.tag === "deleted" ? { "Gone": v.val } : (({ "cleared": "All gone" } as any)[(v as any).tag] ?? (v as any).tag);
const fromChange = (j: any): Change => typeof j === "string" ? { tag: (({ "All gone": "cleared" } as any)[j] ?? j) } as Change : "Moved" in j ? { tag: "status-changed", val: { id: j["Moved"]["id"], status: fromStatus(j["Moved"]["status"]) } } : "Gone" in j ? { tag: "deleted", val: j["Gone"] } : j;

/// The IR this file was generated from: what `_ir` answers.
export const ticketsIr = {"ir": "plexus/0", "activation": "tickets", "hash": "43acd59c587f3318", "doc": "A case is written as its kebab name unless it says otherwise:\n`#[wire(\"In Review\")]` keeps a spelling the wire already has.", "types": [{"name": "status", "enum": [{"name": "draft"}, {"name": "in-review", "wire": "In Review"}, {"name": "complete"}]}, {"name": "change", "variant": [{"name": "status-changed", "wire": "Moved", "record": [{"name": "id", "type": "string"}, {"name": "status", "type": "status"}]}, {"name": "deleted", "wire": "Gone", "type": "string"}, {"name": "cleared", "wire": "All gone"}]}], "methods": [{"name": "set-status", "params": [{"name": "id", "type": "string"}, {"name": "status", "type": "status"}], "returns": "change"}]};

/// The caller's side of `tickets`: every method, typed, sent as frames.
export class TicketsClient {
  /// The interface this client was generated from.
  static readonly interface = "43acd59c587f3318";
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
  async setStatus(id: string, status: Status): Promise<Change> {
    const j = await this.conn.call(this.at + ".set-status", { "id": id, "status": toStatus(status) });
    return fromChange(j);
  }
}

/// What `tickets` must offer to be served: a transpiled component does.
export interface TicketsActivation {
  setStatus(id: string, status: Status): Change | Promise<Change>;
}

/// The activation's side: each `open` frame becomes a call into `activation`.
/// `at` is where it is mounted: its own name unless a parent holds it.
export function serveTickets(activation: TicketsActivation, server: Server, at: string = "tickets"): void {
  server.method(at + ".set-status", async (p: any) => { const v = await activation.setStatus(p["id"], fromStatus(p["status"])); return toChange(v); });
  server.describe(at, ticketsIr);
}
