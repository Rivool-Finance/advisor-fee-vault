import assert from "node:assert/strict";
import { createHash } from "node:crypto";
import { appendFileSync, existsSync, readFileSync, writeFileSync } from "node:fs";
import { join } from "node:path";
import { setTimeout } from "node:timers/promises";
import type { TokenBalance } from "@solana/web3.js";
import { PublicKey } from "@solana/web3.js";
import { getAccount, getMint } from "@solana/spl-token";
import { ata, chainTime, connect, deposit, explorer, LOADER, pda, PROGRAM, SOURCE_PROGRAM, UNIT, withdraw } from "./client.ts";
import { directory, localKey, readState } from "./local-state.ts";
import { boundary } from "./keeper.ts";
import { runKeeper } from "./keeper-core.ts";

// One-shot evidence run: inspect the journal and confirmed state before any manual recovery.
const verifyOnly = process.argv.includes("--verify");
const journal = join(directory, "operations-transactions.jsonl");
assert.ok(verifyOnly || !existsSync(journal), "Operations already started; do not blindly repeat transactions");
const state = readState();
for (const name of ["client", "keeper"]) assert.ok(existsSync(join(directory, `${name}-keypair.json`)));
const client = connect(localKey("client"), state.rpc);
const keeper = connect(localKey("keeper"), state.rpc);
assert.equal(await client.connection.getGenesisHash(), "EtWTRABZaYq6iMfeYKouRu166VU2xqa1wcaWoxPkrZBG");
assert.equal(client.signer.publicKey.toBase58(), state.client);
assert.equal(keeper.signer.publicKey.toBase58(), state.keeper);
const mint = new PublicKey(state.mint);
const vault = new PublicKey(state.vault);
const source = new PublicKey(state.source);
const position = pda(PROGRAM, "position", vault, client.signer.publicKey);
const config = await client.fee.account.config.fetch(pda(PROGRAM, "config"));
assert.equal(config.settlementInterval.toString(), "60");
const binaries = [];
for (const [name, program] of [["advisor_fee_vault", PROGRAM], ["stand_in_yield", SOURCE_PROGRAM]] as const) {
  const account = await client.connection.getAccountInfo(program);
  assert.ok(account);
  assert.ok(account.owner.equals(LOADER) && account.executable);
  assert.equal(account.data.readUInt32LE(), 2);
  const data = await client.connection.getAccountInfo(new PublicKey(account.data.subarray(4, 36)));
  assert.ok(data);
  assert.ok(data.owner.equals(LOADER));
  assert.equal(data.data.readUInt32LE(), 3);
  const local = readFileSync(`target/deploy/${name}.so`);
  assert.ok(data.data.subarray(45, 45 + local.length).equals(local), "Deployed bytes differ from tested binary");
  assert.ok(data.data.subarray(45 + local.length).every(byte => byte === 0));
  binaries.push({ name, sha256: createHash("sha256").update(local).digest("hex"), bytes: local.length });
}
async function snapshot() {
  const p = await client.fee.account.position.fetch(position);
  const owners = [client.signer.publicKey, new PublicKey(state.admin), ...p.recipients];
  const addresses = [...owners.map(owner => ata(mint, owner)), pda(SOURCE_PROGRAM, "reserve", source), pda(PROGRAM, "cash", vault)];
  const balances = [];
  for (const address of addresses) {
    await setTimeout(1500);
    balances.push({ address: address.toBase58(), atoms: (await getAccount(client.connection, address)).amount.toString() });
  }
  const supply = (await getMint(client.connection, mint)).supply;
  assert.equal(balances.reduce((sum, b) => sum + BigInt(b.atoms), 0n), supply);
  return { chainTime: (await chainTime(client.connection)).toString(), shares: p.shares.toString(), cash: p.cash.toString(), settledAt: p.settledAt.toString(), highWaterMark: p.highWaterMark.toString(), supply: supply.toString(), balances };
}
const before: Awaited<ReturnType<typeof snapshot>> = verifyOnly
  ? JSON.parse(readFileSync(join(directory, "operations-before.json"), "utf8"))
  : await snapshot();
const transactions: { step: string; signature: string; explorer: string }[] = verifyOnly
  ? readFileSync(journal, "utf8").trim().split("\n").map(line => JSON.parse(line)).filter(entry => entry.signature)
  : [];
if (!verifyOnly) {
  assert.ok(BigInt(before.shares) > 0n);
  assert.ok(BigInt(before.balances[0].atoms) >= 200n * UNIT);
  for (const c of [client, keeper]) assert.ok(await c.connection.getBalance(c.signer.publicKey) >= 1_000_000);
  writeFileSync(journal, "", { flag: "wx", mode: 0o600 });
  writeFileSync(join(directory, "operations-before.json"), JSON.stringify(before, null, 2) + "\n");
}
async function record(step: string, action: () => Promise<string>) {
  appendFileSync(journal, JSON.stringify({ step, status: "attempting" }) + "\n");
  const signature = await action();
  const entry = { step, signature, explorer: explorer(signature, state.rpc) };
  transactions.push(entry);
  appendFileSync(journal, JSON.stringify(entry) + "\n");
  console.log(JSON.stringify(entry));
}
if (!verifyOnly) {
  await record("top-up 200 mock USDC", () => deposit(client, position, 200n * UNIT, 190n * UNIT));
  await record("partial withdrawal 50 receipt tokens", () => withdraw(client, position, 50n * UNIT, 51n * UNIT));
  const p = await client.fee.account.position.fetch(position);
  const due = BigInt(p.settledAt.toString()) + 60n;
  const deadline = Date.now() + 180_000;
  console.log("Waiting for the real 60-second devnet billing interval");
  while (await chainTime(client.connection) < due) {
    assert.ok(Date.now() < deadline, "Chain clock did not reach eligibility; inspect before resuming");
    await setTimeout(2000);
  }
  await record("permissionless keeper settlement", async () => {
    const b = boundary(keeper);
    const results = await runKeeper({ ...b, discover: async () => [await b.read(position.toBase58())].filter(p => p !== null) });
    assert.equal(results.length, 1);
    assert.ok(results[0].signature, JSON.stringify(results));
    return results[0].signature;
  });
  await record("client full exit", () => withdraw(client, position, null, 247n * UNIT));
  const exited = await snapshot();
  assert.equal(exited.shares, "0");
  assert.equal(exited.cash, "0");
  await record("redeposit 100 mock USDC for hourly keeper", () => deposit(client, position, 100n * UNIT, 95n * UNIT));
}
assert.equal(transactions.length, 5, "Incomplete operation sequence; inspect before recovery");
const p = await client.fee.account.position.fetch(position);
const after = await snapshot();
assert.ok(BigInt(after.shares) > 0n);
assert.equal(after.supply, before.supply);
const verified = [];
for (const entry of transactions) {
  await setTimeout(1200);
  const tx = await client.connection.getParsedTransaction(entry.signature, { commitment: "confirmed", maxSupportedTransactionVersion: 0 });
  assert.ok(tx?.meta && tx.meta.err === null);
  const transfers = (tx.meta.innerInstructions ?? []).flatMap(i => i.instructions).flatMap(ix => "parsed" in ix && ix.parsed.type === "transferChecked" ? [ix.parsed.info] : []);
  const signers = tx.transaction.message.accountKeys.filter(k => k.signer).map(k => k.pubkey.toBase58());
  if (entry.step === "permissionless keeper settlement") {
    assert.deepEqual(signers, [state.keeper]);
    for (const owner of p.recipients.slice(0, 2)) assert.ok(transfers.some(t => t.destination === ata(mint, owner).toBase58() && BigInt(t.tokenAmount.amount) > 0n));
    assert.ok(!transfers.some(t => t.destination === ata(mint, p.recipients[2]).toBase58()));
  }
  if (entry.step === "client full exit") {
    for (const address of [pda(PROGRAM, "custody", vault), pda(PROGRAM, "cash", vault)]) {
      const index = tx.transaction.message.accountKeys.findIndex(k => k.pubkey.equals(address));
      const balance: TokenBalance | undefined = tx.meta.postTokenBalances?.find(b => b.accountIndex === index);
      assert.ok(balance, "Missing full-exit custody/cash evidence");
      assert.equal(balance.uiTokenAmount.amount, "0");
    }
  }
  verified.push({ ...entry, slot: tx.slot, blockTime: tx.blockTime, signers, transfers, postTokenBalances: tx.meta.postTokenBalances });
}
const evidence = { checkedAt: new Date().toISOString(), rpc: state.rpc, position: position.toBase58(), mint: state.mint, binaries, before, fullExitVerified: true, after, transactions: verified };
writeFileSync(join(directory, "operations-evidence.json"), JSON.stringify(evidence, null, 2) + "\n");
console.log(JSON.stringify(evidence, null, 2));
