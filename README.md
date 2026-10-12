# Rivool × Colosseum Crypto World's Fair: context pack

Everything gathered on 2026-10-09 for Rivool's entry. Submission closes 13 Oct 2026, 03:59 BRT.

**The entry:** a dollar yield vault on Solana that collects the advisory fee. Rivool builds the fee
vault program; yield comes from an existing Solana vault (Kamino USDC first). The edge is advisor
distribution.

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

Open inputs: who builds; fee design; confirmed network numbers; the specialist's answers.

## Technologies

| # | Technology | Role |
|---|---|---|
| 1 | Rust + Anchor | The fee vault program |
| 2 | SPL Token | USDC deposits and the three-way fee payout |
| 3 | LiteSVM | Fee-maths tests, with the clock moved forward to show accrual |
| 4 | TypeScript Anchor client | Runs the demo loop against devnet |
| 5 | Stand-in yield vault (ours) | **The yield source in this build.** A small Anchor program of ours, with a share price we set, that the fee vault deposits into on devnet |
| 6 | Kamino Lend (USDC reserve) | **The target yield source, not integrated in this build.** Kamino runs no official USDC market on devnet, so the stand-in takes its place; it exposes the same three calls a Kamino adapter needs: deposit, redeem and exchange rate |
