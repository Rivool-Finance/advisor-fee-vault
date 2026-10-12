import assert from "node:assert/strict";
import { pathToFileURL } from "node:url";
import { PublicKey, Transaction, TransactionInstruction } from "@solana/web3.js";
import type { AccountInfo } from "@solana/web3.js";
import * as squads from "@sqds/multisig";
import { connect, LOADER, pda, PROGRAM, SOURCE_PROGRAM } from "./client.ts";
import type { Client } from "./client.ts";
import { localKey, readState } from "./local-state.ts";

export function validateSquads(address: PublicKey, info: AccountInfo<Buffer>) {
  assert.ok(info.owner.equals(squads.PROGRAM_ID), "Multisig must be owned by the official Squads v4 program");
  assert.deepEqual([...info.data.subarray(0, 8)], squads.generated.multisigDiscriminator, "Incorrect multisig discriminator");
  const [account] = squads.accounts.Multisig.fromAccountInfo(info);
  assert.ok(squads.getMultisigPda({ createKey: account.createKey })[0].equals(address), "Invalid multisig PDA");
  assert.equal(account.threshold, 2, "Multisig threshold must be two");
  assert.equal(account.members.length, 3, "Multisig must have exactly three members");
  assert.equal(new Set(account.members.map(m => m.key.toBase58())).size, 3, "Members must be distinct");
  assert.ok(account.configAuthority.equals(PublicKey.default), "A config authority must not bypass the multisig threshold");
  assert.ok(account.members.every(m => m.permissions.mask === squads.types.Permissions.all().mask), "All three members need initiate, vote and execute permissions");
  return { account, vault: squads.getVaultPda({ multisigPda: address, index: 0 })[0] };
}

export async function verifiedSquads(c: Client, address: PublicKey) {
  const info = await c.connection.getAccountInfo(address);
  assert.ok(info, "Multisig account does not exist");
  return validateSquads(address, info);
}

export function upgradeAuthorityInstruction(program: PublicKey, current: PublicKey, next: PublicKey) {
  // Loader-v3 SetAuthority (variant 4); the verified Squads PDA cannot sign this initial handoff.
  const data = Buffer.alloc(4);
  data.writeUInt32LE(4);
  return new TransactionInstruction({ programId: LOADER, data, keys: [
    { pubkey: pda(LOADER, program), isSigner: false, isWritable: true },
    { pubkey: current, isSigner: true, isWritable: false },
    { pubkey: next, isSigner: false, isWritable: false },
  ] });
}

export async function readUpgradeAuthority(c: Client, program: PublicKey) {
  const info = await c.connection.getAccountInfo(pda(LOADER, program));
  assert.ok(info && info.owner.equals(LOADER) && info.data.length >= 45 && info.data.readUInt32LE(0) === 3 && info.data[12] === 1, "Expected upgradeable ProgramData");
  return new PublicKey(info.data.subarray(13, 45));
}

export async function transferGovernance(c: Client, office: PublicKey, multisig: PublicKey) {
  const { vault } = await verifiedSquads(c, multisig);
  const config = pda(PROGRAM, "config");
  const authorities = [
    (await c.fee.account.config.fetch(config)).authority,
    (await c.fee.account.office.fetch(office)).authority,
    await readUpgradeAuthority(c, PROGRAM), await readUpgradeAuthority(c, SOURCE_PROGRAM),
  ];
  if (authorities.every(a => a.equals(vault))) return { vault, signature: null };
  assert.ok(authorities.every(a => a.equals(c.signer.publicKey)), "All four authorities must still belong to the deployer before atomic handoff");
  const tx = new Transaction().add(
    await c.fee.methods.transferOfficeAuthority(vault).accountsStrict({ authority: c.signer.publicKey, office }).instruction(),
    await c.fee.methods.transferConfigAuthority(vault).accountsStrict({ authority: c.signer.publicKey, config }).instruction(),
    upgradeAuthorityInstruction(PROGRAM, c.signer.publicKey, vault),
    upgradeAuthorityInstruction(SOURCE_PROGRAM, c.signer.publicKey, vault),
  );
  const signature = await c.provider.sendAndConfirm(tx);
  assert.ok((await c.fee.account.office.fetch(office)).authority.equals(vault));
  assert.ok((await c.fee.account.config.fetch(config)).authority.equals(vault));
  for (const program of [PROGRAM, SOURCE_PROGRAM]) assert.ok((await readUpgradeAuthority(c, program)).equals(vault));
  return { vault, signature };
}

if (process.argv[1] && import.meta.url === pathToFileURL(process.argv[1]).href) {
  const state = readState();
  const c = connect(localKey("admin"), state.rpc);
  const [action, ...args] = process.argv.slice(2);
  if ((action === "inspect" || action === "transfer") && args.length === 1) {
    const multisig = new PublicKey(args[0]);
    const { account, vault } = await verifiedSquads(c, multisig);
    if (action === "transfer") {
      const result = await transferGovernance(c, new PublicKey(state.office), multisig);
      console.log(JSON.stringify({ vault: result.vault.toBase58(), signature: result.signature }));
    }
    console.log(JSON.stringify({ multisig: multisig.toBase58(), vault: vault.toBase58(), threshold: account.threshold, members: account.members.map(m => m.key.toBase58()), upgrades: await Promise.all([PROGRAM, SOURCE_PROGRAM].map(async p => ({ program: p.toBase58(), authority: (await readUpgradeAuthority(c, p)).toBase58() }))) }, null, 2));
  } else throw new Error("Usage: governance inspect <multisig> | transfer <multisig>; use npm run squads:create to create it with the CLI");
}
