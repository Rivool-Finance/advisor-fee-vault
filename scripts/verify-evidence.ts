import assert from "node:assert/strict";
import { readFileSync, writeFileSync } from "node:fs";
import { join } from "node:path";
import { PublicKey } from "@solana/web3.js";
import { getAccount, getMint } from "@solana/spl-token";
import { ata, connect, pda, PROGRAM, SOURCE_PROGRAM, UNIT } from "./client.ts";
import { directory, localKey, readState } from "./local-state.ts";

const state = readState();
const c = connect(localKey("keeper"), state.rpc);
const mint = new PublicKey(state.mint);
const vault = new PublicKey(state.vault);
const source = new PublicKey(state.source);
const position = pda(PROGRAM, "position", vault, new PublicKey(state.client));
const p = await c.fee.account.position.fetch(position);
assert.ok(p.shares.isZero() && p.cash.isZero(), "Client has not fully exited");
const records = readFileSync(join(directory, "transactions.jsonl"), "utf8").trim().split("\n").map(line => JSON.parse(line) as { step: string; signature: string });
const settlements = [];
for (const record of records) {
  const tx = await c.connection.getParsedTransaction(record.signature, { commitment: "confirmed", maxSupportedTransactionVersion: 0 });
  assert.ok(tx?.meta && tx.meta.err === null, `Transaction not confirmed: ${record.step}`);
  if (record.step === "keeper settlement") {
    const transfers = (tx.meta.innerInstructions ?? []).flatMap(i => i.instructions).flatMap(ix => "parsed" in ix && ix.parsed.type === "transferChecked" ? [ix.parsed.info] : []);
    const payments = p.recipients.map(owner => {
      const destination = ata(mint, owner).toBase58();
      const transfer = transfers.find(t => t.destination === destination);
      assert.ok(transfer, `Missing recipient transfer to ${destination}`);
      assert.equal(transfer.mint, mint.toBase58());
      return { destination, amount: transfer.tokenAmount.amount as string };
    });
    assert.equal(payments[2].amount, (6n * UNIT).toString());
    settlements.push({ signature: record.signature, payments });
  }
}
assert.equal(settlements.length, 1, "Expected exactly one demo keeper settlement");
const addresses = [ata(mint, new PublicKey(state.client)), ata(mint, new PublicKey(state.admin)), pda(SOURCE_PROGRAM, "reserve", source), pda(PROGRAM, "cash", vault), ...p.recipients.map(owner => ata(mint, owner))];
const unique = [...new Set(addresses.map(a => a.toBase58()))];
const balances = await Promise.all(unique.map(async address => ({ address, amount: (await getAccount(c.connection, new PublicKey(address))).amount.toString() })));
const supply = (await getMint(c.connection, mint)).supply;
assert.equal(supply, 1040n * UNIT);
assert.equal(balances.reduce((total, item) => total + BigInt(item.amount), 0n), supply);
assert.equal((await getMint(c.connection, pda(SOURCE_PROGRAM, "shares", source))).supply, 0n);
const evidence = { rpc: state.rpc, checkedAt: new Date().toISOString(), mint: state.mint, position: position.toBase58(), supply: supply.toString(), settlements, balances, transactions: records };
writeFileSync(join(directory, "verified-evidence.json"), JSON.stringify(evidence, null, 2) + "\n");
console.log(JSON.stringify(evidence, null, 2));
