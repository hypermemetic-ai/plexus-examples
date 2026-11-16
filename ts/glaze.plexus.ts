// Generated from the IR of `glaze` (interface 4854f4dff7f0146d). Not edited by hand.

import { Connection, Server, each } from "./plexus-rt.ts";

export type Colorant = "clear" | "iron" | "copper" | "cobalt";
export interface Recipe { name: string; colorant: Colorant; cone: number }
/// What to weigh out.
export interface Batch { recipe: string; silicaGrams: number; fluxGrams: number; colorantGrams: number }
export type GlazeError =
  | { tag: "unknown"; val: string }
  | { tag: "too-small"; val: number };

const toColorant = (v: Colorant): any => v;
const fromColorant = (j: any): Colorant => j;
const toRecipe = (v: Recipe): any => ({ "name": v.name, "colorant": toColorant(v.colorant), "cone": v.cone });
const fromRecipe = (j: any): Recipe => ({ name: j["name"], colorant: fromColorant(j["colorant"]), cone: j["cone"] });
const toBatch = (v: Batch): any => ({ "recipe": v.recipe, "silica-grams": v.silicaGrams, "flux-grams": v.fluxGrams, "colorant-grams": v.colorantGrams });
const fromBatch = (j: any): Batch => ({ recipe: j["recipe"], silicaGrams: j["silica-grams"], fluxGrams: j["flux-grams"], colorantGrams: j["colorant-grams"] });
const toGlazeError = (v: GlazeError): any => v.tag === "unknown" ? { "unknown": v.val } : v.tag === "too-small" ? { "too-small": v.val } : (v as any).tag;
const fromGlazeError = (j: any): GlazeError => typeof j === "string" ? { tag: j } as GlazeError : "unknown" in j ? { tag: "unknown", val: j["unknown"] } : "too-small" in j ? { tag: "too-small", val: j["too-small"] } : j;

/// The IR this file was generated from: what `_ir` answers.
export const glazeIr = {"ir": "plexus/0", "activation": "glaze", "hash": "4854f4dff7f0146d", "doc": "Glaze recipes, and what a batch of one needs.", "types": [{"name": "colorant", "enum": [{"name": "clear"}, {"name": "iron"}, {"name": "copper"}, {"name": "cobalt"}]}, {"name": "recipe", "record": [{"name": "name", "type": "string"}, {"name": "colorant", "type": "colorant"}, {"name": "cone", "type": "u8", "doc": "The cone it matures at."}]}, {"name": "batch", "doc": "What to weigh out.", "record": [{"name": "recipe", "type": "string"}, {"name": "silica-grams", "type": "u32"}, {"name": "flux-grams", "type": "u32"}, {"name": "colorant-grams", "type": "u32"}]}, {"name": "glaze-error", "variant": [{"name": "unknown", "type": "string", "doc": "No recipe has this name."}, {"name": "too-small", "type": "u32", "doc": "A batch needs at least 100 grams."}]}], "methods": [{"name": "recipes", "doc": "Every recipe in the book.", "params": [], "returns": {"list": "recipe"}}, {"name": "coloured", "doc": "The recipes coloured with one thing.", "params": [{"name": "colorant", "type": "colorant"}], "returns": {"list": "recipe"}}, {"name": "weigh", "doc": "What a batch of a recipe needs.", "params": [{"name": "recipe", "type": "string"}, {"name": "grams", "type": "u32"}], "returns": "batch", "fails": "glaze-error"}]};

/// The caller's side of `glaze`: every method, typed, sent as frames.
export class GlazeClient {
  /// The interface this client was generated from.
  static readonly interface = "4854f4dff7f0146d";
  private conn: Connection;
  private at: string;
  /// `at` is where the activation is mounted: its own name unless a parent holds it.
  constructor(conn: Connection, at: string = "glaze") {
    this.conn = conn;
    this.at = at;
  }
  /// Ask what is served at `at`, once, and refuse if it is not this interface.
  async bind(): Promise<this> {
    const served = await this.conn.call(this.at + "._hash", {});
    if (served !== GlazeClient.interface) throw new Error(`${this.at} serves interface ${served}; this client was generated from ${GlazeClient.interface}`);
    return this;
  }
  /// Every recipe in the book.
  async recipes(): Promise<Array<Recipe>> {
    const j = await this.conn.call(this.at + ".recipes", {  });
    return j.map((x: any) => fromRecipe(x));
  }
  /// The recipes coloured with one thing.
  async coloured(colorant: Colorant): Promise<Array<Recipe>> {
    const j = await this.conn.call(this.at + ".coloured", { "colorant": toColorant(colorant) });
    return j.map((x: any) => fromRecipe(x));
  }
  /// What a batch of a recipe needs.
  async weigh(recipe: string, grams: number): Promise<Batch> {
    const j = await this.conn.call(this.at + ".weigh", { "recipe": recipe, "grams": grams }, (e: any) => fromGlazeError(e));
    return fromBatch(j);
  }
}

/// What `glaze` must offer to be served: a transpiled component does.
export interface GlazeActivation {
  recipes(): Array<Recipe> | Promise<Array<Recipe>>;
  coloured(colorant: Colorant): Array<Recipe> | Promise<Array<Recipe>>;
  weigh(recipe: string, grams: number): Batch | Promise<Batch>;
}

/// The activation's side: each `open` frame becomes a call into `activation`.
/// `at` is where it is mounted: its own name unless a parent holds it.
export function serveGlaze(activation: GlazeActivation, server: Server, at: string = "glaze"): void {
  server.method(at + ".recipes", async (p: any) => { const v = await activation.recipes(); return v.map((x: any) => toRecipe(x)); });
  server.method(at + ".coloured", async (p: any) => { const v = await activation.coloured(fromColorant(p["colorant"])); return v.map((x: any) => toRecipe(x)); });
  server.method(at + ".weigh", async (p: any) => { const v = await activation.weigh(p["recipe"], p["grams"]); return toBatch(v); }, (e: any) => toGlazeError(e));
  server.describe(at, glazeIr);
}
