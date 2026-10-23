# Repository Guidelines

## Project Structure & Scope

Anchor prototype using mock USDC and a funded, controlled yield stand-in; Kamino, Privy, UI and mainnet are deferred. Read `README.md`, `context/02-fee-vault-design.md`, and `context/07-implementation-plan.md`. Verify release claims against `context/08-implementation-evidence.md`; historical decks and memory may be stale.

- `programs/`: fee vault and stand-in yield programs.
- `tests/svm/`: compiled-program LiteSVM tests.
- `scripts/`: shared TypeScript client, keeper, demo and governance tools.
- `idl/`: generated interfaces; regenerate with program changes.
- `.github/workflows/`: CI and hourly devnet keeper.

## Build, Test & Style

Use pinned tools from `README.md`; preserve Anchor/web3.js compatibility.

```sh
npm ci --ignore-scripts
NO_DNA=1 anchor build
npm run idl
cargo fmt --all -- --check
cargo test --workspace --locked
cargo clippy --workspace --all-targets --locked -- -D warnings
npm run typecheck
npm test
git diff --check
```

Build before LiteSVM tests. Do not pass `--locked` directly to `anchor build`; check that `Cargo.lock` remains unchanged. Check Anchor SPL feature flags when token-CPI compilation fails. Keep installation scripts disabled: the native `bigint-buffer` dependency has an unresolved vulnerability.

Use rustfmt, surrounding two-space TypeScript formatting, descriptive test names and `scripts/*.test.ts`. Preserve checked integer accounting, fractional carry, atomic settlement before deposits/withdrawals, and authorization/rollback tests. No coverage percentage is enforced.

## Lessons for Deployment & Verification

- Derive the funding address from the signer and compare every character; a truncated address can still be valid. Verify balance before deployment.
- After failed funding attempts, inspect the actual error and faucet liquidity. Avoid repeating unchanged requests or funding questions without new evidence.
- After RPC errors, inspect confirmed state before retrying; preserve keys and buffers. Bootstrap is not rerunnable after governance handoff; repeat demos on fresh local ledgers.
- Compare deployed bytes with tested binaries. Verify Squads threshold and all four authorities; preserve the office's initial-authority PDA seed. Disclose centralized signer custody.
- Wait for chain-clock eligibility before dispatching the keeper. An empty successful run proves no payment. Verify confirmed transfers and conservation; distinguish manual dispatch from observed cron execution.
- Avoid redundant CI runs. Before canceling duplicates, check their PR association; cancellation can appear as a failed check.

## Contributions & Secrets

Use `feat:`, `docs:` or `chore:` subjects. PRs explain behavior, validation and limitations; link issues and include screenshots for visual changes.

**Before planning a fix or a feature, search the open PRs** for the same problem and for the files you would touch: `gh pr list --state open --search "<problem>" --json number,title,files`. If one already does it, say so, and review or join it instead of planning a second.

Never commit secrets or keypairs; protect private files under ignored paths. Give Actions only the SOL-only keeper signer. Put transaction history in the evidence document and durable operating rules here.
