# Implementation evidence

Recorded 2026-10-12. Local tests, public devnet transactions and hosted operations
are identified separately below.

## Build and local checks

- [Hosted CI passed for `bd7e98c`](https://github.com/Rivool-Finance/advisor-fee-vault/actions/runs/38306473098),
  including a clean build, all 38 tests, generated-IDL checks and the real-clock demo.
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

Both public devnet ProgramData accounts were fetched on 2026-10-12 at 21:21 UTC.
Their executable bytes match these local artifacts byte for byte. This comparison
is not an independent reproducible-build attestation or security audit.

## Public devnet deployment and demo

The user funded the correct deployer with 20 devnet SOL through DevnetFaucet.org.
The earlier CLI/PoW funding failures are resolved. Both deployments use Solana devnet
(genesis `EtWTRABZaYq6iMfeYKouRu166VU2xqa1wcaWoxPkrZBG`):

| Artifact | Address |
|---|---|
| Fee program | `F4Q5gatG2GjEt8L62Q71z7jctaitHRxunJDDnbdo6oXm` |
| Stand-in program | `4ywtd7N6oR52qPRiZ66mX76HPpjHAXiCYqoyaNVFF1xh` |
| Mock-USDC mint | `HFPqYKUp4fgcuQg2AndhkKJaadENJGajs72BQoyqK4mU` |
| Vault | `8tBbjGVtPKGAjXXB7Megbw66JRVLgLG5xbE22jcgxG1t` |
| Client position | `F2vFWqun31BmqXp7jVuQySYMUutPoJQ4XBpUZLzLCdk9` |

- [Deploy stand-in](https://explorer.solana.com/tx/Mj23DKqP2P7rZB3Mo36jVqmRzDbcGDq91kPXpa24SbuiJn4HqAGJm5b1BkamUeci7iUXHpEF95JmxWxzMjVsBQX?cluster=devnet)
- [Deploy fee vault](https://explorer.solana.com/tx/iEb565Biseytq9ndUa8mcSfzC6CEdrtnmPUAZtyWEiebyhPuCtwXnbaHLntgXg2d1doDiKuYvFSoS54TLWcuXjb?cluster=devnet)
- [open](https://explorer.solana.com/tx/47ARDqcBK2waYdcs8327nhVuQqPuLL7nkboCHbQSEiz4d5rFazpgGZR1SqGKquUvEGD8YYCnDLfGPDuHxX6VGWjw?cluster=devnet)
- [deposit 1000 mock USDC](https://explorer.solana.com/tx/3szouWLmDErtJTnGrqbQ59hET6cCv7uRNhnKnddjMFHtVcBURkJDXpdNADBXTrAkCx76xorYrFFzNnn5t75f7Qc7?cluster=devnet)
- [fund 40 mock USDC yield](https://explorer.solana.com/tx/2o727NeQRSCa2RHQuWAArfZ1QCmS3pjjpieYd6LtkkmyR2YsQXcPj93RXw3CshKy5zoe1bzHMxWF5cRuB7WKZA16?cluster=devnet)
- [backed exchange rate 1.04](https://explorer.solana.com/tx/3moBrVeyafCwV3WBtaNbKzTK2sWEZxqhNPjYF3juojE6BMtJN9s8Whoo7oixBaEFyq8N6YjYJQ8jLsYf5cyfdA5c?cluster=devnet)
- [keeper settlement](https://explorer.solana.com/tx/RYCrh6c2zPSBrTkpxanLYGAktbmVhv4vwb94foip4mm7cw1wpjBvLgZwTaL5M2MoapVuY7yKBUewJB4DXSKX2Nk?cluster=devnet)
- [client full withdrawal](https://explorer.solana.com/tx/4kzHEYbfjmv2nJaqSFiSAt4wB3W1g1jh9EMtK5LnjC21H6VkDMtTPxgb4uwF3GyUp7ifTa8w3aHZjt9jVgbVNTLC?cluster=devnet)

`npm run verify:demo` independently read every confirmed demo transaction and its
inner token transfers at 21:20:35 UTC. The keeper paid 8 office atoms, 12 advisor
atoms and 6,000,000 Rivool atoms. The withdrawal collected one additional advisor
atom accrued since settlement. Final balances were:

| Holder | Mock-USDC atoms |
|---|---:|
| Client | 1,033,999,978 |
| Office | 8 |
| Advisor | 13 |
| Rivool | 6,000,000 |
| Source reserve rounding dust | 1 |
| Fee-vault cash | 0 |
| **Total / mint supply** | **1,040,000,000** |

Client shares, client cash and receipt supply were zero after exit. The live position
was subsequently funded again with 100 mock USDC for hosted keeper verification;
these exit balances describe the verified demo checkpoint, not the current position.
Bootstrap encountered public-RPC 429 responses; idempotent retries after cooldown
completed initialization without redeploying or duplicating initial mint funding.

## Public devnet Squads governance

- Multisig: `G4R896v89tFLL2WHH3keg4VKGz16bjYtK7QxhC7nkLxX`.
- Authority vault (index 0): `7ToafrEJ444XHaz2yGeMJFwZzXxYxzyimpxBk57HRr7y`.
- Threshold: **2 of 3**, three distinct full-permission members, no config authority.
- [CLI creation](https://explorer.solana.com/tx/4pwcXRWnscxaxePQungQu8A7LvNDc9x9zY2cmMvNZAHja9Poc7Er5Y321iocqVjokbJMZsJdjbzRYprR4Ys1fvhG?cluster=devnet).
- [Atomic authority handoff](https://explorer.solana.com/tx/4pgiSTMTJ4mvpvQsRobkDH92aCFjFkAro75hD3DMn7EPxwUZ22hapvbsV7DEx53oYcQMbfhrmooHn8jzL3YJtoKU?cluster=devnet).

The script checked canonical Squads ownership, discriminator, PDA, threshold and
membership, then reread config, office and both upgrade authorities after the
handoff. All four equal the authority vault above. Member public keys:

- `5oEoYne2AsKBoREh3D1rqugM3mFN5Dn15eYohbp9uGWk`
- `5wfQZDSdbbZHucJo4WtYaezbffdMpyP7AWyqAx2Xzsqv`
- `D4rBA8ACX3mWpDes8NqCygoDa2MjtnRjnWmqFw1zQ8vo`

Private signer files remain ignored and protected on this machine. This is real
onchain threshold governance, with centralized initial key custody. The one-vs-two
approval execution test above was local; the public proof covers creation and handoff.

## Hosted keeper

[PR #1](https://github.com/Rivool-Finance/advisor-fee-vault/pull/1) was merged to
`main` as `77a7aa8`. The hourly workflow is active, with repository secret
`KEEPER_KEYPAIR_JSON` for SOL-only signer
`2G5Ls118SBL2hieCUuiJmFTxjp2PE1qH1su6dapsvG5k`, funded by bootstrap.
No admin, client or multisig key is used by Actions.

The GitHub account has write access but cannot create repository environments.
The keeper uses the repository secret directly; no environment protection rule
was removed or bypassed.

- [Hosted run](https://github.com/Rivool-Finance/advisor-fee-vault/actions/runs/38310296257): **passed**, dispatched against `main`.
- [Confirmed settlement](https://explorer.solana.com/tx/3mdVRzT28uESen8Sg4zs99FDHo3EqUJ7CEE2ZctoPMYtPUgKMeuvFx9NTtxpDoqNuBoY23R1XtivmP4rFrH3EGC8?cluster=devnet).
- Independent RPC verification at 21:24:04 UTC confirmed the keeper was the sole
  signer, office/advisor each received one atom, and the position's settlement
  timestamp advanced. No new performance gain arose at the unchanged price.
- The complete mock-token supply still reconciled to 1,040,000,000 atoms. The
  100-token test position remains funded for subsequent hourly runs.
- Keeper balance after verification: **0.04999 devnet SOL**.

The hourly cron (`17 * * * *`) is installed on the default branch and the workflow
is active. The observed execution used manual dispatch of that same workflow;
a future timer-triggered run and long-term unattended reliability are not yet observed.
GitHub may delay scheduled jobs. Normal billing remains 30 days; this deployed
configuration intentionally uses the labelled 60-second demo interval.

## Remaining team work and limitations

Team recording, final claim reconciliation, application and submission remain
team-owned, as specified in the plan. The discrepancy checklist is prepared;
no Colosseum application text, edits or messages were sent.

The dependency audit retains one upstream native-addon issue (`bigint-buffer`),
reported through four packages after adding Squads. Installation scripts are
disabled and the tested JS fallback is used. This prototype has no independent audit.
