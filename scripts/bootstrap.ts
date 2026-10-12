import { SystemProgram, Transaction } from "@solana/web3.js";
import { createMint, getOrCreateAssociatedTokenAccount, mintTo, TOKEN_PROGRAM_ID } from "@solana/spl-token";
import { connect, pda, PROGRAM, SOURCE_PROGRAM, LOADER, UNIT } from "./client.ts";
import { localKey, saveState } from "./local-state.ts";

const admin = localKey("admin");
const client = localKey("client");
const advisor = localKey("advisor");
const keeper = localKey("keeper");
const officeTreasury = localKey("office-treasury");
const rivoolTreasury = localKey("rivool-treasury");
const mintKey = localKey("mock-usdc-mint");
const c = connect(admin);
const genesis = await c.connection.getGenesisHash();
const isLocal = new URL(c.connection.rpcEndpoint).hostname === "127.0.0.1" || new URL(c.connection.rpcEndpoint).hostname === "localhost";
if (!isLocal && genesis !== "EtWTRABZaYq6iMfeYKouRu166VU2xqa1wcaWoxPkrZBG") throw new Error("Bootstrap is restricted to localhost and Solana devnet");
for (const signer of [admin, client, advisor, keeper]) {
  if (await c.connection.getBalance(signer.publicKey) < 50_000_000) {
    if (isLocal) {
      const signature = await c.connection.requestAirdrop(signer.publicKey, 1_000_000_000);
      await c.connection.confirmTransaction(signature, "confirmed");
    } else {
      if (signer === admin) throw new Error(`Fund deployer ${admin.publicKey.toBase58()} with devnet SOL first`);
      await c.provider.sendAndConfirm(new Transaction().add(SystemProgram.transfer({ fromPubkey: admin.publicKey, toPubkey: signer.publicKey, lamports: 50_000_000 })));
    }
  }
}
const mint = mintKey.publicKey;
if (!await c.connection.getAccountInfo(mint)) await createMint(c.connection, admin, admin.publicKey, null, 6, mintKey);
const source = pda(SOURCE_PROGRAM, "yield", admin.publicKey, mint);
const sharesMint = pda(SOURCE_PROGRAM, "shares", source);
const reserve = pda(SOURCE_PROGRAM, "reserve", source);
if (!await c.connection.getAccountInfo(source)) await c.source.methods.initialize().accountsStrict({
  authority: admin.publicKey, mint, vault: source, sharesMint, reserve, tokenProgram: TOKEN_PROGRAM_ID, systemProgram: SystemProgram.programId,
}).rpc();
const config = pda(PROGRAM, "config");
if (!await c.connection.getAccountInfo(config)) await c.fee.methods.initializeConfig(rivoolTreasury.publicKey, true).accountsStrict({
  authority: admin.publicKey, program: PROGRAM, programData: pda(LOADER, PROGRAM), mint, config, systemProgram: SystemProgram.programId,
}).rpc();
const configState = await c.fee.account.config.fetch(config);
if (!configState.mint.equals(mint) || !configState.authority.equals(admin.publicKey) || configState.settlementInterval.toNumber() !== 60) throw new Error("Existing config does not match this demo's keys/mint/interval");
const vault = pda(PROGRAM, "vault", config, source);
if (!await c.connection.getAccountInfo(vault)) await c.fee.methods.registerVault().accountsStrict({
  authority: admin.publicKey, config, mint, source, sharesMint, vault, cash: pda(PROGRAM, "cash", vault), custody: pda(PROGRAM, "custody", vault), tokenProgram: TOKEN_PROGRAM_ID, systemProgram: SystemProgram.programId,
}).rpc();
const office = pda(PROGRAM, "office", config, admin.publicKey);
if (!await c.connection.getAccountInfo(office)) await c.fee.methods.registerOffice(officeTreasury.publicKey, 40, 60).accountsStrict({
  authority: admin.publicKey, config, officeAuthority: admin.publicKey, office, systemProgram: SystemProgram.programId,
}).rpc();
const advisorAddress = pda(PROGRAM, "advisor", office, advisor.publicKey);
if (!await c.connection.getAccountInfo(advisorAddress)) await c.fee.methods.registerAdvisor(advisor.publicKey, 60).accountsStrict({
  authority: admin.publicKey, office, advisorAuthority: advisor.publicKey, advisor: advisorAddress, systemProgram: SystemProgram.programId,
}).rpc();
for (const owner of [client, admin, advisor, officeTreasury, rivoolTreasury]) {
  const account = await getOrCreateAssociatedTokenAccount(c.connection, admin, mint, owner.publicKey);
  const amount = owner === client ? 1000n * UNIT : owner === admin ? 40n * UNIT : 0n;
  // Initial funding only; never replenish a spent demo silently on a second bootstrap.
  if (amount && account.amount === 0n && (await c.connection.getTokenSupply(mint)).value.amount === (owner === client ? "0" : (1000n * UNIT).toString())) {
    await mintTo(c.connection, admin, mint, account.address, admin, amount);
  }
}
saveState({ mint: mint.toBase58(), source: source.toBase58(), vault: vault.toBase58(), office: office.toBase58(), advisor: advisorAddress.toBase58(), client: client.publicKey.toBase58(), keeper: keeper.publicKey.toBase58(), admin: admin.publicKey.toBase58(), rpc: c.connection.rpcEndpoint });
console.log(JSON.stringify({ mint: mint.toBase58(), vault: vault.toBase58(), intervalSeconds: 60, label: "mock USDC; controlled yield; provisional local office authority" }, null, 2));
