// Generated from the IR of `tickets` (interface 323d85c1285ebabc). Not edited by hand.

import { Connection, Server, each } from "./plexus-rt.ts";

export interface Ticket { id: string }
export type TicketError =
  | { tag: "not-found"; val: string }
  | { tag: "no-title" };

const toTicket = (v: Ticket): any => ({ "id": v.id });
const fromTicket = (j: any): Ticket => ({ id: j["id"] });
const toTicketError = (v: TicketError): any => v.tag === "not-found" ? { "not-found": v.val } : (v as any).tag;
const fromTicketError = (j: any): TicketError => typeof j === "string" ? { tag: j } as TicketError : "not-found" in j ? { tag: "not-found", val: j["not-found"] } : j;

/// The IR this file was generated from: what `_ir` answers.
export const ticketsIr = {"ir": "plexus/0", "activation": "tickets", "hash": "323d85c1285ebabc", "doc": "A call that can fail names its error. The error is part of the interface.", "types": [{"name": "ticket", "record": [{"name": "id", "type": "string"}]}, {"name": "ticket-error", "variant": [{"name": "not-found", "type": "string"}, {"name": "no-title", "doc": "The title is empty."}]}], "methods": [{"name": "open", "params": [{"name": "title", "type": "string"}], "returns": "ticket", "fails": "ticket-error"}, {"name": "delete", "doc": "Nothing comes back; it can still fail.", "params": [{"name": "id", "type": "string"}], "fails": "ticket-error"}]};

/// The caller's side of `tickets`: every method, typed, sent as frames.
export class TicketsClient {
  /// The interface this client was generated from.
  static readonly interface = "323d85c1285ebabc";
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
  async open(title: string): Promise<Ticket> {
    const j = await this.conn.call(this.at + ".open", { "title": title }, (e: any) => fromTicketError(e));
    return fromTicket(j);
  }
  /// Nothing comes back; it can still fail.
  async delete(id: string): Promise<void> {
    await this.conn.call(this.at + ".delete", { "id": id }, (e: any) => fromTicketError(e));
  }
}

/// What `tickets` must offer to be served: a transpiled component does.
export interface TicketsActivation {
  open(title: string): Ticket | Promise<Ticket>;
  delete(id: string): void | Promise<void>;
}

/// The activation's side: each `open` frame becomes a call into `activation`.
/// `at` is where it is mounted: its own name unless a parent holds it.
export function serveTickets(activation: TicketsActivation, server: Server, at: string = "tickets"): void {
  server.method(at + ".open", async (p: any) => { const v = await activation.open(p["title"]); return toTicket(v); }, (e: any) => toTicketError(e));
  server.method(at + ".delete", async (p: any) => { await activation.delete(p["id"]); }, (e: any) => toTicketError(e));
  server.describe(at, ticketsIr);
}
