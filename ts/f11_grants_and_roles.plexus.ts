// Generated from the IR of `tickets` (interface 8719fd908caae22b). Not edited by hand.

import { Connection, Server, each } from "./plexus-rt.ts";



/// The IR this file was generated from: what `_ir` answers.
export const ticketsIr = {"ir": "plexus/0", "activation": "tickets", "hash": "8719fd908caae22b", "doc": "Grants and roles. Every method is its own grant. A role is a list of\ngrants, and may include another role. `#[target]` marks the parameter\nthat names what the grant must be held on.", "roles": [{"name": "viewer", "grants": ["tickets.get", "tickets.list"]}, {"name": "editor", "includes": ["viewer"], "grants": ["tickets.set-status"]}, {"name": "admin", "includes": ["editor"], "grants": ["tickets.delete"]}], "types": [], "methods": [{"name": "get", "doc": "A viewer of this ticket may read it.", "caller": "required", "grant": "tickets.get", "target": "id", "params": [{"name": "id", "type": "string"}], "returns": "string"}, {"name": "list", "doc": "Answers only what the caller may see.", "caller": "required", "grant": "tickets.list", "params": [], "returns": {"list": "string"}}, {"name": "set-status", "doc": "An editor of this ticket may move it.", "caller": "required", "grant": "tickets.set-status", "target": "id", "params": [{"name": "id", "type": "string"}, {"name": "status", "type": "string"}], "returns": "string"}, {"name": "delete", "caller": "required", "grant": "tickets.delete", "target": "id", "params": [{"name": "id", "type": "string"}], "returns": "bool"}]};

/// The caller's side of `tickets`: every method, typed, sent as frames.
export class TicketsClient {
  /// The interface this client was generated from.
  static readonly interface = "8719fd908caae22b";
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
  /// A viewer of this ticket may read it.
  async get(id: string): Promise<string> {
    const j = await this.conn.call(this.at + ".get", { "id": id });
    return j;
  }
  /// Answers only what the caller may see.
  async list(): Promise<Array<string>> {
    const j = await this.conn.call(this.at + ".list", {  });
    return j;
  }
  /// An editor of this ticket may move it.
  async setStatus(id: string, status: string): Promise<string> {
    const j = await this.conn.call(this.at + ".set-status", { "id": id, "status": status });
    return j;
  }
  async delete(id: string): Promise<boolean> {
    const j = await this.conn.call(this.at + ".delete", { "id": id });
    return j;
  }
}

/// What `tickets` must offer to be served: a transpiled component does.
export interface TicketsActivation {
  get(id: string): string | Promise<string>;
  list(): Array<string> | Promise<Array<string>>;
  setStatus(id: string, status: string): string | Promise<string>;
  delete(id: string): boolean | Promise<boolean>;
}

/// The activation's side: each `open` frame becomes a call into `activation`.
/// `at` is where it is mounted: its own name unless a parent holds it.
export function serveTickets(activation: TicketsActivation, server: Server, at: string = "tickets"): void {
  throw new Error("serving this activation requires a trusted caller or an activation context; use the native Plexus server");
}
