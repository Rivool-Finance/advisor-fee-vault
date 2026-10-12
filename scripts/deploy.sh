#!/usr/bin/env bash
set -euo pipefail
export PATH="$HOME/.cargo/bin:$HOME/.local/bin:$HOME/.local/share/solana/install/active_release/bin:$PATH"
: "${RPC_URL:=https://api.devnet.solana.com}"
: "${STATE_DIR:=.local/devnet}"
export STATE_DIR RPC_URL
deployer="$STATE_DIR/admin-keypair.json"
node --input-type=module -e 'import { localKey } from "./scripts/local-state.ts"; console.log("Deployer:", localKey("admin").publicKey.toBase58());'
genesis=$(solana genesis-hash --url "$RPC_URL")
case "$RPC_URL" in
  http://127.0.0.1:*|http://localhost:*) ;;
  *) test "$genesis" = EtWTRABZaYq6iMfeYKouRu166VU2xqa1wcaWoxPkrZBG || { echo 'Deployment is restricted to devnet/localnet' >&2; exit 1; } ;;
esac
node --input-type=module -e '
  import { localKey } from "./scripts/local-state.ts";
  for (const name of ["stand_in_yield-buffer", "advisor_fee_vault-buffer"]) localKey(name);
'
for program in stand_in_yield advisor_fee_vault; do
  case "$program" in
    stand_in_yield) expected=4ywtd7N6oR52qPRiZ66mX76HPpjHAXiCYqoyaNVFF1xh ;;
    advisor_fee_vault) expected=F4Q5gatG2GjEt8L62Q71z7jctaitHRxunJDDnbdo6oXm ;;
  esac
  actual=$(solana-keygen pubkey "target/deploy/$program-keypair.json")
  test "$actual" = "$expected" || { echo "Program key does not match $program: rebuild with intentionally synchronized IDs" >&2; exit 1; }
  solana program deploy "target/deploy/$program.so" --program-id "target/deploy/$program-keypair.json" \
    --url "$RPC_URL" --keypair "$deployer" --upgrade-authority "$deployer" \
    --buffer "$STATE_DIR/$program-buffer-keypair.json" \
    --max-len "$(wc -c < "target/deploy/$program.so")"
  solana program show "$expected" --url "$RPC_URL" --keypair "$deployer" --output json > "$STATE_DIR/$program-deployment.json"
done
