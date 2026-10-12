# Advisor Fee Vault

A Solana prototype for collecting agreed office, advisor, and performance fees from
individual client positions in pooled custody. The yield source is a **funded,
controlled stand-in**, and the six-decimal token is **mock USDC**. Kamino is not integrated.

Implemented: protected configuration, accepted-term snapshots, client deposits and
withdrawals, time-weighted advisory accrual, atomic three-recipient settlement,
a permissionless keeper, CLI demo, and a Squads authority handoff.

**Verified locally:** 32 Rust tests, TypeScript checks and worker tests, a complete
1,000-token demo, token conservation, and 2-of-3 Squads execution using the official
program cloned from devnet. **Devnet deployment and hosted scheduling are not yet
verified.** See [evidence and remaining gates](context/08-implementation-evidence.md).

## Repository map

| Path | Purpose |
|---|---|
| `programs/advisor-fee-vault/` | Fee policies, positions, custody, accounting and authority transfer |
| `programs/stand-in-yield/` | Backed receipt shares and controlled exchange rate |
| `tests/svm/` | Tests against compiled SBF programs in LiteSVM |
| `scripts/` | Shared typed client, bootstrap, demo, keeper and governance utilities |
| `idl/` | Generated IDLs and TypeScript types; regenerate after program changes |
| `.github/workflows/` | Build checks, local demo and hourly devnet keeper |
| `context/07-implementation-plan.md` | Accepted full plan and progress |
| `context/02-fee-vault-design.md` | Current accounting and trust model |

The original research is preserved in `context/00-*.md` through `context/06-*.txt`.
`advisor-fee-vault.html` and `submission-outline.html` are historical, self-contained
proposal documents; open them offline. Press `N` in the deck for source notes.
Their product claims are not deployment evidence. See the
[submission discrepancy checklist](context/09-submission-discrepancies.md).

## Build and checks

Pinned tools: Rust **1.99.0**, Anchor **1.2.1**, Agave **4.3.0**, Node **26.6.0**,
LiteSVM **0.18.0**. Rust uses Anchor's Solana v4 feature; TypeScript uses the compatible
web3.js v1 client. Install [Rust](https://rustup.rs/),
[Anchor](https://github.com/otter-sec/anchor/releases/tag/v1.2.1), and
[Agave](https://github.com/anza-xyz/agave/releases/tag/v4.3.0), then:

```sh
. "$HOME/.cargo/env"
export PATH="$HOME/.local/bin:$HOME/.local/share/solana/install/active_release/bin:$PATH"
npm ci --ignore-scripts
anchor build
npm run idl
cargo test --workspace --locked
cargo fmt --all -- --check
cargo clippy --workspace --all-targets --locked -- -D warnings
npm run typecheck
npm test
git diff --check
```

Build before Rust transaction tests; they load `target/deploy/*.so`. Commit generated
`idl/` changes with program changes; CI checks regeneration. Keep program keypairs
under ignored `target/deploy/`. A fresh build may generate different keypairs;
`npm run deploy` rejects keys that do not match the declared IDs.

## Run the local demo

Start a validator in one terminal:

```sh
npm run validator -- --quiet
```

In another terminal:

```sh
npm run bootstrap
npm run demo
npm run verify:demo
npm run inspect
KEEPER_KEYPAIR=.local/localnet/keeper-keypair.json npm run keeper
```

The demo creates a position, deposits 1,000 mock tokens, funds 40 tokens of yield,
sets price 1.04, waits for the **real 60-second** billing interval, calls the actual
keeper, verifies all three payouts, rejects an unauthorized exit, and withdraws.
Annual rates are not accelerated. `verify:demo` checks confirmed inner transfers,
zero remaining shares, and conservation of the 1,040-token supply.

Keys and transaction evidence live under ignored `.local/localnet/`; private keys
are created with mode `0600`. `STATE_DIR` selects a separate run directory. The demo
refuses to overwrite an existing position; use a fresh directory **and fresh ledger**
to repeat. Do not delete a devnet key directory after deployment.

## Devnet deployment

The declared IDs are `F4Q5gatG2GjEt8L62Q71z7jctaitHRxunJDDnbdo6oXm` (fee vault) and
`4ywtd7N6oR52qPRiZ66mX76HPpjHAXiCYqoyaNVFF1xh` (stand-in). IDs alone do not prove deployment.

```sh
export STATE_DIR=.local/devnet
export RPC_URL=https://api.devnet.solana.com
npm run deploy
npm run bootstrap
npm run demo
npm run verify:demo
```

Fund the deployer with devnet SOL before deploying. Both programs and temporary
buffers require rent; retain enough SOL for initialization and transaction fees.
The script checks the devnet genesis hash and matching program keys. Bootstrap
creates separate client/advisor/keeper signers and funds their transaction fees
from the deployer. Program deployment files are recorded in the state directory.

## Squads governance

Use **squads-multisig-cli 0.1.7** (`cargo install squads-multisig-cli --version 0.1.7`).
After bootstrap and the demo:

```sh
npm run squads:create
npm run governance -- inspect <MULTISIG_ADDRESS>
npm run governance -- transfer <MULTISIG_ADDRESS>
```

The creation script uses three locally generated signer keys and a 2-of-3 threshold,
with no bypassing config authority. Review the CLI's displayed cluster and threshold.
The transfer command checks the multisig's ownership, PDA, membership and permissions,
then atomically moves office/config authority and both upgrade authorities to vault
index 0. Rerunning it verifies an already-completed transfer. Initial key custody is
on this machine, not distributed among three people; arrange a secure team handoff.

For the local governance smoke test, start a fresh validator with the official
Squads program and config cloned from devnet:

```sh
npm run validator -- --url https://api.devnet.solana.com \
  --clone-upgradeable-program SQDS4ep65T869zMMBKyuUq6aD6EgTu8psMjkvj52pCf \
  --clone BSTq9w3kZwNwpBXJEvTZz2G9ZTNyKBvoSeXMvwb4cNZr --quiet
```

After local bootstrap, Squads creation and transfer, run `npm run governance:smoke`.
It proves one approval cannot execute and two approvals can. It is local-only.

## Keeper operations

`.github/workflows/keeper.yml` runs hourly at minute 17 and supports manual dispatch.
It serializes runs and uses a SOL-only signer without client or admin privileges.
Configure repository Actions secret `KEEPER_KEYPAIR_JSON`
and optionally variable `DEVNET_RPC_URL`. Upload the ignored **keeper** key file,
never an admin, client, or multisig key. Scheduling begins only after the workflow
is on the repository's default branch and the secret is configured. No GitHub
environment or repository-admin permission is required.

The worker discovers funded positions, reads the chain clock, and settles those
whose billing interval elapsed. A timeout triggers an onchain state check; it never
blindly repeats a transaction. An unresolved attempt fails the run after other
positions have been processed, and the next run retries from fresh state.

## Fee rules and prototype limits

Office rate plus advisor allotment is capped at **100 bps annually**. Performance
fees are **15% above a high-water mark**. Rates and recipients are locked per position.
Every deposit or withdrawal settles all accrued fees; advisory fees may reduce
principal. Clients control withdrawals, while the program can collect agreed fees
without fresh client signatures. Upgrade authorities can change program behavior.

The mint and exchange-rate authority remain controlled fixtures. No real-dollar
backing, Kamino, UI/Privy, fiat rails, multichain support, or mainnet deployment is
provided. This code has not received an independent security audit.

`npm audit` still flags the unpatched transitive `bigint-buffer` native addon through
SPL/Squads. `.npmrc` and CI disable install scripts, so the tested path uses its JS
fallback. Do not enable the native addon. TOML and JSON-RPC dependencies are pinned
to patched versions through overrides; do not apply breaking `npm audit fix --force`.
