#!/usr/bin/env bash
set -euo pipefail
export PATH="$HOME/.cargo/bin:$HOME/.local/share/solana/install/active_release/bin:$PATH"
export STATE_DIR="${STATE_DIR:-.local/devnet}"
export RPC_URL="${RPC_URL:-https://api.devnet.solana.com}"
genesis=$(solana genesis-hash --url "$RPC_URL")
case "$RPC_URL" in
  http://127.0.0.1:*|http://localhost:*) ;;
  *) test "$genesis" = EtWTRABZaYq6iMfeYKouRu166VU2xqa1wcaWoxPkrZBG || { echo 'Squads setup is restricted to devnet/localnet' >&2; exit 1; } ;;
esac
mapfile -t members < <(node --input-type=module -e '
  import { localKey } from "./scripts/local-state.ts";
  localKey("admin"); localKey("squads-create");
  for (const name of ["squads-member-1", "squads-member-2", "squads-member-3"])
    console.log(`${localKey(name).publicKey.toBase58()},7`);
')
test "${#members[@]}" = 3
exec squads-multisig-cli multisig-create --rpc-url "$RPC_URL" \
  --program-id SQDS4ep65T869zMMBKyuUq6aD6EgTu8psMjkvj52pCf \
  --keypair "$STATE_DIR/admin-keypair.json" --seed-keypair "$STATE_DIR/squads-create-keypair.json" \
  --threshold 2 --members "${members[*]}"
