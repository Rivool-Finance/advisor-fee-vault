# GitHub plan: what judges need to see by Sunday

Colosseum reviews the repo to see that the team did significant work during the hackathon, did it
themselves, and prioritised well. They do not grade language, framework or code style. A thin repo
is fine if it is honest and the loop works; an empty one with mockups is the pattern that loses.

## Start today

- Create a new repo for the Solana build only (public preferred; private is allowed with access
  granted to Colosseum's review address). Nothing from the monorepo goes in it.
- First commit today: Anchor scaffold, `README.md` with the scope and the prior-work note,
  `docs/design.md` (from 02-fee-vault-design.md).
- Commit small and often. Commit dates are the evidence that the work is inside the window.

## Minimum that makes a credible submission

| Priority | Item | Proves |
|---|---|---|
| 1 | Program: config, office and advisor fee with ceiling check, deposit, accrue, settle, withdraw | The core claim |
| 2 | LiteSVM tests for the fee maths | It works and was thought through |
| 3 | Devnet deployment with the program id in the README | It runs |
| 4 | Test vault with settable share price (Kamino if it lands) | Yield and fees move |
| 5 | Minimal advisor and client screens with Privy | The demo loop |
| 6 | README: what was built when, how to run, what is next | Judges can follow |

If time runs out, cut from the bottom: a scripted demo against the deployed program (a CLI or a
test that prints each step) is acceptable for the demo video; screens are not.

## README sections

1. What it is (one paragraph, advisor first).
2. Status: what works on devnet today, what is stubbed.
3. Built during Crypto World's Fair: dated list.
4. Prior work, kept elsewhere: one paragraph matching the form's disclosure.
5. Architecture and fee maths (link to docs/design.md).
6. Run it: build, test, deploy.
