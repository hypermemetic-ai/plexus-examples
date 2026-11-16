// native › its generated client: ts/kiln.plexus.ts, its types taken out,
// on plexus-rt in Node, against the kiln process over a WebSocket.
// Run by tests/client.rs: node tests/client.mjs WS_URL CLIENT_JS PLEXUS_RT_JS

import assert from "node:assert/strict";

const [url, client, rt] = process.argv.slice(2);
const { Codec, Connection, PlexusFailure, socketChannel } = await import(rt);
const { KilnClient } = await import(client);

const channel = socketChannel(url);
const conn = new Connection(channel, await Codec.load(), KilnClient.interface);
await conn.ready;
const kiln = await new KilnClient(conn).bind();

const fired = await kiln.fire(6, 0.5, "ada");
assert.deepEqual(fired, { id: 1, cone: 6, hours: 0.5, loadedBy: "ada" });
assert.deepEqual(await kiln.log(), [fired]);
assert.deepEqual(await kiln.get(1), fired);
assert.deepEqual(await kiln.hottest(), fired);

const readings = [];
for await (const r of kiln.heat(6, 1)) readings.push(r);
assert.equal(readings.length, 13);
assert.deepEqual(readings[0], { minute: 0, celsius: 20, phase: "climbing" });
assert.equal(readings[12].phase, "cooling");

await assert.rejects(kiln.fire(11, 1, "ada"), (e) => {
  assert.ok(e instanceof PlexusFailure);
  assert.equal(e.code, "too-hot");
  assert.deepEqual(e.payload, { tag: "too-hot", val: 11 });
  return true;
});
await assert.rejects(kiln.fire(6, 1, " "), { code: "unattended", payload: { tag: "unattended" } });
await assert.rejects(kiln.get(9), { code: "not-found", payload: { tag: "not-found", val: 9 } });

channel.socket.close();
console.log("the generated client: bind, fire, log, get, hottest, heat and three typed failures");
