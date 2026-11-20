// Generated from the IR of `session` (interface 8b3dab39752e35a2). Not edited by hand.

import { Connection, Server, each } from "./plexus-rt.ts";



/// The IR this file was generated from: what `_ir` answers.
export const sessionIr = {"ir": "plexus/0", "activation": "session", "hash": "8b3dab39752e35a2", "doc": "The user: `&Caller` is who is calling. A method that takes one needs a\nsigned-in caller; `Option<&Caller>` lets anyone in. Neither is sent as\na parameter.", "types": [], "methods": [{"name": "whoami", "doc": "Refused unless signed in.", "caller": "required", "params": [], "returns": "string"}, {"name": "greeting", "doc": "Anyone may ask.", "caller": "optional", "params": [], "returns": "string"}]};

/// The caller's side of `session`: every method, typed, sent as frames.
export class SessionClient {
  /// The interface this client was generated from.
  static readonly interface = "8b3dab39752e35a2";
  private conn: Connection;
  private at: string;
  /// `at` is where the activation is mounted: its own name unless a parent holds it.
  constructor(conn: Connection, at: string = "session") {
    this.conn = conn;
    this.at = at;
  }
  /// Ask what is served at `at`, once, and refuse if it is not this interface.
  async bind(): Promise<this> {
    const served = await this.conn.call(this.at + "._hash", {});
    if (served !== SessionClient.interface) throw new Error(`${this.at} serves interface ${served}; this client was generated from ${SessionClient.interface}`);
    return this;
  }
  /// Refused unless signed in.
  async whoami(): Promise<string> {
    const j = await this.conn.call(this.at + ".whoami", {  });
    return j;
  }
  /// Anyone may ask.
  async greeting(): Promise<string> {
    const j = await this.conn.call(this.at + ".greeting", {  });
    return j;
  }
}

/// What `session` must offer to be served: a transpiled component does.
export interface SessionActivation {
  whoami(): string | Promise<string>;
  greeting(): string | Promise<string>;
}

/// The activation's side: each `open` frame becomes a call into `activation`.
/// `at` is where it is mounted: its own name unless a parent holds it.
export function serveSession(activation: SessionActivation, server: Server, at: string = "session"): void {
  throw new Error("serving this activation requires a trusted caller or an activation context; use the native Plexus server");
}
