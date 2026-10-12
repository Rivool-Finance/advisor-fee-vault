import test from "node:test";
import assert from "node:assert/strict";
import { Keypair, PublicKey, SystemProgram } from "@solana/web3.js";
import * as squads from "@sqds/multisig";
import { validateSquads } from "./governance.ts";

function fixture(overrides = {}) {
  const createKey = Keypair.generate().publicKey;
  const address = squads.getMultisigPda({ createKey })[0];
  const data = squads.accounts.Multisig.fromArgs({
    createKey, configAuthority: PublicKey.default, threshold: 2, timeLock: 0, transactionIndex: 0, staleTransactionIndex: 0,
    rentCollector: null, bump: squads.getMultisigPda({ createKey })[1],
    members: Array.from({ length: 3 }, () => ({ key: Keypair.generate().publicKey, permissions: squads.types.Permissions.all() })), ...overrides,
  }).serialize()[0];
  return { address, info: { data, owner: squads.PROGRAM_ID, executable: false, lamports: 1 } };
}

test("only a canonical, unmanaged 2-of-3 Squads account is accepted", () => {
  const f = fixture();
  assert.ok(validateSquads(f.address, f.info).vault.equals(squads.getVaultPda({ multisigPda: f.address, index: 0 })[0]));
  assert.throws(() => validateSquads(f.address, { ...f.info, owner: SystemProgram.programId }), /official Squads/);
  assert.throws(() => validateSquads(Keypair.generate().publicKey, f.info), /PDA/);
  for (const invalid of [{ threshold: 1 }, { configAuthority: Keypair.generate().publicKey }, { members: [] }]) {
    const f = fixture(invalid);
    assert.throws(() => validateSquads(f.address, f.info));
  }
});
