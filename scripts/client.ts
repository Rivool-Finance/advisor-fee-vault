import { readFileSync } from "node:fs";
import anchor from "@anchor-lang/core";
import { Connection, Keypair, PublicKey, SystemProgram, Transaction, SYSVAR_CLOCK_PUBKEY } from "@solana/web3.js";
import type { TransactionInstruction } from "@solana/web3.js";
import { getAssociatedTokenAddressSync, TOKEN_PROGRAM_ID } from "@solana/spl-token";
import type { AdvisorFeeVault } from "../idl/advisor_fee_vault.ts";
import type { StandInYield } from "../idl/stand_in_yield.ts";

export const { BN } = anchor;
export const SCALE = 1_000_000_000_000n;
export const UNIT = 1_000_000n;
export const PROGRAM = new PublicKey("F4Q5gatG2GjEt8L62Q71z7jctaitHRxunJDDnbdo6oXm");
export const SOURCE_PROGRAM = new PublicKey("4ywtd7N6oR52qPRiZ66mX76HPpjHAXiCYqoyaNVFF1xh");
export const LOADER = new PublicKey("BPFLoaderUpgradeab1e11111111111111111111111");
export const bn = (n: bigint | number) => new BN(n.toString());
export const pda = (program: PublicKey, ...seeds: (string | PublicKey)[]) =>
  PublicKey.findProgramAddressSync(seeds.map(s => typeof s === "string" ? Buffer.from(s) : s.toBuffer()), program)[0];
export const ata = (mint: PublicKey, owner: PublicKey) => getAssociatedTokenAddressSync(mint, owner, true);
export const keypair = (path: string) => Keypair.fromSecretKey(Uint8Array.from(JSON.parse(readFileSync(path, "utf8"))));

export function connect(signer: Keypair, url = process.env.RPC_URL ?? "http://127.0.0.1:8899") {
  const connection = new Connection(url, { commitment: "confirmed", disableRetryOnRateLimit: true });
  const provider = new anchor.AnchorProvider(connection, new anchor.Wallet(signer), { commitment: "confirmed", preflightCommitment: "confirmed" });
  const fee = new anchor.Program<AdvisorFeeVault>(JSON.parse(readFileSync(new URL("../idl/advisor_fee_vault.json", import.meta.url), "utf8")), provider);
  const source = new anchor.Program<StandInYield>(JSON.parse(readFileSync(new URL("../idl/stand_in_yield.json", import.meta.url), "utf8")), provider);
  if (!fee.programId.equals(PROGRAM) || !source.programId.equals(SOURCE_PROGRAM)) throw new Error("Rebuild IDLs: program IDs do not match client");
  return { connection, provider, fee, source, signer };
}
export type Client = ReturnType<typeof connect>;

export async function chainTime(connection: Connection): Promise<bigint> {
  const clock = await connection.getAccountInfo(SYSVAR_CLOCK_PUBKEY);
  if (!clock || clock.data.length < 40) throw new Error("Chain clock unavailable");
  return clock.data.readBigInt64LE(32);
}

export async function settlementAccounts(c: Client, position: PublicKey) {
  const p = await c.fee.account.position.fetch(position);
  const v = await c.fee.account.vault.fetch(p.vault);
  const config = await c.fee.account.config.fetch(v.config);
  return {
    config: v.config, vault: p.vault, position, source: v.source, mint: config.mint,
    sharesMint: pda(SOURCE_PROGRAM, "shares", v.source), reserve: pda(SOURCE_PROGRAM, "reserve", v.source),
    cash: pda(PROGRAM, "cash", p.vault), custody: pda(PROGRAM, "custody", p.vault),
    officeAssets: ata(config.mint, p.recipients[0]), advisorAssets: ata(config.mint, p.recipients[1]), rivoolAssets: ata(config.mint, p.recipients[2]),
    sourceProgram: SOURCE_PROGRAM, tokenProgram: TOKEN_PROGRAM_ID,
  };
}

export async function openPosition(c: Client, vault: PublicKey, advisor: PublicKey) {
  const v = await c.fee.account.vault.fetch(vault);
  const a = await c.fee.account.advisor.fetch(advisor);
  const o = await c.fee.account.office.fetch(a.office);
  const config = await c.fee.account.config.fetch(v.config);
  const position = pda(PROGRAM, "position", vault, c.signer.publicKey);
  const signature = await c.fee.methods.openPosition([o.feeBps, a.feeBps], [o.treasury, a.treasury, config.treasury]).accountsStrict({
    client: c.signer.publicKey, config: v.config, vault, source: v.source, office: a.office, advisor, position, systemProgram: SystemProgram.programId,
  }).rpc();
  return { position, signature };
}

export async function deposit(c: Client, position: PublicKey, assets: bigint, minShares: bigint) {
  const settlement = await settlementAccounts(c, position);
  return c.fee.methods.deposit(bn(assets), bn(minShares)).accountsStrict({ client: c.signer.publicKey, clientAssets: ata(settlement.mint, c.signer.publicKey), settlement }).rpc();
}

export async function withdrawalInstruction(c: Client, position: PublicKey, shares: bigint | null, minAssets: bigint) {
  const settlement = await settlementAccounts(c, position);
  return c.fee.methods.withdraw(shares === null ? null : bn(shares), bn(minAssets)).accountsStrict({ client: c.signer.publicKey, clientAssets: ata(settlement.mint, c.signer.publicKey), settlement }).instruction();
}

export async function withdraw(c: Client, position: PublicKey, shares: bigint | null, minAssets: bigint) {
  return c.provider.sendAndConfirm(new Transaction().add(await withdrawalInstruction(c, position, shares, minAssets)));
}

export class TransactionAttemptError extends Error {
  signature: string;
  constructor(signature: string, cause: unknown) {
    super(`Transaction ${signature} was not confirmed`, { cause });
    this.signature = signature;
  }
}

export async function sendConfirmed(c: Client, ix: TransactionInstruction): Promise<string> {
  const block = await c.connection.getLatestBlockhash("confirmed");
  const tx = new Transaction({ ...block, feePayer: c.signer.publicKey }).add(ix);
  tx.sign(c.signer);
  const signature = anchor.utils.bytes.bs58.encode(tx.signature!);
  try {
    await c.connection.sendRawTransaction(tx.serialize(), { maxRetries: 3, preflightCommitment: "confirmed" });
    const confirmation = await c.connection.confirmTransaction({ ...block, signature }, "confirmed");
    if (confirmation.value.err) throw new Error(JSON.stringify(confirmation.value.err));
    return signature;
  } catch (cause) { throw new TransactionAttemptError(signature, cause); }
}

export function explorer(signature: string, rpc = "https://api.devnet.solana.com") {
  const local = ["127.0.0.1", "localhost"].includes(new URL(rpc).hostname);
  return `https://explorer.solana.com/tx/${signature}?${local ? `cluster=custom&customUrl=${encodeURIComponent(rpc)}` : "cluster=devnet"}`;
}
