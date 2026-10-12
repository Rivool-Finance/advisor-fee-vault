# Advisor Fee Vault

Brazilian financial advisors have no way to offer their clients onchain dollar yield without holding
the clients' money. This Solana program closes that gap: the client deposits USDC into a vault that
earns yield, the program charges the office fee, the advisor fee and Rivool's performance fee
onchain and pays all three in one transaction, and only the client can withdraw. Built by
[Rivool](https://rivool.finance) for Colosseum's Crypto World's Fair.

## Status

What runs on Solana devnet, updated as each piece lands:

- [ ] Program: config, office and advisor fees under a ceiling, deposit, accrue, settle, withdraw
- [ ] Fee-maths tests in LiteSVM
- [ ] Devnet deployment (program id here)
- [ ] Demo client that runs the full loop on devnet

## Technologies

| # | Technology | Role |
|---|---|---|
| 1 | Rust + Anchor | The fee vault program |
| 2 | SPL Token | USDC deposits and the three-way fee payout |
| 3 | LiteSVM | Fee-maths tests, with the clock moved forward to show accrual |
| 4 | TypeScript Anchor client | Runs the demo loop against devnet |
| 5 | Stand-in yield vault (ours) | **The yield source in this build.** A small Anchor program of ours, with a share price we set, that the fee vault deposits into on devnet |
| 6 | Kamino Lend (USDC reserve) | **The target yield source, not integrated in this build.** Kamino runs no official USDC market on devnet, so the stand-in takes its place; it exposes the same three calls a Kamino adapter needs: deposit, redeem and exchange rate |

## How it works

The design, with accounts, instructions, fee formulas and trust rules, is in
[`context/02-fee-vault-design.md`](context/02-fee-vault-design.md).

- **Accounts:** Config (Rivool), Office, Advisor, Vault (one per yield source) and Position (one per
  client).
- **Fees:** the advisory fee accrues on the position's value over time, as office part plus advisor
  part under the office ceiling; Rivool's performance fee applies only to yield above the
  position's high-water mark. Fees come out of yield first, and the client sees net yield beside gross.
- **Trust:** the program moves principal only on the position owner's signature. The advisor can
  set fees but cannot withdraw, and there is no admin withdrawal path.

## Built during Crypto World's Fair

- 2026-10-09: chose Solana and the advisor fee vault; wrote the program design and the context pack.
- 2026-10-12: opened this repository with the context pack and this README.

## Prior work, kept elsewhere

Rivool's live platform predates the hackathon and lives in a separate private codebase: dollar yield
on Base through Deframe into Morpho vaults, Stellar with Blend, Ondo stocks on BSC and Ethereum, a PIX
on- and off-ramp, Privy wallets, the advisor and office network, and an offchain fee engine (daily
accrual, collection at withdrawal). No code from that codebase is in this repository.

Read on-chain on Base on 12 Oct 2026 (block 52,514,499):

- **160 clients** hold at least US$1 in Morpho USDC vaults through Rivool. Accounts we know belong
  to Rivool's team are not counted.
- **61** of those clients have an active advisor.
- Those 61 clients belong to **19 advisors**, and 14 of those advisors work in **11 offices**. An
  advisor or office counts when at least one of its active clients holds such a position.

## Run it

Build, test and deploy commands land here with the program.

## Next

- A Kamino Lend adapter in place of the stand-in vault.
- Advisor and client screens with Privy embedded Solana wallets.
- The program's upgrade authority on a Squads multisig.
- USDY and Ondo Global Markets on Solana for qualified clients.

## Context pack

What the team gathered on 2026-10-09 to decide the entry:

| File | What it is |
|---|---|
| `advisor-fee-vault.html` | Proposal deck (13 slides). Press N for sources on each slide |
| `submission-outline.html` | Field-by-field checklist for the form, plus the demo shot list |
| `context/00-original-draft.md` | The team's first draft, for reference |
| `context/01-decisions-and-findings.md` | Locked decisions, hackathon rules, Base vs Solana, fee model, economics, precedents, risks |
| `context/02-fee-vault-design.md` | Program accounts, instructions, fee maths, trust rules |
| `context/03-solana-opportunity-map.md` | What is live on Solana that fits Rivool, with sources |
| `context/04-solana-specialist-brief.md` | Questions for the Solana vault specialist |
| `context/05-github-plan.md` | What the repo needs by Sunday and what to cut |
| `context/06-google-doc-outline.txt` | The outline formatted to paste into the shared Google Doc |

Both HTML files are self-contained (fonts and styles embedded) and open offline.
