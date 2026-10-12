# Brief for the Solana vault specialist

Context: Rivool is a Brazilian wealth platform distributed through financial advisors. We are
entering Colosseum's Crypto World's Fair (submission closes 13 Oct) with an advisor fee vault on
Solana. A client deposits USDC, yield comes from an existing Solana vault, and our program charges
a time-based advisory fee (up to 1% a year, split office and advisor) plus 15% of realised yield
above a high-water mark, paying office, advisor and Rivool onchain. Only the client can withdraw.
Fees differ per advisor and per client. Design attached (02-fee-vault-design.md).

Questions, in order of how much they change the build:

1. Yield source: for USDC, which is the best fit to call from our program today: Kamino lending
   vaults, Voltr, other? Is there a maintained example of calling it from another program?
2. Build or compose: does any vault framework already support per-position fee rates and a split
   to three or more recipients, or is our own program right?
3. Fee accounting: pooled vault with per-client positions; accrue as pending amounts, or mint fee shares?
4. Custody wording: no admin withdrawal path, upgrade authority on Squads; how would you describe
   custody to judges and clients?
5. Wallets: known issues with Privy embedded Solana wallets plus a policy-restricted advisor signer?
6. Review: can you look at the design before Saturday and a test run before we submit?
7. After the hackathon: who should we talk to about USDY and Ondo Global Markets on Solana for
   Brazilian qualified investors?

Timing: answers to 1 to 3 by Friday morning; design and test review Saturday.
Note for the pitch: describe the specialist's role as advice, never as code.
