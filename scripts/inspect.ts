import { readState, localKey } from "./local-state.ts";
import { connect, chainTime, PROGRAM } from "./client.ts";
const state = readState();
const c = connect(localKey("keeper"), state.rpc);
const positions = await c.fee.account.position.all();
console.log(JSON.stringify({ ...state, program: PROGRAM.toBase58(), chainTime: (await chainTime(c.connection)).toString(), positions: positions.map(({ publicKey, account: p }) => ({ address: publicKey.toBase58(), client: p.client.toBase58(), shares: p.shares.toString(), cash: p.cash.toString(), rates: p.rates, recipients: p.recipients.map(k => k.toBase58()), settledAt: p.settledAt.toString(), highWaterMark: p.highWaterMark.toString() })) }, null, 2));
