# Implementation evidence

Recorded 2026-10-12. This separates implemented behavior from release gates that
still require a public network or repository operation.

## Local checks

- Both SBF programs build with Anchor 1.2.1 and Agave 4.3.0.
- The deployment script uploaded both binaries to an empty local ledger using
  explicit buffer keypairs and recorded their program/authority metadata.
- 10 integer-math tests and 22 compiled-program LiteSVM tests pass.
- TypeScript type checking and six Node tests pass; these include keeper eligibility,
  ambiguous-result reconciliation, competing keepers, failure isolation and Squads validation.
- Rust formatting, Clippy with warnings denied, workflow lint and shell syntax checks pass.
- The complete CLI demo ran against a local validator, using the actual keeper and
  real chain time. `npm run verify:demo` independently fetched confirmed transactions,
  verified their inner token transfers, and reconciled the complete token supply.
- The official Squads v4 devnet program was cloned into the local validator.
  `squads-multisig-cli 0.1.7` created a 2-of-3 multisig. The authority handoff and
  subsequent governance test ran against that program, not a mock implementation.

## Deterministic accounting result

The LiteSVM example deposits 1,000 tokens, holds price 1 for 30 days, donates 40 tokens,
and raises price to 1.04. Office/advisor rates are 40/60 bps annually:

| Recipient | Atomic units | Tokens |
|---|---:|---:|
| Office | 328767 | 0.328767 |
| Advisor | 493150 | 0.493150 |
| Rivool | 6000000 | 6.000000 |

The suite also checks losses/recovery, top-up basis, complete settlement before a
partial exit, locked rates, multiple clients, rounding cash, fee exhaustion, full
exit/reopen, account substitution, Token-2022 rejection and all-or-nothing payment.
Freezing the third recipient causes both earlier transfers and redemption to roll back.

## Real-clock local demo

Local RPC: `http://127.0.0.1:8899`. These addresses and signatures belong to the local
ledger under ignored `.local/localnet-v2/`; they are **not devnet explorer evidence**.

- Mint: `GEF2facwrK18S8rU5frTuTgUPE6aK7Zi3AisPZfUCNeV`.
- Position: `6Ec4kGoNWD1R4FHjMMgbvPASnMDvwXSCdAzxGP5xuTEy`.
- Settlement: `4LzKyyXf9zbx2KXYBxtzydVsrcVb7StZfAZBgQFVJDTEPr4Pr4nyJG5evPAhe7ebkNm7L8zjFM8UXRZbvmrcSoQd`.
- Withdrawal: `384yToAy554Mye5Jb93WQpPsQhy8wxxGKjjtM4zVH7NgGERduE7eQpXtdwHDCfFFMuQm9UQARZFa7s9HqaYUwKWj`.
- Confirmed keeper transfers: office **8 atoms**, advisor **12 atoms**, Rivool
  **6,000,000 atoms**. Client exit returned **1,033,999,980 atoms**.
- Client assets plus recipient assets equal the **1,040,000,000-atom** mint supply.
  Receipt supply, vault cash and remaining reserve were all zero after this exit.

Actual advisory amounts vary with elapsed chain time. They are intentionally much
smaller than the 30-day deterministic example.

## Local Squads verification

- Multisig: `FA436UUr5v8io1Y3uWXwqN4f6yKRXxYuSbkgKjmY8mwb`.
- Authority vault (index 0): `3akn5KhEpN9baTD1bXf3CTbpHtQWbDDwsahhoSh5kRpc`.
- Threshold: **2 of 3**, no config authority, three members with full permissions.
- Creation: `4az3Wco6PCFAxN6UJ8ehmxkFJJooX1xM9VagxZ6Y1ktQGobLgJnMcXKeajBNTyo3WyjKbqtMhFyUm2mvHLegkHy9`.
- Atomic handoff: `4JAB77Dnw6E5NBx2Gzk35ZcKUYJmo8a3M1GjenArypxpgs9gGaRn9pogD8DVnNKsmbTRP9UEt52LCV1a6TwJz21r`.
- Two-approval execution: `5uY6n6JZjaDcAnwgQfNSYT9o6v2vSLZzfb9Z6UbjTX3DVsSQJtHq9P7Ng4i53sTn1XaszK2qn5GDUeLsAHhLifjJ`.

The handoff moved config, office and both upgrade authorities in one transaction,
then reread all four. One approval failed execution; two approvals executed a
fee-setting instruction through Squads. The three local keys are controlled from
one machine, as authorized for autonomous setup; this is not distributed human custody.

## Artifact hashes

SHA-256 of the locally tested SBF artifacts:

```text
a8b66af02b832c23c8f5da973af1fc0c6934f141bbc4b592af7864d60a5900ba  advisor_fee_vault.so
7c1964a72992992a850cd331e0f75c71707edaf4a5ba94e0995b1c3f76dd5b71  stand_in_yield.so
```

These identify the local binaries; they are not proof of a reproducible/verifiable
build or a public deployment.

## Outstanding release gates

- [ ] Fund deployer `AVUogj2RciV8GyqVqsdsy3J5HRDouY2JJnQcYKFbrmJX` with devnet SOL.
- [ ] Deploy both programs, bootstrap, run and independently verify the public demo.
- [ ] Create devnet Squads with the requested CLI and transfer/verify all authorities.
- [ ] Publish the workflow, configure its SOL-only keeper secret, and confirm a hosted run.
- [ ] Team recording, claim reconciliation, application and submission.

The CLI faucet rejected requests for 2, 1 and 0.001 SOL. The installed `devnet-pow`
0.1.4 reports the suggested difficulty-3/0.02-SOL pool empty. Its mining command also failed on the initial bootstrap airdrop.
No discovered faucet can currently fund a deployment at its advertised reward. No real funds, mainnet deployment, Colosseum edits or messages were sent.

The dependency audit retains one upstream native-addon issue (`bigint-buffer`),
reported through four packages after adding Squads. Installation scripts are
disabled and the tested JS fallback is used. This prototype has no independent audit.
