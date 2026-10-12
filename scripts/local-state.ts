import { existsSync, mkdirSync, readFileSync, writeFileSync } from "node:fs";
import { join } from "node:path";
import { Keypair } from "@solana/web3.js";
import { keypair } from "./client.ts";

export const directory = process.env.STATE_DIR ?? ".local/localnet";
export function localKey(name: string): Keypair {
  mkdirSync(directory, { recursive: true, mode: 0o700 });
  const path = join(directory, `${name}-keypair.json`);
  if (!existsSync(path)) writeFileSync(path, JSON.stringify(Array.from(Keypair.generate().secretKey)), { mode: 0o600, flag: "wx" });
  return keypair(path);
}
export interface DemoState { mint: string; source: string; vault: string; office: string; advisor: string; client: string; keeper: string; admin: string; rpc: string }
export function saveState(state: DemoState) { writeFileSync(join(directory, "state.json"), JSON.stringify(state, null, 2) + "\n", { mode: 0o600 }); }
export function readState(): DemoState { return JSON.parse(readFileSync(join(directory, "state.json"), "utf8")); }
