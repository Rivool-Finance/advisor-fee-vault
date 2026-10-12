# Fee vault program: first design

Draft for the developer and the Solana vault specialist. Mirrors the rules in
`apps/api/src/fees/` (rivool-monorepo 83acca0bc). Not reviewed, not audited.

## Accounts

| Account | One per | Holds |
|---|---|---|
| `Config` | Rivool | Rivool treasury, performance fee (1500 bps), office and advisor ceilings, allowed yield sources |
| `Office` | Office | Authority (its Squads vault), treasury, office fee bps, advisor allotment |
| `Advisor` | Advisor | Authority, office, advisor fee bps, checked with the same rule as `advisorFeeLimit` |
| `Vault` | Strategy (e.g. Kamino USDC) | Underlying vault shares held by a program-derived account; total shares |
| `Position` | Client | Owner, advisor, shares, high-water mark, last accrual time, fees owed per recipient |

## Instructions

| Instruction | Signer | Does |
|---|---|---|
| `set_office_fee` / `set_advisor_fee` | Office / advisor | Rejects anything above the ceiling |
| `open_position` | Client | Binds the client to an advisor; only the client can change it |
| `deposit` | Client | USDC in, deposited into the yield vault, shares credited |
| `accrue` | Anyone (crank) | Updates fees owed from the vault share price |
| `settle` | Anyone | Redeems owed fees and pays office, advisor and Rivool in USDC in one transaction |
| `withdraw` | Client only | Accrues, settles fees for the withdrawn part, sends the rest to the client |

## Fee maths, per position (P = vault share price)

- Advisory fee: value x (office bps + advisor bps) x elapsed seconds / seconds per year.
- Performance fee: if P > high-water mark, 15% x shares x (P - mark); then mark = P.
- Fees come out of yield first; the demo shows net yield beside gross yield.

## Trust rules

- No admin withdrawal path. Only the client's signature moves principal.
- Upgrade authority on a Squads multisig, stated openly. Say "funds move only with the client's
  signature", not "fully self-custodial".
- The advisor's Privy signer is restricted by policy to fee-setting instructions; never `withdraw`.

## Build notes

- Pooled vault, separate positions: fees differ per advisor, so accounting is per position.
- Biggest unknown: the call into Kamino. Check the current interface first; keep `Vault` generic
  and use a test vault with a settable share price if Kamino slips.
- Tests: LiteSVM for fee maths (one accrual, high-water mark, three-way split, mid-period
  withdrawal); Surfpool for a run against mainnet state.
- Playbook: the `solana-dev` skill; Solana program examples; Anchor docs; Kamino curator docs;
  Blueshift program security. No Solana Cookbook recipe covers vaults or fee splits.
- Shortcut considered and not chosen: a Voltr vault with a small splitter. One fee per vault,
  performance fee only, and most of the build would be someone else's program.

## Open design questions

1. Does the advisory fee apply to yield-only balances, and is it capped?
2. Settle only at withdrawal and on a crank, or also pay advisors on a schedule?
