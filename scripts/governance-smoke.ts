import assert from "node:assert/strict";
import { PublicKey, TransactionMessage } from "@solana/web3.js";
import * as squads from "@sqds/multisig";
import { connect } from "./client.ts";
import { localKey, readState } from "./local-state.ts";
import { verifiedSquads } from "./governance.ts";

const state = readState();
assert.ok(["localhost", "127.0.0.1"].includes(new URL(state.rpc).hostname), "Governance smoke test is local-only");
const c = connect(localKey("admin"), state.rpc);
const members = [localKey("squads-member-1"), localKey("squads-member-2"), localKey("squads-member-3")];
for (const member of members) {
  if (await c.connection.getBalance(member.publicKey) < 50_000_000) {
    const signature = await c.connection.requestAirdrop(member.publicKey, 100_000_000);
    assert.equal((await c.connection.confirmTransaction(signature, "confirmed")).value.err, null);
  }
}
const multisigPda = squads.getMultisigPda({ createKey: localKey("squads-create").publicKey })[0];
const { account, vault } = await verifiedSquads(c, multisigPda);
assert.deepEqual(account.members.map(m => m.key.toBase58()).sort(), members.map(m => m.publicKey.toBase58()).sort());
const office = new PublicKey(state.office);
const current = await c.fee.account.office.fetch(office);
assert.ok(current.authority.equals(vault));
const ix = await c.fee.methods.setOfficeFee(current.feeBps, current.advisorAllotmentBps).accountsStrict({ authority: vault, office }).instruction();
const transactionIndex = BigInt(account.transactionIndex.toString()) + 1n;
const transactionMessage = new TransactionMessage({ payerKey: vault, recentBlockhash: (await c.connection.getLatestBlockhash()).blockhash, instructions: [ix] });
const common = { connection: c.connection, feePayer: c.signer, multisigPda, transactionIndex, sendOptions: { preflightCommitment: "confirmed" as const } };
async function confirmed(signature: string) { assert.equal((await c.connection.confirmTransaction(signature, "confirmed")).value.err, null); return signature; }
await confirmed(await squads.rpc.vaultTransactionCreate({ ...common, creator: members[0].publicKey, rentPayer: c.signer.publicKey, vaultIndex: 0, ephemeralSigners: 0, transactionMessage, signers: [members[0]] }));
await confirmed(await squads.rpc.proposalCreate({ ...common, creator: members[0], rentPayer: c.signer, isDraft: false }));
await confirmed(await squads.rpc.proposalApprove({ ...common, member: members[0] }));
await assert.rejects(() => squads.rpc.vaultTransactionExecute({ ...common, member: members[0].publicKey, signers: [members[0]] }), /InvalidProposalStatus|not.*approved|0x1779/i);
console.log("One approval cannot execute the office instruction");
await confirmed(await squads.rpc.proposalApprove({ ...common, member: members[1] }));
const signature = await confirmed(await squads.rpc.vaultTransactionExecute({ ...common, member: members[0].publicKey, signers: [members[0]] }));
assert.equal((await c.fee.account.office.fetch(office)).feeBps, current.feeBps);
console.log(JSON.stringify({ multisig: multisigPda.toBase58(), signature, verified: "Two approvals executed an authorized office instruction" }));
