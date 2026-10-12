import { pathToFileURL } from "node:url";
import { PublicKey } from "@solana/web3.js";
import { chainTime, connect, keypair, sendConfirmed, settlementAccounts, pda, PROGRAM } from "./client.ts";
import type { Client } from "./client.ts";
import { runKeeper } from "./keeper-core.ts";
import type { DuePosition } from "./keeper-core.ts";

export function boundary(c: Client) {
  async function state(id: string): Promise<DuePosition | null> {
    const p = await c.fee.account.position.fetchNullable(new PublicKey(id));
    if (!p) return null;
    const v = await c.fee.account.vault.fetch(p.vault);
    const config = await c.fee.account.config.fetch(v.config);
    return { id, settledAt: BigInt(p.settledAt.toString()), interval: BigInt(config.settlementInterval.toString()), funded: !p.shares.isZero() || !p.cash.isZero() };
  }
  return {
    now: () => chainTime(c.connection),
    async discover() {
      const positions = await c.fee.account.position.all();
      const config = await c.fee.account.config.fetch(pda(PROGRAM, "config"));
      return positions.map(({ publicKey, account: p }) => ({
        id: publicKey.toBase58(), settledAt: BigInt(p.settledAt.toString()),
        interval: BigInt(config.settlementInterval.toString()), funded: !p.shares.isZero() || !p.cash.isZero(),
      }));
    },
    read: state,
    async settle(id: string) {
      const ix = await c.fee.methods.settle().accountsStrict(await settlementAccounts(c, new PublicKey(id))).instruction();
      return sendConfirmed(c, ix);
    },
  };
}

if (process.argv[1] && import.meta.url === pathToFileURL(process.argv[1]).href) {
  const path = process.env.KEEPER_KEYPAIR;
  if (!path) throw new Error("Set KEEPER_KEYPAIR to a SOL-only fee payer keypair file");
  const results = await runKeeper(boundary(connect(keypair(path))));
  console.log(JSON.stringify(results, null, 2));
  if (results.some(r => r.status === "failed")) process.exitCode = 1;
}
