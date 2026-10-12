# Fee vault: implemented prototype design

Updated 2026-10-12. This supersedes the original design draft. The accepted scope is
in [the implementation plan](07-implementation-plan.md); verification and deployment
status are recorded separately in [the implementation evidence](08-implementation-evidence.md).
This is a supervised mock-USDC prototype, not an audited investment product.

## Accounts and custody

The fee program owns a `Vault` PDA with pooled underlying-token and receipt-token
accounts. Each `Position` records its own shares, rounding cash, and fee checkpoints.
`Vault.total_shares` tracks allocated receipt shares; unsolicited token transfers do
not create a position claim.

| Account | Identity and responsibility |
|---|---|
| `Config` | Singleton; administrative authority, six-decimal legacy SPL mint, Rivool treasury, billing interval |
| `Office` | Config plus initial office authority; current authority, treasury, fee and advisor allotment |
| `Advisor` | Office plus advisor authority; offered advisor rate and treasury |
| `Vault` | Config plus pinned stand-in source; custody authority and allocated share total |
| `Position` | Vault plus client; accepted office/advisor identities, two rates, three recipient owners, shares, cash, high-water mark, fee carry and timestamps |

The office's PDA uses its initial authority, allowing later authority transfer to
Squads without changing office identity or existing positions. Positions snapshot
rates and recipient owners when opened; profile changes affect new positions only.
The client submits expected terms to reject configuration changes between preview
and signing. Full exit, close, and reopen are required to accept different terms.

## Instructions and permissions

- Only the deployed fee program's upgrade authority can initialize `Config`.
- The config authority registers offices and yield sources. The office authority
  registers advisors and sets its fee/allotment; advisors set their own offered rate.
- The client opens, deposits, withdraws, and closes its position. Closing requires
  zero shares and zero rounding cash.
- Anyone can `accrue`, which checkpoints advisory fees without crystallizing gains.
- Anyone can `settle` after 30 days since the last settlement. The explicit demo
  configuration substitutes 60 seconds; it does not accelerate annual fee rates.
- Every deposit and withdrawal first settles **all** earned position fees through
  the same collection function, regardless of how much the client changes.
- Current config/office authorities can transfer those roles. The governance script
  verifies the Squads account and atomically transfers both roles and both program
  upgrade authorities to its vault PDA.

The source program ID, source account, token program, mint, receipt mint, reserves,
PDA custody, client ownership, and recipient ownership are checked onchain. Only
legacy SPL Token is supported; Token-2022 mints are rejected. Recipient destinations
must be token accounts owned by the snapshotted treasury owners; clients cannot
substitute their own recipients. There is no administrative withdrawal or arbitrary CPI.

## Integer accounting

Token amounts and shares use atomic `u64` units; checked `u128` intermediates use a
price scale of 10¹². A year is 31,536,000 seconds. Office rate plus advisor allotment
cannot exceed 100 bps, and the advisor rate cannot exceed its allotment.

The stand-in checkpoints a cumulative integral of price over time before every
price change. Advisory charges integrate position shares against this index, plus
position rounding cash against elapsed time. Fractional remainders survive
checkpoints, so additional `accrue` calls cannot change the amount charged.

Performance fees are 1,500 bps of gross appreciation above the position's per-share
high-water mark, before advisory deductions. Only settlement crystallizes gains.
Losses do not lower the mark. A top-up first settles, then blends the remaining
shares' previous basis with the new purchase price, rounding the basis upward.
Partial withdrawal preserves the remaining shares' mark.

Fees first use the position's cash, then redeem enough shares, rounded upward.
Any excess redeemed tokens stay as that position's cash. All three transfers are
atomic; failure of the last transfer also rolls back the earlier payments and
redemption. Charges are capped at the position's own assets and allocated
proportionally on exhaustion; excess debt and fractional carry are written off.
Deposits round shares down. Withdrawals round underlying payouts down and enforce
minimum output. A full exit can burn otherwise unredeemable sub-token share dust.
Stand-in redemption dust remains unallocated in the source reserve.

For 1,000 tokens held 30 days at price 1 and then repriced to 1.04 with backing:
office 40 bps pays **0.328767**, advisor 60 bps pays **0.493150**, and performance
pays **6.000000** tokens. The actual short demo uses elapsed chain time, so its
advisory fees are much smaller.

## Yield source and trust

The stand-in mints receipt shares on deposit and burns them on redemption. A fixture
authority controls its price; increases require enough underlying tokens to back
all outstanding shares. The demo donates 40 mock tokens before setting price 1.04.
A cumulative price-time index provides exact integration for piecewise-constant prices.

Clients control withdrawals. **The program can collect agreed fees without a fresh
client signature, including fees that reduce principal.** Upgrade authorities can
change program behavior. The mock mint and source price authority remain fixture
controls; this is not market-generated yield or real-dollar-backed USDC.

Squads governance requires a canonical, unmanaged 2-of-3 multisig with three distinct
members holding full permissions. A single config authority would bypass that
threshold and is rejected by the handoff script. Per the user's autonomous setup
instruction, the initial three signer keys are generated locally; onchain threshold
enforcement does not establish independent human custody.

Kamino, Privy, product UI, fiat, other strategies, and mainnet are deferred. No CPI
compatibility with Kamino is claimed.
