# Advisor Fee Vault: implementation plan

Accepted on 2026-10-12. Repository facts read at `main` commit `c529cc9`.
This plan is for contributors implementing the hackathon prototype. It records
the agreed scope, accounting contract, interfaces, validation, and release gates.
Items below describe planned behavior, not evidence of deployment.

## 1. Delivery and agreed scope

Build a **tested Solana devnet prototype with a scripted demo and automatic fee settlement**. The first delivery proves:

**Advisor configures fees → client deposits → funded yield appears → fees settle to three recipients → client withdraws.**

At planning time the repository contains documentation only. Rust, Anchor, and Solana tools are not on PATH, so toolchain setup is the first delivery gate.

Accepted decisions:

- Prioritize the program, tests, CLI demo, and keeper; defer screens and Privy.
- Allow advisory fees to reduce principal, with explicit disclosure.
- Cap combined office/advisor fees at **1% annually**; charge Rivool **15% of gains above a high-water mark**.
- Lock accepted rates for existing positions.
- Settle all earned fees before every deposit or withdrawal.
- Automatically settle inactive positions monthly.

The deadline is **13 October 2026, 03:59 BRT**, confirmed against Colosseum's official rules. Reserve the final two hours for recording and submission; do not trade accounting checks for deadline completion. [Official rules](https://colosseum.com/legal/Crypto%20World%27s%20Fair%20Hackathon%20Rules.pdf)

## 2. Programs and public interfaces

**Bootstrap one Anchor workspace containing two programs**, a Rust integration-test crate, and TypeScript scripts. Use Anchor 1.2.1 with matching CLI, Rust crates, and `@anchor-lang/core`; keep the TypeScript client on compatible `@solana/web3.js` v1. Pin dependencies and prove the combination with a compiled SPL-token transfer test before building the vault. Anchor and LiteSVM are both listed in the Colosseum resource hub. [Resources](https://colosseum.com/worldsfair/resources), [Anchor compatibility](https://www.anchor-lang.com/docs/clients/typescript)

### Stand-in yield program

- Use a project-controlled, six-decimal test mint, clearly labelled "mock USDC."
- Implement deposit, redemption, and exchange-rate reading using SPL tokens and receipt shares.
- Permit a fixture authority to change the exchange rate. Price increases require sufficient token backing; the demo funds gains before raising the price.
- Maintain a cumulative price-time index, checkpointed before price changes. This gives exact time-weighted AUM accounting for the stand-in's piecewise-constant prices.
- Pin this program and its accounts in the fee vault. Do not accept caller-selected CPI targets or prices.
- Keep Kamino as a documented future integration. Similar deposit/redemption concepts do not establish CPI compatibility.

### Fee-vault accounts

| Account | Responsibility |
|---|---|
| `Config` | Administrative authority, mint, fee ceilings, Rivool treasury, settlement interval |
| `Office` | Office authority, treasury, office rate, advisor allotment |
| `Advisor` | Advisor authority, office relationship, treasury, offered rate |
| `Vault` | Pinned yield source, PDA custody accounts, tracked receipt shares |
| `Position` | Client owner, advisor, accepted rates and recipients, shares, high-water mark, accrual checkpoints |

Use one position per client and vault. Hold underlying receipt shares in pooled PDA custody, with ownership recorded separately per position.

### Instructions and permissions

- Administrative initialization registers the configuration, yield source, offices, and advisors. Protect initial configuration against takeover by requiring the deployed program's upgrade authority.
- `set_office_fee` and `set_advisor_fee` require their respective authorities and enforce both allotments and the global ceiling.
- `open_position` requires the client and explicit expected rates/recipients, preventing configuration changes between preview and signing.
- `deposit`, partial/full `withdraw`, and `close_position` require the client.
- `accrue` is permissionless and checkpoints advisory fees without crystallizing performance gains.
- `settle` is permissionless when the billing interval is due. Client-authorized deposits and withdrawals invoke the same settlement logic immediately.
- Rates, recipients, and advisor association remain fixed while a position is funded. Changing them requires a full exit and reopening.

Emit events for configuration, deposits, accrual, settlement, and withdrawals. Generate IDLs and TypeScript types from the build; the CLI and keeper share one transaction-building module.

## 3. Accounting and custody rules

Implement fee calculations in a small Rust module used by the program and tested independently.

- Use integer token amounts, checked `u128` intermediates, a fixed-point exchange rate, and a **365-day year**. Carry fractional fee remainders between checkpoints.
- Advisory fees equal the integral of gross position value over elapsed time, multiplied by each accepted annual rate. Use the stand-in's price-time index so calling `accrue` more frequently cannot change the charge.
- Performance fees equal **15% of positive appreciation above the position's high-water mark**, calculated before advisory deductions and crystallized only during settlement.
- A falling price never lowers the high-water mark. After settling before a top-up, blend the remaining shares' existing high-water basis with the new shares' purchase price. Deposits themselves never count as profit.
- Settle all earned fees before changing the position's shares. Partial withdrawals leave the remaining shares with the settled high-water mark.
- Redeem the shares needed for fees and pay the office, advisor, and Rivool in one atomic transaction. Attribute redemption rounding residuals to the same position.
- Bound total collection by that position's assets. If fees exhaust the position, allocate available assets proportionally among amounts owed and write off uncollectible excess; never charge another position or a later deposit.
- Support minimum shares received on deposits and minimum assets received on withdrawals. Round issued shares and token payouts down; round shares required for an exact fee redemption up.

Validate signers, PDA seeds, account ownership, mints, token-program IDs, source accounts, and recipient addresses onchain. Reject unsupported Token-2022 mints and account substitutions.

There is no administrative withdrawal or arbitrary-call instruction. Document custody accurately: **clients control withdrawals; the program can collect agreed fees without a fresh client signature; upgrade authorities can change program behavior.**

## 4. Keeper, demo, and verification

### Keeper

- Implement a one-shot TypeScript worker that discovers positions, reads onchain eligibility, and settles due positions.
- Define "monthly" as **30 days since the last settlement** for this version.
- Run the worker hourly through GitHub Actions, with manual dispatch and overlapping-run protection.
- Give its signer only enough devnet SOL to pay transaction fees; it receives no administrative or client authority.
- On ambiguous transaction results, reread onchain state before retrying. Continue processing other positions, record confirmed signatures and failures, and fail the run if unresolved settlements remain.
- Test the worker with Node's built-in test runner and a controlled RPC/transaction boundary.

### Demo

Provide reproducible bootstrap, inspection, demo, and keeper commands. Use separate local keypairs for client, advisor, and fee payer.

Demonstrate an excessive fee rejection, valid configuration, a 1,000-token deposit, funded simulated yield, three recipient transfers, an unauthorized withdrawal rejection, and a successful client withdrawal. Print gross value, each fee, net value, and explorer links.

Use clock advancement only in LiteSVM. A separately labelled devnet demo configuration may use a fixed **60-second settlement interval**; annual fee rates and elapsed-time calculations remain unchanged. The demo invokes the same keeper worker used by the scheduler.

### Required tests

- Fee ceilings, role separation, configuration takeover, and substituted accounts.
- Constant-value AUM accrual, price changes, repeated accrual, fractional rounding, and timestamp bounds.
- Profit, loss, recovery, top-ups below the high-water mark, partial withdrawal, full exit, and reopening.
- Two clients with different rates sharing custody without shifting fees or assets between them.
- Settlement retries, simultaneous keepers, early settlement, insufficient backing, and atomic rollback.
- Conservation of deposited tokens, funded yield, client assets, and recipient payouts.
- A deterministic example: 1,000 tokens held for 30 days, then 40 tokens of funded appreciation; verify the 6-token performance fee and separately calculated advisory fees.

Run formatting checks, `anchor build`, Rust unit/LiteSVM tests, TypeScript type checking, and keeper tests in CI. Finish with a real devnet transaction sequence. [LiteSVM documentation](https://solana.com/docs/tools/litesvm)

## 5. Release gates and evidence

Implement in this order:

1. **Build gate:** reproducible toolchain, both programs compile, token-transfer smoke test passes.
2. **Accounting gate:** funded yield fixture, positions, fee calculations, and conservation tests pass.
3. **Behavior gate:** complete deposit/settle/withdraw loop and authorization tests pass.
4. **Operations gate:** CLI demo and keeper retry/scheduling tests pass.
5. **Devnet gate:** deploy, initialize, execute the full demo, and verify transaction balances.

Use a **2-of-3 Squads multisig** for the office authority and program upgrade authority, with three team-provided public keys. Update on 2026-10-12: the user authorized autonomous creation with `squads-multisig-cli`; generate three protected local signer files and disclose that initial custody is on one machine. Verify the authority transfer onchain before describing multisig governance as implemented. Squads publishes a devnet deployment. [Squads v4](https://github.com/Squads-Protocol/v4)

Update `README.md` and the existing design note with commands, program IDs, mint addresses, actual authority ownership, transaction evidence, accepted fee rules, and explicit prototype limitations. Leave the existing `AGENTS.md` unchanged.

Prepare a discrepancy checklist for the team's Colosseum draft: replace current Kamino and unrestricted custody claims, remove unimplemented mobile/multichain claims, substantiate traction figures, and disclose prior platform work. The team writes and submits its own answers. Colosseum requires a repository link, a 2–3-minute presentation, and a demo no longer than three minutes. [Submission requirements](https://colosseum.com/hackathon#h-faq-11)

**Deferred:** product UI, Privy, Kamino integration, fiat funding, additional strategies, stocks, and mainnet deployment. This first delivery serves a supervised technical demo; it does not yet provide the intended non-crypto client onboarding.

**Risks:** no implementation or compatibility tests ran during planning. The schedule is tight, and successful devnet tests do not establish production security or regulatory readiness.

## 6. Implementation progress

First increment, 2026-10-12:

- [x] Save the complete accepted plan.
- [x] Pin Rust 1.99.0, Anchor 1.2.1, Agave 4.3.0, and LiteSVM 0.18.0; build both programs and generate IDLs.
- [x] Implement the funded stand-in with deposits, redemptions, backing checks, and the price-time index.
- [x] Implement protected fee configuration, office/advisor registration, and rate-setting instructions.
- [x] Test advisory and performance arithmetic, rounding carry, high-water basis blending, and proportional fee caps.
- [x] Execute SPL-token CPIs and authorization/rollback tests in LiteSVM.
- [x] Add build/test instructions and a CI workflow.
- [x] Add pooled fee-vault custody and client positions, snapshot accepted terms, and connect the accounting helpers to position state.
- [x] Implement atomic three-way settlement and client deposit/withdrawal through the fee vault.
- [x] Implement the TypeScript client/demo, monthly keeper, and their tests; execute the real-clock local demo.
- [x] Verify the Squads CLI, atomic authority handoff and one-vs-two approval behavior against the official program on a local validator.
- [x] Activate the hosted keeper and verify its GitHub Actions run.
- [x] Configure Squads governance, deploy to devnet, and record transaction evidence.

The build, accounting, behavior and devnet gates now pass. See
[implementation evidence](08-implementation-evidence.md) for test results, local
and public transaction signatures and hosted keeper evidence. All implementation
release gates are complete. The team retains responsibility for recording, final
application claims and submission; those are not automated implementation tasks.
