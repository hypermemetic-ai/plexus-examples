// Generated from the IR of `kiln` (interface a4fe48dbd62f1985). Not edited by hand.

import { Connection, Server, each } from "./plexus-rt.ts";

/// One firing.
export interface Firing { id: number; cone: number; hours: number; loadedBy: string }
export type KilnError =
  | { tag: "too-hot"; val: number }
  | { tag: "not-found"; val: number }
  | { tag: "unattended" };
/// The kiln's temperature at one moment of a firing.
export interface Reading { minute: number; celsius: number; phase: Phase }
export type Phase = "climbing" | "holding" | "cooling";

const toFiring = (v: Firing): any => ({ "id": v.id, "cone": v.cone, "hours": v.hours, "loaded-by": v.loadedBy });
const fromFiring = (j: any): Firing => ({ id: j["id"], cone: j["cone"], hours: j["hours"], loadedBy: j["loaded-by"] });
const toKilnError = (v: KilnError): any => v.tag === "too-hot" ? { "too-hot": v.val } : v.tag === "not-found" ? { "not-found": v.val } : (v as any).tag;
const fromKilnError = (j: any): KilnError => typeof j === "string" ? { tag: j } as KilnError : "too-hot" in j ? { tag: "too-hot", val: j["too-hot"] } : "not-found" in j ? { tag: "not-found", val: j["not-found"] } : j;
const toReading = (v: Reading): any => ({ "minute": v.minute, "celsius": v.celsius, "phase": toPhase(v.phase) });
const fromReading = (j: any): Reading => ({ minute: j["minute"], celsius: j["celsius"], phase: fromPhase(j["phase"]) });
const toPhase = (v: Phase): any => v;
const fromPhase = (j: any): Phase => j;

/// The IR this file was generated from: what `_ir` answers.
export const kilnIr = {"ir": "plexus/0", "activation": "kiln", "hash": "a4fe48dbd62f1985", "doc": "A kiln, and the log of what was fired in it.", "types": [{"name": "firing", "doc": "One firing.", "record": [{"name": "id", "type": "u32"}, {"name": "cone", "type": "u8", "doc": "How hot: the pyrometric cone, 1 to 10."}, {"name": "hours", "type": "f32"}, {"name": "loaded-by", "type": "string"}]}, {"name": "kiln-error", "variant": [{"name": "too-hot", "type": "u8", "doc": "The kiln goes to cone 10; this asked for more."}, {"name": "not-found", "type": "u32", "doc": "No firing has this id."}, {"name": "unattended", "doc": "Nobody loaded it."}]}, {"name": "reading", "doc": "The kiln's temperature at one moment of a firing.", "record": [{"name": "minute", "type": "u32"}, {"name": "celsius", "type": "f32"}, {"name": "phase", "type": "phase"}]}, {"name": "phase", "enum": [{"name": "climbing"}, {"name": "holding"}, {"name": "cooling"}]}], "methods": [{"name": "fire", "doc": "Fire the kiln and write it in the log. It waits out the whole\nfiring, reading by reading; other calls run meanwhile.", "params": [{"name": "cone", "type": "u8"}, {"name": "hours", "type": "f32"}, {"name": "loaded-by", "type": "string"}], "returns": "firing", "fails": "kiln-error"}, {"name": "log", "doc": "Every firing, oldest first.", "params": [], "returns": {"list": "firing"}}, {"name": "get", "doc": "One firing by its id.", "params": [{"name": "id", "type": "u32"}], "returns": "firing", "fails": "kiln-error"}, {"name": "heat", "doc": "A firing as it happens: a reading every five minutes, climbing\nto the cone's temperature, holding there, then cooling. The\nnext reading is not worked out until the last one is taken.", "params": [{"name": "cone", "type": "u8"}, {"name": "hours", "type": "f32"}], "stream": "reading"}, {"name": "hottest", "doc": "The hottest firing so far, if there was one.", "params": [], "returns": {"option": "firing"}}]};

/// The caller's side of `kiln`: every method, typed, sent as frames.
export class KilnClient {
  /// The interface this client was generated from.
  static readonly interface = "a4fe48dbd62f1985";
  private conn: Connection;
  private at: string;
  /// `at` is where the activation is mounted: its own name unless a parent holds it.
  constructor(conn: Connection, at: string = "kiln") {
    this.conn = conn;
    this.at = at;
  }
  /// Ask what is served at `at`, once, and refuse if it is not this interface.
  async bind(): Promise<this> {
    const served = await this.conn.call(this.at + "._hash", {});
    if (served !== KilnClient.interface) throw new Error(`${this.at} serves interface ${served}; this client was generated from ${KilnClient.interface}`);
    return this;
  }
  /// Fire the kiln and write it in the log. It waits out the whole
  /// firing, reading by reading; other calls run meanwhile.
  async fire(cone: number, hours: number, loadedBy: string): Promise<Firing> {
    const j = await this.conn.call(this.at + ".fire", { "cone": cone, "hours": hours, "loaded-by": loadedBy }, (e: any) => fromKilnError(e));
    return fromFiring(j);
  }
  /// Every firing, oldest first.
  async log(): Promise<Array<Firing>> {
    const j = await this.conn.call(this.at + ".log", {  });
    return j.map((x: any) => fromFiring(x));
  }
  /// One firing by its id.
  async get(id: number): Promise<Firing> {
    const j = await this.conn.call(this.at + ".get", { "id": id }, (e: any) => fromKilnError(e));
    return fromFiring(j);
  }
  /// A firing as it happens: a reading every five minutes, climbing
  /// to the cone's temperature, holding there, then cooling. The
  /// next reading is not worked out until the last one is taken.
  heat(cone: number, hours: number): AsyncIterable<Reading> {
    return each(this.conn.stream(this.at + ".heat", { "cone": cone, "hours": hours }), (j: any) => fromReading(j));
  }
  /// The hottest firing so far, if there was one.
  async hottest(): Promise<Firing | undefined> {
    const j = await this.conn.call(this.at + ".hottest", {  });
    return (j == null ? undefined : fromFiring(j));
  }
}

/// What `kiln` must offer to be served: a transpiled component does.
export interface KilnActivation {
  fire(cone: number, hours: number, loadedBy: string): Firing | Promise<Firing>;
  log(): Array<Firing> | Promise<Array<Firing>>;
  get(id: number): Firing | Promise<Firing>;
  heat(cone: number, hours: number): AsyncIterable<Reading> | Promise<AsyncIterable<Reading>>;
  hottest(): (Firing | undefined) | Promise<(Firing | undefined)>;
}

/// The activation's side: each `open` frame becomes a call into `activation`.
/// `at` is where it is mounted: its own name unless a parent holds it.
export function serveKiln(activation: KilnActivation, server: Server, at: string = "kiln"): void {
  server.method(at + ".fire", async (p: any) => { const v = await activation.fire(p["cone"], p["hours"], p["loaded-by"]); return toFiring(v); }, (e: any) => toKilnError(e));
  server.method(at + ".log", async (p: any) => { const v = await activation.log(); return v.map((x: any) => toFiring(x)); });
  server.method(at + ".get", async (p: any) => { const v = await activation.get(p["id"]); return toFiring(v); }, (e: any) => toKilnError(e));
  server.stream(at + ".heat", (p: any) => each(activation.heat(p["cone"], p["hours"]), (v: any) => toReading(v)));
  server.method(at + ".hottest", async (p: any) => { const v = await activation.hottest(); return (v == null ? undefined : toFiring(v)); });
  server.describe(at, kilnIr);
}
