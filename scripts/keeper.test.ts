import test from "node:test";
import assert from "node:assert/strict";
import { runKeeper } from "./keeper-core.ts";
import type { DuePosition, KeeperBoundary } from "./keeper-core.ts";

const position = (id: string, settledAt = 0n): DuePosition => ({ id, settledAt, interval: 100n, funded: true });
function fake(positions: DuePosition[]) {
  const current = new Map(positions.map(p => [p.id, p]));
  const calls: string[] = [];
  const rpc: KeeperBoundary = {
    now: async () => 100n,
    discover: async () => positions,
    read: async id => current.get(id) ?? null,
    settle: async id => { calls.push(id); current.set(id, position(id, 100n)); return `sig-${id}`; },
  };
  return { rpc, current, calls };
}

test("chain time selects due, funded positions and retry runs do not bill twice", async () => {
  const f = fake([position("due"), position("early", 1n), { ...position("empty"), funded: false }]);
  assert.deepEqual(await runKeeper(f.rpc), [{ position: "due", status: "confirmed", signature: "sig-due" }]);
  assert.deepEqual(await runKeeper(f.rpc), []);
  assert.deepEqual(f.calls, ["due"]);
});

test("ambiguous success reconciles state without retrying", async () => {
  const f = fake([position("a")]);
  f.rpc.settle = async id => { f.current.set(id, position(id, 100n)); throw Object.assign(new Error("timeout"), { signature: "known-signature" }); };
  assert.deepEqual(await runKeeper(f.rpc), [{ position: "a", status: "reconciled", signature: "known-signature" }]);
});

test("an unresolved transaction does not stop other positions", async () => {
  const f = fake([position("a"), position("b")]);
  const send = f.rpc.settle;
  f.rpc.settle = async id => { if (id === "a") throw new Error("RPC unavailable"); return send(id); };
  assert.deepEqual(await runKeeper(f.rpc), [
    { position: "a", status: "failed", signature: undefined, error: "RPC unavailable" },
    { position: "b", status: "confirmed", signature: "sig-b" },
  ]);
});

test("a competing keeper between discovery and sending is skipped", async () => {
  const f = fake([position("a")]);
  f.current.set("a", position("a", 100n));
  assert.deepEqual(await runKeeper(f.rpc), []);
  assert.equal(f.calls.length, 0);
});

test("failure to reconcile remains a failure", async () => {
  const f = fake([position("a")]);
  f.rpc.settle = async () => { f.rpc.read = async () => { throw new Error("offline"); }; throw new Error("timeout"); };
  assert.equal((await runKeeper(f.rpc))[0].status, "failed");
});
