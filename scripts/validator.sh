#!/usr/bin/env bash
set -euo pipefail
export PATH="$HOME/.cargo/bin:$HOME/.local/share/solana/install/active_release/bin:$PATH"
export STATE_DIR="${STATE_DIR:-.local/localnet}"
node --input-type=module -e 'import { localKey } from "./scripts/local-state.ts"; console.log("Local admin:", localKey("admin").publicKey.toBase58());'
exec solana-test-validator --ledger "$STATE_DIR/ledger" --bind-address 127.0.0.1 --rpc-port 8899 \
  --upgradeable-program F4Q5gatG2GjEt8L62Q71z7jctaitHRxunJDDnbdo6oXm target/deploy/advisor_fee_vault.so "$STATE_DIR/admin-keypair.json" \
  --upgradeable-program 4ywtd7N6oR52qPRiZ66mX76HPpjHAXiCYqoyaNVFF1xh target/deploy/stand_in_yield.so "$STATE_DIR/admin-keypair.json" "$@"
