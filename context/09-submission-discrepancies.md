# Submission discrepancy checklist

Review checklist only, not answers to paste into Colosseum's application. Checked
against the repository and the connected Colosseum draft on 2026-10-12 using
[colosseum-copilot](https://copilot.colosseum.com/). The team owns its final claims,
application, presentation and demo recording.

- [ ] Replace claims that Kamino is integrated. This build uses our funded,
  controlled stand-in; Kamino is a future integration without verified CPI compatibility.
- [ ] Identify the asset as six-decimal **mock USDC**. Its mint and controlled yield
  do not establish dollar backing or market-generated returns.
- [ ] Correct claims that all funds move only with a client signature. Client
  withdrawals require that signature; agreed fees can settle permissionlessly and
  advisory fees can reduce principal. Upgrade authority remains a trust assumption.
- [ ] Remove unimplemented mobile, multichain, Privy, fiat and stock features from
  the implemented-feature list. Distinguish the broader Rivool platform from this entry.
- [ ] Substantiate office, advisor and investor counts with dated evidence and
  definitions. Do not treat draft figures as verified adoption of this Solana program.
- [ ] Disclose prior platform work and identify the new hackathon code separately.
- [ ] Use public devnet transaction links only after confirming deployment and
  balances. Local-validator signatures are not public deployment evidence.
- [ ] State the 60-second demo interval and normal 30-day interval. The demo does
  not accelerate annual fee math; deterministic 30-day amounts come from LiteSVM.
- [ ] Verify the actual Squads threshold and program authorities on devnet. Initially
  generated signer keys live on one machine; do not imply independent team custody.
- [ ] Provide a repository link, a 2–3-minute presentation and a demo no longer than
  three minutes. Review [Colosseum's requirements](https://colosseum.com/hackathon#h-faq-11)
  and [official rules](https://colosseum.com/legal/Crypto%20World%27s%20Fair%20Hackathon%20Rules.pdf)
  before submission. The confirmed deadline is 13 October 2026, 03:59 BRT.

The proposed recording sequence is configuration rejection, client deposit, funded
yield, keeper settlement with three visible transfers, rejected unauthorized exit,
and successful client withdrawal. Use the script's actual gross/fee/net outputs.
No submission or application-field edits have been made by this implementation work.
