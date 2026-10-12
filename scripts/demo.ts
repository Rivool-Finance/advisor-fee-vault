import assert from "node:assert/strict";
import { setTimeout } from "node:timers/promises";
import { PublicKey } from "@solana/web3.js";
import { getAccount, transferChecked } from "@solana/spl-token";
import { appendFileSync } from "node:fs";
import { join } from "node:path";
import { ata, bn, chainTime, connect, deposit, explorer, openPosition, pda, PROGRAM, SCALE, SOURCE_PROGRAM, UNIT, withdraw } from "./client.ts";
import { directory, localKey, readState } from "./local-state.ts";
import { boundary } from "./keeper.ts";
import { runKeeper } from "./keeper-core.ts";

const state = readState();
const admin = connect(localKey("admin"), state.rpc);
const client = connect(localKey("client"), state.rpc);
const advisor = connect(localKey("advisor"), state.rpc);
const keeper = connect(localKey("keeper"), state.rpc);
const mint = new PublicKey(state.mint);
const vault = new PublicKey(state.vault);
const source = new PublicKey(state.source);
const advisorAddress = new PublicKey(state.advisor);
const position = pda(PROGRAM, "position", vault, client.signer.publicKey);
function record(step: string, signature: string) {
  const entry = { step, signature, explorer: explorer(signature, state.rpc), rpc: state.rpc };
  console.log(JSON.stringify(entry));
  appendFileSync(join(directory, "transactions.jsonl"), JSON.stringify(entry) + "\n");
}
async function rejection(name: string, code: string, action: () => Promise<unknown>) {
  await assert.rejects(action, (error: unknown) => {
    assert.match(String(error), new RegExp(code));
    console.log(`${name}: rejected (${code})`);
    return true;
  });
}
if (await client.connection.getAccountInfo(position)) throw new Error("Demo already started: inspect the position before resuming; use a fresh local ledger to repeat");
const initialPrice = (await admin.source.account.yieldVault.fetch(source)).price;
assert.equal(initialPrice.toString(), SCALE.toString(), "Demo requires initial price 1");
await rejection("Excessive advisor fee", "AdvisorAllotment", () => advisor.fee.methods.setAdvisorFee(61).accountsStrict({ authority: advisor.signer.publicKey, office: new PublicKey(state.office), advisor: advisorAddress }).rpc());
record("open", (await openPosition(client, vault, advisorAddress)).signature);
record("deposit 1000 mock USDC", await deposit(client, position, 1000n * UNIT, 1000n * UNIT));
record("fund 40 mock USDC yield", await transferChecked(admin.connection, admin.signer, ata(mint, admin.signer.publicKey), mint, pda(SOURCE_PROGRAM, "reserve", source), admin.signer, 40n * UNIT, 6));
record("backed exchange rate 1.04", await admin.source.methods.setExchangeRate(bn(SCALE * 104n / 100n)).accountsStrict({ authority: admin.signer.publicKey, vault: source, sharesMint: pda(SOURCE_PROGRAM, "shares", source), reserve: pda(SOURCE_PROGRAM, "reserve", source) }).rpc());
const p = await client.fee.account.position.fetch(position);
const recipients = p.recipients.map(owner => ata(mint, owner));
const before = await Promise.all(recipients.map(async address => (await getAccount(client.connection, address)).amount));
const due = BigInt(p.settledAt.toString()) + 60n;
console.log("Waiting for the real chain clock to reach the 60-second demo billing interval...");
while (await chainTime(client.connection) < due) await setTimeout(2000);
const results = await runKeeper(boundary(keeper));
console.log(JSON.stringify(results));
assert.ok(results.some(r => r.position === position.toBase58() && r.status !== "failed"), "Keeper must settle the demo position");
assert.ok(!results.some(r => r.status === "failed"), "Keeper has unresolved settlements");
for (const r of results) if (r.signature) record("keeper settlement", r.signature);
const after = await Promise.all(recipients.map(async address => (await getAccount(client.connection, address)).amount));
const fees = after.map((n, i) => n - before[i]);
assert.equal(fees[2], 6n * UNIT);
assert.ok(fees[0] > 0n && fees[1] > 0n);
console.log(JSON.stringify({ gross: "1040", officeFeeAtoms: fees[0].toString(), advisorFeeAtoms: fees[1].toString(), rivoolFeeAtoms: fees[2].toString(), netBeforeExitAtoms: (1040n * UNIT - fees.reduce((a, b) => a + b, 0n)).toString() }));
await rejection("Unauthorized withdrawal", "Unauthorized", () => withdraw(admin, position, null, 0n));
record("client full withdrawal", await withdraw(client, position, null, 1030n * UNIT));
const exited = await client.fee.account.position.fetch(position);
assert.ok(exited.shares.isZero() && exited.cash.isZero());
const clientAssets = (await getAccount(client.connection, ata(mint, client.signer.publicKey))).amount;
const sourceAssets = (await getAccount(client.connection, pda(SOURCE_PROGRAM, "reserve", source))).amount;
const vaultAssets = (await getAccount(client.connection, pda(PROGRAM, "cash", vault))).amount;
const finalRecipients = await Promise.all(recipients.map(async address => (await getAccount(client.connection, address)).amount));
assert.equal(clientAssets + sourceAssets + vaultAssets + finalRecipients.reduce((a, b, i) => a + b - before[i], 0n), 1040n * UNIT);
console.log(JSON.stringify({ clientNetAtoms: clientAssets.toString(), sourceRoundingAtoms: sourceAssets.toString(), conservedAtoms: (1040n * UNIT).toString() }));
