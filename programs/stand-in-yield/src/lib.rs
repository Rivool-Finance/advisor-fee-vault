use anchor_lang::prelude::*;
use anchor_spl::token::{self, Burn, Mint, MintTo, Token, TokenAccount, TransferChecked};

declare_id!("4ywtd7N6oR52qPRiZ66mX76HPpjHAXiCYqoyaNVFF1xh");

pub const PRICE_SCALE: u128 = 1_000_000_000_000;
pub const DECIMALS: u8 = 6;

#[program]
pub mod stand_in_yield {
    use super::*;

    pub fn initialize(ctx: Context<Initialize>) -> Result<()> {
        let vault = &mut ctx.accounts.vault;
        vault.authority = ctx.accounts.authority.key();
        vault.mint = ctx.accounts.mint.key();
        vault.price = PRICE_SCALE;
        vault.price_time_index = 0;
        vault.updated_at = Clock::get()?.unix_timestamp;
        vault.bump = ctx.bumps.vault;
        Ok(())
    }

    pub fn set_exchange_rate(ctx: Context<SetExchangeRate>, price: u128) -> Result<()> {
        require!(price > 0, YieldError::InvalidPrice);
        let backing = assets_for_shares(ctx.accounts.shares_mint.supply, price, true)?;
        require!(
            ctx.accounts.reserve.amount >= backing,
            YieldError::InsufficientBacking
        );
        ctx.accounts
            .vault
            .checkpoint(Clock::get()?.unix_timestamp)?;
        ctx.accounts.vault.price = price;
        emit!(ExchangeRateChanged {
            vault: ctx.accounts.vault.key(),
            price
        });
        Ok(())
    }

    pub fn deposit(ctx: Context<Transact>, assets: u64, min_shares: u64) -> Result<()> {
        require!(assets > 0, YieldError::ZeroAmount);
        let shares = u64::try_from(
            u128::from(assets)
                .checked_mul(PRICE_SCALE)
                .ok_or(YieldError::Overflow)?
                / ctx.accounts.vault.price,
        )
        .map_err(|_| YieldError::Overflow)?;
        require!(shares > 0 && shares >= min_shares, YieldError::Slippage);
        let backing = assets_for_shares(
            ctx.accounts.shares_mint.supply,
            ctx.accounts.vault.price,
            true,
        )?;
        require!(
            ctx.accounts.reserve.amount >= backing,
            YieldError::InsufficientBacking
        );
        ctx.accounts
            .vault
            .checkpoint(Clock::get()?.unix_timestamp)?;
        token::transfer_checked(
            CpiContext::new(
                token::ID,
                TransferChecked {
                    from: ctx.accounts.owner_assets.to_account_info(),
                    mint: ctx.accounts.mint.to_account_info(),
                    to: ctx.accounts.reserve.to_account_info(),
                    authority: ctx.accounts.owner.to_account_info(),
                },
            ),
            assets,
            DECIMALS,
        )?;
        let vault = &ctx.accounts.vault;
        let seeds: &[&[u8]] = &[
            b"yield",
            vault.authority.as_ref(),
            vault.mint.as_ref(),
            &[vault.bump],
        ];
        token::mint_to(
            CpiContext::new(
                token::ID,
                MintTo {
                    mint: ctx.accounts.shares_mint.to_account_info(),
                    to: ctx.accounts.owner_shares.to_account_info(),
                    authority: vault.to_account_info(),
                },
            )
            .with_signer(&[seeds]),
            shares,
        )?;
        emit!(Deposited {
            vault: vault.key(),
            owner: ctx.accounts.owner.key(),
            assets,
            shares
        });
        Ok(())
    }

    pub fn redeem(ctx: Context<Transact>, shares: u64, min_assets: u64) -> Result<()> {
        require!(shares > 0, YieldError::ZeroAmount);
        let assets = assets_for_shares(shares, ctx.accounts.vault.price, false)?;
        // A full exit may burn shares worth less than one atomic unit.
        require!(assets >= min_assets, YieldError::Slippage);
        require!(
            ctx.accounts.reserve.amount >= assets,
            YieldError::InsufficientBacking
        );
        ctx.accounts
            .vault
            .checkpoint(Clock::get()?.unix_timestamp)?;
        token::burn(
            CpiContext::new(
                token::ID,
                Burn {
                    mint: ctx.accounts.shares_mint.to_account_info(),
                    from: ctx.accounts.owner_shares.to_account_info(),
                    authority: ctx.accounts.owner.to_account_info(),
                },
            ),
            shares,
        )?;
        let vault = &ctx.accounts.vault;
        let seeds: &[&[u8]] = &[
            b"yield",
            vault.authority.as_ref(),
            vault.mint.as_ref(),
            &[vault.bump],
        ];
        token::transfer_checked(
            CpiContext::new(
                token::ID,
                TransferChecked {
                    from: ctx.accounts.reserve.to_account_info(),
                    mint: ctx.accounts.mint.to_account_info(),
                    to: ctx.accounts.owner_assets.to_account_info(),
                    authority: vault.to_account_info(),
                },
            )
            .with_signer(&[seeds]),
            assets,
            DECIMALS,
        )?;
        emit!(Redeemed {
            vault: vault.key(),
            owner: ctx.accounts.owner.key(),
            assets,
            shares
        });
        Ok(())
    }
}

#[derive(Accounts)]
pub struct Initialize<'info> {
    #[account(mut)]
    pub authority: Signer<'info>,
    #[account(constraint = mint.decimals == DECIMALS @ YieldError::InvalidMint)]
    pub mint: Account<'info, Mint>,
    #[account(init, payer = authority, space = 8 + YieldVault::INIT_SPACE,
        seeds = [b"yield", authority.key().as_ref(), mint.key().as_ref()], bump)]
    pub vault: Account<'info, YieldVault>,
    #[account(init, payer = authority, seeds = [b"shares", vault.key().as_ref()], bump,
        mint::decimals = DECIMALS, mint::authority = vault)]
    pub shares_mint: Account<'info, Mint>,
    #[account(init, payer = authority, seeds = [b"reserve", vault.key().as_ref()], bump,
        token::mint = mint, token::authority = vault)]
    pub reserve: Account<'info, TokenAccount>,
    pub token_program: Program<'info, Token>,
    pub system_program: Program<'info, System>,
}

#[derive(Accounts)]
pub struct SetExchangeRate<'info> {
    pub authority: Signer<'info>,
    #[account(mut, has_one = authority,
        seeds = [b"yield", vault.authority.as_ref(), vault.mint.as_ref()], bump = vault.bump)]
    pub vault: Account<'info, YieldVault>,
    #[account(seeds = [b"shares", vault.key().as_ref()], bump)]
    pub shares_mint: Account<'info, Mint>,
    #[account(seeds = [b"reserve", vault.key().as_ref()], bump,
        token::mint = vault.mint, token::authority = vault)]
    pub reserve: Account<'info, TokenAccount>,
}

#[derive(Accounts)]
pub struct Transact<'info> {
    pub owner: Signer<'info>,
    #[account(mut, has_one = mint,
        seeds = [b"yield", vault.authority.as_ref(), vault.mint.as_ref()], bump = vault.bump)]
    pub vault: Account<'info, YieldVault>,
    pub mint: Account<'info, Mint>,
    #[account(mut, seeds = [b"shares", vault.key().as_ref()], bump)]
    pub shares_mint: Account<'info, Mint>,
    #[account(mut, seeds = [b"reserve", vault.key().as_ref()], bump,
        token::mint = mint, token::authority = vault)]
    pub reserve: Account<'info, TokenAccount>,
    #[account(mut, token::mint = mint, token::authority = owner)]
    pub owner_assets: Account<'info, TokenAccount>,
    #[account(mut, token::mint = shares_mint, token::authority = owner)]
    pub owner_shares: Account<'info, TokenAccount>,
    pub token_program: Program<'info, Token>,
}

#[account]
#[derive(InitSpace)]
pub struct YieldVault {
    pub authority: Pubkey,
    pub mint: Pubkey,
    pub price: u128,
    pub price_time_index: u128,
    pub updated_at: i64,
    pub bump: u8,
}

impl YieldVault {
    pub fn price_index_at(&self, now: i64) -> Result<u128> {
        let elapsed = now
            .checked_sub(self.updated_at)
            .ok_or(YieldError::Overflow)?;
        require!(elapsed >= 0, YieldError::ClockWentBackwards);
        self.price
            .checked_mul(elapsed as u128)
            .and_then(|delta| self.price_time_index.checked_add(delta))
            .ok_or_else(|| error!(YieldError::Overflow))
    }

    fn checkpoint(&mut self, now: i64) -> Result<()> {
        self.price_time_index = self.price_index_at(now)?;
        self.updated_at = now;
        Ok(())
    }
}

fn assets_for_shares(shares: u64, price: u128, round_up: bool) -> Result<u64> {
    let value = u128::from(shares)
        .checked_mul(price)
        .ok_or(YieldError::Overflow)?;
    let assets = if round_up {
        value.div_ceil(PRICE_SCALE)
    } else {
        value / PRICE_SCALE
    };
    u64::try_from(assets).map_err(|_| error!(YieldError::Overflow))
}

#[event]
pub struct ExchangeRateChanged {
    pub vault: Pubkey,
    pub price: u128,
}
#[event]
pub struct Deposited {
    pub vault: Pubkey,
    pub owner: Pubkey,
    pub assets: u64,
    pub shares: u64,
}
#[event]
pub struct Redeemed {
    pub vault: Pubkey,
    pub owner: Pubkey,
    pub assets: u64,
    pub shares: u64,
}

#[error_code]
pub enum YieldError {
    #[msg("Arithmetic overflow")]
    Overflow,
    #[msg("Exchange rate must be positive")]
    InvalidPrice,
    #[msg("The reserve cannot back this exchange rate")]
    InsufficientBacking,
    #[msg("Amount must be positive")]
    ZeroAmount,
    #[msg("Output is below the requested minimum")]
    Slippage,
    #[msg("Clock moved backwards")]
    ClockWentBackwards,
    #[msg("Only six-decimal SPL token mints are supported")]
    InvalidMint,
}
