# Decisions and findings

Session of 2026-10-09. Code facts from rivool-monorepo `origin/development` 83acca0bc (2026-10-08).

## Locked decisions (do not relitigate)

- **Entry:** a dollar yield vault on Solana that collects the advisory fee. Rivool develops the
  fee vault program; yield comes from an existing Solana vault (Kamino USDC first, USDY later).
- **Chain:** Solana. Base and Ethereum appear only in the prior-work disclosure.
- **Edge:** advisor distribution. The vault is the mechanism; the advisor network is the moat.
- **Scope:** one working loop (advisor sets fee, client deposits, yield accrues, fee splits
  onchain to office, advisor and Rivool, client withdraws). Stocks, BRL, cards, rebalancing are out.
- **Next step after the hackathon:** Ondo Global Markets and USDY on Solana for qualified clients.
- **Wording:** form answers are written by the team in its own words.

## Hackathon facts

- Crypto World's Fair, 14 Sep to 12 Oct; submission deadline 2026-10-13 06:59 UTC (03:59 BRT).
- Only work done inside the window is judged. All relevant past development must be disclosed;
  misrepresentation can lead to disqualification, bans and revoked prizes.
- Overall awards on any chain: Grand Prize US$30k, next 20 projects US$15k each.
- Solana Ecosystem track: US$100k pool, 10 x US$10k. Other ecosystem tracks publish no amounts.
- Judging: founder-market fit, insight, product and execution, market size, communication,
  viability, traction.
- Repo: public, or private with access granted to Colosseum's hackathon review address. They look
  for significant work in the window, done by the team, prioritised well.

## Why Solana, not Base (git history check)

- Base is the production chain: five Morpho USDC vaults through Deframe
  (`apps/api/src/config/strategies.json`), USDC fee collection via pre-signed
  transferWithAuthorization (`apps/api/src/services/presigned-fee-submitter.service.ts`),
  Base-funded Ondo stocks.
- Work inside the window is hardening: Ondo on Ethereum (#498), fee module (#1119, #1122),
  withdraw indexer (#994, #986), Tron fees (#521, #496). No new onchain artifact.
- Rivool has no onchain code of its own (only two dead QuestRegistry contracts, removed f64b45b2a).
- Rivool has no Solana code. The `"solana"` value in `apps/api/src/schemas/investor.schema.ts:132`
  belongs to the retired Unblock PIX rail and accepts only 0x addresses.

## Fee model today

- Advisory fee on managed balance, office part + advisor part, up to 1% a year
  (`apps/api/src/config/aum-fee.ts:12`); advisor limit under office ceiling
  (`apps/api/src/fees/advisory/policy.ts`, `advisorFeeLimit`).
- Rivool performance fee 15% of realised yield, never principal (wiki commission note; not
  re-verified in code).
- Accrued daily offchain, charged at withdrawal as USDC transfers, reconciled by indexers.

## Economics (illustrative, per US$100k client per year, 4 to 8% yield)

- Yield vault: Rivool about US$600 to 1,200; advisor and office up to US$1,000; every client.
- Ondo stocks: Rivool near zero for buy and hold (US$0.05 per trade plus a proposed 0 to 1%);
  advisor up to 1%; qualified investors only in Brazil.
- Fee-stack tension: at 4% yield the client keeps about 2.4%; at 8% about 5.8%. Decide caps.
- Composing with Kamino directly removes the Deframe layer (`deframe_fee` exists in config; size unknown).

## Precedents (Colosseum, winners only)

- Fee routers (Zarnith, Splitter, Reflink, SplitPact): no award.
- GLAM, 2nd DeFi & Payments, Renaissance: asset-management infrastructure.
- Myfye, University Prize DeFi, Breakout: emerging-market RWA app, embedded wallets, USDY.
- LocalPay (3rd Stablecoins, Breakout), Credible Finance (2nd Stablecoins, Cypherpunk).
- Reflect Protocol won the Grand Prize at Radar; Reflect sponsors this Grand Prize.
- None had an advisor channel.

## Risks

- Build capacity: someone must write and test the Anchor program by Sunday.
- Disclosure: list the whole live platform, not only Morpho.
- Who may charge a fee-based advisory fee in Brazil (consultor vs assessor).
- Brazilian Central Bank Res. 519 to 521: reported compliance deadline 30 Oct 2026; stablecoin
  transfers to self-custody wallets under FX rules. From a Notabene summary; take to legal.
- Unverified: Reflect on Solana mainnet, USDY terms for Brazil, Kamino permissionless vault
  creation, Rain settlement on Solana. Avoid Drift (secondary reports of an April 2026 exploit).

## Open inputs

1. Who writes the Anchor program and who builds the screens.
2. Fee design (caps, one fee per balance type).
3. Confirmed network numbers: offices, advisors, clients through advisors, assets through advisors.
4. The Solana vault specialist's answers (yield source, framework, accounting).
