// Generated from the IR of `tickets` (interface 31074238ce93b36f). Not edited by hand.

import { Connection, Server, each } from "./plexus-rt.ts";

/// A ticket by any form of its address.
export type TicketRef = string;
export type Message = string;
export type Status = "pending" | "ready" | "coding" | "in-review" | "blocked" | "complete" | "draft" | "epic" | "superseded" | "canceled" | "doc";
export type CallerSource = "app" | "api" | "pipeline";
export type TicketsEvent =
  | { tag: "status-changed"; val: { id: TicketRef; status: Status } }
  | { tag: "not-found"; val: TicketRef }
  | { tag: "error"; val: string };

const toTicketRef = (v: TicketRef): any => v;
const fromTicketRef = (j: any): TicketRef => j;
const toMessage = (v: Message): any => v;
const fromMessage = (j: any): Message => j;
const toStatus = (v: Status): any => v;
const fromStatus = (j: any): Status => j;
const toCallerSource = (v: CallerSource): any => v;
const fromCallerSource = (j: any): CallerSource => j;
const toTicketsEvent = (v: TicketsEvent): any => v.tag === "status-changed" ? { "status-changed": { "id": toTicketRef(v.val.id), "status": toStatus(v.val.status) } } : v.tag === "not-found" ? { "not-found": toTicketRef(v.val) } : v.tag === "error" ? { "error": v.val } : (v as any).tag;
const fromTicketsEvent = (j: any): TicketsEvent => typeof j === "string" ? { tag: j } as TicketsEvent : "status-changed" in j ? { tag: "status-changed", val: { id: fromTicketRef(j["status-changed"]["id"]), status: fromStatus(j["status-changed"]["status"]) } } : "not-found" in j ? { tag: "not-found", val: fromTicketRef(j["not-found"]) } : "error" in j ? { tag: "error", val: j["error"] } : j;

/// The IR this file was generated from: what `_ir` answers.
export const ticketsIr = {"ir": "plexus/0", "activation": "tickets", "hash": "31074238ce93b36f", "doc": "menger's `tickets.set_status`, as it would be written here. For the\ncomparison with today's IR; its event has three of the real one's cases.", "roles": [{"name": "editor", "grants": ["tickets.set-status"]}], "types": [{"name": "ticket-ref", "doc": "A ticket by any form of its address.", "alias": "string"}, {"name": "message", "alias": "string"}, {"name": "status", "enum": [{"name": "pending"}, {"name": "ready"}, {"name": "coding"}, {"name": "in-review"}, {"name": "blocked"}, {"name": "complete"}, {"name": "draft"}, {"name": "epic"}, {"name": "superseded"}, {"name": "canceled"}, {"name": "doc"}]}, {"name": "caller-source", "enum": [{"name": "app"}, {"name": "api"}, {"name": "pipeline"}]}, {"name": "tickets-event", "variant": [{"name": "status-changed", "record": [{"name": "id", "type": "ticket-ref"}, {"name": "status", "type": "status"}]}, {"name": "not-found", "type": "ticket-ref"}, {"name": "error", "type": "string"}]}], "methods": [{"name": "set-status", "doc": "Set a ticket's status.", "async": true, "caller": "required", "grant": "tickets.set-status", "target": "id", "params": [{"name": "id", "type": "ticket-ref"}, {"name": "message", "type": {"option": "message"}}, {"name": "restore-children", "type": "bool", "default": false}, {"name": "source", "type": "caller-source", "default": "api"}, {"name": "status", "type": "status"}], "stream": "tickets-event"}]};

/// The caller's side of `tickets`: every method, typed, sent as frames.
export class TicketsClient {
  /// The interface this client was generated from.
  static readonly interface = "31074238ce93b36f";
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
  /// Set a ticket's status.
  setStatus(id: TicketRef, message: Message | undefined, restoreChildren: boolean, source: CallerSource, status: Status): AsyncIterable<TicketsEvent> {
    return each(this.conn.stream(this.at + ".set-status", { "id": toTicketRef(id), "message": (message == null ? undefined : toMessage(message)), "restore-children": restoreChildren, "source": toCallerSource(source), "status": toStatus(status) }), (j: any) => fromTicketsEvent(j));
  }
}

/// What `tickets` must offer to be served: a transpiled component does.
export interface TicketsActivation {
  setStatus(id: TicketRef, message: Message | undefined, restoreChildren: boolean, source: CallerSource, status: Status): AsyncIterable<TicketsEvent> | Promise<AsyncIterable<TicketsEvent>>;
}

/// The activation's side: each `open` frame becomes a call into `activation`.
/// `at` is where it is mounted: its own name unless a parent holds it.
export function serveTickets(activation: TicketsActivation, server: Server, at: string = "tickets"): void {
  server.stream(at + ".set-status", (p: any) => each(activation.setStatus(fromTicketRef(p["id"]), (p["message"] == null ? undefined : fromMessage(p["message"])), (p["restore-children"] ?? false), fromCallerSource((p["source"] ?? "api")), fromStatus(p["status"])), (v: any) => toTicketsEvent(v)));
  server.describe(at, ticketsIr);
}
