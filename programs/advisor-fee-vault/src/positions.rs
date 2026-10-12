use crate::{math, Advisor, Config, FeeError, Office};
use anchor_lang::prelude::*;
use anchor_spl::token::{self, Mint, Token, TokenAccount, TransferChecked};
use stand_in_yield::{program::StandInYield, YieldVault};

#[account]
#[derive(InitSpace)]
pub struct Vault {
    pub config: Pubkey,
    pub source: Pubkey,
    pub total_shares: u64,
    pub bump: u8,
}

#[account]
#[derive(InitSpace)]
pub struct Position {
    pub client: Pubkey,
    pub vault: Pubkey,
    pub office: Pubkey,
    pub advisor: Pubkey,
    pub rates: [u16; 2],
    pub recipients: [Pubkey; 3],
    pub shares: u64,
    // Redemption rounds shares up; excess underlying belongs only to this position.
    pub cash: u64,
    pub high_water_mark: u128,
    pub price_index: u128,
    pub accrued_at: i64,
    pub settled_at: i64,
    pub owed: [u64; 2],
    pub remainders: [u128; 3],
    pub bump: u8,
}

#[derive(Accounts)]
pub struct RegisterVault<'info> {
    #[account(mut)]
    pub authority: Signer<'info>,
    #[account(has_one = authority, has_one = mint, seeds = [b"config"], bump = config.bump)]
    pub config: Account<'info, Config>,
    pub mint: Account<'info, Mint>,
    #[account(has_one = mint)]
    pub source: Account<'info, YieldVault>,
    #[account(seeds = [b"shares", source.key().as_ref()], bump, seeds::program = stand_in_yield::ID)]
    pub shares_mint: Account<'info, Mint>,
    #[account(init, payer = authority, space = 8 + Vault::INIT_SPACE,
        seeds = [b"vault", config.key().as_ref(), source.key().as_ref()], bump)]
    pub vault: Account<'info, Vault>,
    #[account(init, payer = authority, seeds = [b"cash", vault.key().as_ref()], bump,
        token::mint = mint, token::authority = vault)]
    pub cash: Account<'info, TokenAccount>,
    #[account(init, payer = authority, seeds = [b"custody", vault.key().as_ref()], bump,
        token::mint = shares_mint, token::authority = vault)]
    pub custody: Account<'info, TokenAccount>,
    pub token_program: Program<'info, Token>,
    pub system_program: Program<'info, System>,
}

#[derive(Accounts)]
pub struct OpenPosition<'info> {
    #[account(mut)]
    pub client: Signer<'info>,
    #[account(seeds = [b"config"], bump = config.bump)]
    pub config: Account<'info, Config>,
    #[account(has_one = config, has_one = source,
        seeds = [b"vault", config.key().as_ref(), source.key().as_ref()], bump = vault.bump)]
    pub vault: Account<'info, Vault>,
    pub source: Account<'info, YieldVault>,
    #[account(has_one = config, seeds = [b"office", config.key().as_ref(), office.initial_authority.as_ref()], bump = office.bump)]
    pub office: Account<'info, Office>,
    #[account(has_one = office, seeds = [b"advisor", office.key().as_ref(), advisor.authority.as_ref()], bump = advisor.bump)]
    pub advisor: Account<'info, Advisor>,
    #[account(init, payer = client, space = 8 + Position::INIT_SPACE,
        seeds = [b"position", vault.key().as_ref(), client.key().as_ref()], bump)]
    pub position: Account<'info, Position>,
    pub system_program: Program<'info, System>,
}

#[derive(Accounts)]
pub struct Accrue<'info> {
    #[account(has_one = source, seeds = [b"vault", vault.config.as_ref(), source.key().as_ref()], bump = vault.bump)]
    pub vault: Account<'info, Vault>,
    pub source: Account<'info, YieldVault>,
    #[account(mut, has_one = vault, seeds = [b"position", vault.key().as_ref(), position.client.as_ref()], bump = position.bump)]
    pub position: Account<'info, Position>,
}

#[derive(Accounts)]
pub struct Settlement<'info> {
    #[account(has_one = mint, seeds = [b"config"], bump = config.bump)]
    pub config: Box<Account<'info, Config>>,
    #[account(mut, has_one = config, has_one = source,
        seeds = [b"vault", config.key().as_ref(), source.key().as_ref()], bump = vault.bump)]
    pub vault: Box<Account<'info, Vault>>,
    #[account(mut, has_one = vault, seeds = [b"position", vault.key().as_ref(), position.client.as_ref()], bump = position.bump)]
    pub position: Box<Account<'info, Position>>,
    #[account(mut, has_one = mint)]
    pub source: Box<Account<'info, YieldVault>>,
    pub mint: Box<Account<'info, Mint>>,
    #[account(mut, seeds = [b"shares", source.key().as_ref()], bump, seeds::program = stand_in_yield::ID)]
    pub shares_mint: Box<Account<'info, Mint>>,
    #[account(mut, seeds = [b"reserve", source.key().as_ref()], bump, seeds::program = stand_in_yield::ID,
        token::mint = mint, token::authority = source)]
    pub reserve: Box<Account<'info, TokenAccount>>,
    #[account(mut, seeds = [b"cash", vault.key().as_ref()], bump, token::mint = mint, token::authority = vault)]
    pub cash: Box<Account<'info, TokenAccount>>,
    #[account(mut, seeds = [b"custody", vault.key().as_ref()], bump, token::mint = shares_mint, token::authority = vault)]
    pub custody: Box<Account<'info, TokenAccount>>,
    #[account(mut, dup, token::mint = mint, token::authority = position.recipients[0])]
    pub office_assets: Box<Account<'info, TokenAccount>>,
    #[account(mut, dup, token::mint = mint, token::authority = position.recipients[1])]
    pub advisor_assets: Box<Account<'info, TokenAccount>>,
    #[account(mut, dup, token::mint = mint, token::authority = position.recipients[2])]
    pub rivool_assets: Box<Account<'info, TokenAccount>>,
    pub source_program: Program<'info, StandInYield>,
    pub token_program: Program<'info, Token>,
}

#[derive(Accounts)]
pub struct ClientTransact<'info> {
    pub client: Signer<'info>,
    #[account(mut, dup, token::mint = settlement.mint, token::authority = client)]
    pub client_assets: Box<Account<'info, TokenAccount>>,
    #[account(constraint = settlement.position.client == client.key() @ FeeError::Unauthorized)]
    pub settlement: Settlement<'info>,
}

#[derive(Accounts)]
pub struct ClosePosition<'info> {
    #[account(mut)]
    pub client: Signer<'info>,
    #[account(mut, close = client, has_one = client,
        seeds = [b"position", position.vault.as_ref(), client.key().as_ref()], bump = position.bump)]
    pub position: Account<'info, Position>,
}

pub fn register_vault(ctx: Context<RegisterVault>) -> Result<()> {
    let vault = &mut ctx.accounts.vault;
    vault.config = ctx.accounts.config.key();
    vault.source = ctx.accounts.source.key();
    vault.bump = ctx.bumps.vault;
    Ok(())
}

pub fn open_position(
    ctx: Context<OpenPosition>,
    rates: [u16; 2],
    recipients: [Pubkey; 3],
) -> Result<()> {
    let a = ctx.accounts;
    math::validate_rates(
        a.office.fee_bps,
        a.advisor.fee_bps,
        a.office.advisor_allotment_bps,
    )?;
    require!(
        rates == [a.office.fee_bps, a.advisor.fee_bps]
            && recipients == [a.office.treasury, a.advisor.treasury, a.config.treasury],
        FeeError::TermsChanged
    );
    require!(
        !recipients.contains(&a.vault.key()),
        FeeError::InvalidTreasury
    );
    let now = Clock::get()?.unix_timestamp;
    let p = &mut a.position;
    p.client = a.client.key();
    p.vault = a.vault.key();
    p.office = a.office.key();
    p.advisor = a.advisor.key();
    p.rates = rates;
    p.recipients = recipients;
    p.high_water_mark = a.source.price;
    p.price_index = a.source.price_index_at(now)?;
    p.accrued_at = now;
    p.settled_at = now;
    p.bump = ctx.bumps.position;
    Ok(())
}

impl Position {
    fn checkpoint(&mut self, source: &YieldVault, now: i64) -> Result<()> {
        let index = source.price_index_at(now)?;
        let delta = index
            .checked_sub(self.price_index)
            .ok_or(FeeError::InvalidCheckpoint)?;
        let elapsed = u128::try_from(
            now.checked_sub(self.accrued_at)
                .ok_or(FeeError::InvalidCheckpoint)?,
        )
        .map_err(|_| FeeError::InvalidCheckpoint)?;
        for i in 0..2 {
            let shares = math::advisory_fee(self.shares, delta, self.rates[i], self.remainders[i])?;
            let cash = math::advisory_fee(
                self.cash,
                elapsed
                    .checked_mul(math::PRICE_SCALE)
                    .ok_or(FeeError::Overflow)?,
                self.rates[i],
                shares.remainder,
            )?;
            self.owed[i] = self.owed[i]
                .checked_add(shares.amount)
                .and_then(|n| n.checked_add(cash.amount))
                .ok_or(FeeError::Overflow)?;
            self.remainders[i] = cash.remainder;
        }
        self.price_index = index;
        self.accrued_at = now;
        Ok(())
    }
}

pub fn accrue(ctx: Context<Accrue>) -> Result<()> {
    let now = Clock::get()?.unix_timestamp;
    ctx.accounts
        .position
        .checkpoint(&ctx.accounts.source, now)?;
    emit!(FeesAccrued {
        position: ctx.accounts.position.key(),
        owed: ctx.accounts.position.owed,
        at: now
    });
    Ok(())
}

pub fn settle(ctx: Context<Settlement>) -> Result<()> {
    let now = Clock::get()?.unix_timestamp;
    let elapsed = now
        .checked_sub(ctx.accounts.position.settled_at)
        .ok_or(FeeError::InvalidCheckpoint)?;
    require!(
        i128::from(elapsed) >= i128::from(ctx.accounts.config.settlement_interval),
        FeeError::NotDue
    );
    ctx.accounts.collect(now)
}

impl<'info> Settlement<'info> {
    fn source_accounts(&self) -> stand_in_yield::cpi::accounts::Transact<'info> {
        stand_in_yield::cpi::accounts::Transact {
            owner: self.vault.to_account_info(),
            vault: self.source.to_account_info(),
            mint: self.mint.to_account_info(),
            shares_mint: self.shares_mint.to_account_info(),
            reserve: self.reserve.to_account_info(),
            owner_assets: self.cash.to_account_info(),
            owner_shares: self.custody.to_account_info(),
            token_program: self.token_program.to_account_info(),
        }
    }

    fn redeem(&self, shares: u64) -> Result<u64> {
        if shares == 0 {
            return Ok(0);
        }
        let seeds: &[&[u8]] = &[
            b"vault",
            self.vault.config.as_ref(),
            self.vault.source.as_ref(),
            &[self.vault.bump],
        ];
        stand_in_yield::cpi::redeem(
            CpiContext::new(stand_in_yield::ID, self.source_accounts()).with_signer(&[seeds]),
            shares,
            0,
        )?;
        assets_for_shares(shares, self.source.price)
    }

    fn pay(&self, destination: AccountInfo<'info>, amount: u64) -> Result<()> {
        if amount == 0 {
            return Ok(());
        }
        let seeds: &[&[u8]] = &[
            b"vault",
            self.vault.config.as_ref(),
            self.vault.source.as_ref(),
            &[self.vault.bump],
        ];
        token::transfer_checked(
            CpiContext::new(
                token::ID,
                TransferChecked {
                    from: self.cash.to_account_info(),
                    mint: self.mint.to_account_info(),
                    to: destination,
                    authority: self.vault.to_account_info(),
                },
            )
            .with_signer(&[seeds]),
            amount,
            self.mint.decimals,
        )
    }

    fn collect(&mut self, now: i64) -> Result<()> {
        self.position.checkpoint(&self.source, now)?;
        let p = &self.position;
        let perf = math::performance_fee(
            p.shares,
            self.source.price,
            p.high_water_mark,
            p.remainders[2],
        )?;
        let assets = assets_for_shares(p.shares, self.source.price)?
            .checked_add(p.cash)
            .ok_or(FeeError::Overflow)?;
        let paid = math::cap_fees([p.owed[0], p.owed[1], perf.amount], assets);
        let total = paid
            .iter()
            .try_fold(0u64, |a, b| a.checked_add(*b))
            .ok_or(FeeError::Overflow)?;
        let shares =
            math::shares_to_redeem(total.saturating_sub(p.cash), self.source.price)?.min(p.shares);
        let redeemed = self.redeem(shares)?;
        self.pay(self.office_assets.to_account_info(), paid[0])?;
        self.pay(self.advisor_assets.to_account_info(), paid[1])?;
        self.pay(self.rivool_assets.to_account_info(), paid[2])?;
        let p = &mut self.position;
        p.cash = p
            .cash
            .checked_add(redeemed)
            .and_then(|n| n.checked_sub(total))
            .ok_or(FeeError::Overflow)?;
        p.shares = p.shares.checked_sub(shares).ok_or(FeeError::Overflow)?;
        self.vault.total_shares = self
            .vault
            .total_shares
            .checked_sub(shares)
            .ok_or(FeeError::Overflow)?;
        p.owed = [0; 2];
        p.remainders[2] = perf.remainder;
        // Debt, including sub-token fractions, never survives exhaustion into a new deposit.
        if total == assets {
            p.remainders = [0; 3];
        }
        p.high_water_mark = p.high_water_mark.max(self.source.price);
        p.settled_at = now;
        emit!(FeesSettled {
            position: p.key(),
            gross_assets: assets,
            paid,
            at: now
        });
        Ok(())
    }
}

pub fn deposit(ctx: Context<ClientTransact>, assets: u64, min_shares: u64) -> Result<()> {
    require!(assets > 0, FeeError::InvalidAmount);
    let a = ctx.accounts;
    let s = &mut a.settlement;
    s.collect(Clock::get()?.unix_timestamp)?;
    token::transfer_checked(
        CpiContext::new(
            token::ID,
            TransferChecked {
                from: a.client_assets.to_account_info(),
                mint: s.mint.to_account_info(),
                to: s.cash.to_account_info(),
                authority: a.client.to_account_info(),
            },
        ),
        assets,
        s.mint.decimals,
    )?;
    let shares = u64::try_from((u128::from(assets) * math::PRICE_SCALE) / s.source.price)
        .map_err(|_| FeeError::Overflow)?;
    let seeds: &[&[u8]] = &[
        b"vault",
        s.vault.config.as_ref(),
        s.vault.source.as_ref(),
        &[s.vault.bump],
    ];
    stand_in_yield::cpi::deposit(
        CpiContext::new(stand_in_yield::ID, s.source_accounts()).with_signer(&[seeds]),
        assets,
        min_shares,
    )?;
    let p = &mut s.position;
    p.high_water_mark =
        math::blended_high_water_mark(p.shares, p.high_water_mark, shares, s.source.price)?;
    p.shares = p.shares.checked_add(shares).ok_or(FeeError::Overflow)?;
    s.vault.total_shares = s
        .vault
        .total_shares
        .checked_add(shares)
        .ok_or(FeeError::Overflow)?;
    emit!(PositionDeposited {
        position: p.key(),
        assets,
        shares
    });
    Ok(())
}

pub fn withdraw(ctx: Context<ClientTransact>, shares: Option<u64>, min_assets: u64) -> Result<()> {
    let a = ctx.accounts;
    let s = &mut a.settlement;
    s.collect(Clock::get()?.unix_timestamp)?;
    let p = &s.position;
    let shares = shares.unwrap_or(p.shares);
    require!(
        shares <= p.shares && (shares > 0 || p.shares == 0),
        FeeError::InvalidAmount
    );
    let cash = if shares == p.shares {
        p.cash
    } else {
        (u128::from(p.cash) * u128::from(shares) / u128::from(p.shares)) as u64
    };
    let assets = s
        .redeem(shares)?
        .checked_add(cash)
        .ok_or(FeeError::Overflow)?;
    require!(assets >= min_assets, FeeError::Slippage);
    s.pay(a.client_assets.to_account_info(), assets)?;
    let p = &mut s.position;
    p.shares -= shares;
    p.cash -= cash;
    s.vault.total_shares = s
        .vault
        .total_shares
        .checked_sub(shares)
        .ok_or(FeeError::Overflow)?;
    if p.shares == 0 && p.cash == 0 {
        p.remainders = [0; 3];
    }
    emit!(PositionWithdrawn {
        position: p.key(),
        assets,
        shares
    });
    Ok(())
}

fn assets_for_shares(shares: u64, price: u128) -> Result<u64> {
    u64::try_from(
        u128::from(shares)
            .checked_mul(price)
            .ok_or(FeeError::Overflow)?
            / math::PRICE_SCALE,
    )
    .map_err(|_| error!(FeeError::Overflow))
}

#[event]
pub struct FeesAccrued {
    pub position: Pubkey,
    pub owed: [u64; 2],
    pub at: i64,
}
#[event]
pub struct FeesSettled {
    pub position: Pubkey,
    pub gross_assets: u64,
    pub paid: [u64; 3],
    pub at: i64,
}
#[event]
pub struct PositionDeposited {
    pub position: Pubkey,
    pub assets: u64,
    pub shares: u64,
}
#[event]
pub struct PositionWithdrawn {
    pub position: Pubkey,
    pub assets: u64,
    pub shares: u64,
}
