use anchor_lang::prelude::*;
use anchor_spl::token::Mint;

pub mod math;
pub mod positions;
use positions::*;

declare_id!("F4Q5gatG2GjEt8L62Q71z7jctaitHRxunJDDnbdo6oXm");

pub const MAX_ADVISORY_BPS: u16 = 100;
pub const PERFORMANCE_BPS: u16 = 1500;
pub const MONTH_SECONDS: u64 = 30 * 24 * 60 * 60;
pub const DEMO_INTERVAL_SECONDS: u64 = 60;

#[program]
pub mod advisor_fee_vault {
    use super::*;

    pub fn register_vault(ctx: Context<RegisterVault>) -> Result<()> {
        positions::register_vault(ctx)
    }

    pub fn open_position(
        ctx: Context<OpenPosition>,
        rates: [u16; 2],
        recipients: [Pubkey; 3],
    ) -> Result<()> {
        positions::open_position(ctx, rates, recipients)
    }

    pub fn accrue(ctx: Context<Accrue>) -> Result<()> {
        positions::accrue(ctx)
    }

    pub fn settle(ctx: Context<Settlement>) -> Result<()> {
        positions::settle(ctx)
    }

    pub fn deposit(ctx: Context<ClientTransact>, assets: u64, min_shares: u64) -> Result<()> {
        positions::deposit(ctx, assets, min_shares)
    }

    pub fn withdraw(
        ctx: Context<ClientTransact>,
        shares: Option<u64>,
        min_assets: u64,
    ) -> Result<()> {
        positions::withdraw(ctx, shares, min_assets)
    }

    pub fn close_position(ctx: Context<ClosePosition>) -> Result<()> {
        require!(
            ctx.accounts.position.shares == 0 && ctx.accounts.position.cash == 0,
            FeeError::PositionFunded
        );
        Ok(())
    }

    pub fn transfer_config_authority(
        ctx: Context<TransferConfigAuthority>,
        authority: Pubkey,
    ) -> Result<()> {
        require!(authority != Pubkey::default(), FeeError::Unauthorized);
        ctx.accounts.config.authority = authority;
        emit!(AuthorityTransferred {
            account: ctx.accounts.config.key(),
            authority
        });
        Ok(())
    }

    pub fn transfer_office_authority(ctx: Context<SetOfficeFee>, authority: Pubkey) -> Result<()> {
        require!(authority != Pubkey::default(), FeeError::Unauthorized);
        ctx.accounts.office.authority = authority;
        emit!(AuthorityTransferred {
            account: ctx.accounts.office.key(),
            authority
        });
        Ok(())
    }

    pub fn initialize_config(
        ctx: Context<InitializeConfig>,
        treasury: Pubkey,
        demo: bool,
    ) -> Result<()> {
        require!(treasury != Pubkey::default(), FeeError::InvalidTreasury);
        let config = &mut ctx.accounts.config;
        config.authority = ctx.accounts.authority.key();
        config.mint = ctx.accounts.mint.key();
        config.treasury = treasury;
        config.settlement_interval = if demo {
            DEMO_INTERVAL_SECONDS
        } else {
            MONTH_SECONDS
        };
        config.bump = ctx.bumps.config;
        emit!(ConfigCreated {
            config: config.key(),
            authority: config.authority,
            demo
        });
        Ok(())
    }

    pub fn register_office(
        ctx: Context<RegisterOffice>,
        treasury: Pubkey,
        fee_bps: u16,
        advisor_allotment_bps: u16,
    ) -> Result<()> {
        math::validate_rates(fee_bps, 0, advisor_allotment_bps)?;
        require!(treasury != Pubkey::default(), FeeError::InvalidTreasury);
        let office = &mut ctx.accounts.office;
        office.config = ctx.accounts.config.key();
        office.authority = ctx.accounts.office_authority.key();
        office.initial_authority = office.authority;
        office.treasury = treasury;
        office.fee_bps = fee_bps;
        office.advisor_allotment_bps = advisor_allotment_bps;
        office.bump = ctx.bumps.office;
        emit!(OfficeFeeSet {
            office: office.key(),
            fee_bps,
            advisor_allotment_bps
        });
        Ok(())
    }

    pub fn set_office_fee(
        ctx: Context<SetOfficeFee>,
        fee_bps: u16,
        advisor_allotment_bps: u16,
    ) -> Result<()> {
        math::validate_rates(fee_bps, 0, advisor_allotment_bps)?;
        ctx.accounts.office.fee_bps = fee_bps;
        ctx.accounts.office.advisor_allotment_bps = advisor_allotment_bps;
        emit!(OfficeFeeSet {
            office: ctx.accounts.office.key(),
            fee_bps,
            advisor_allotment_bps
        });
        Ok(())
    }

    pub fn register_advisor(
        ctx: Context<RegisterAdvisor>,
        treasury: Pubkey,
        fee_bps: u16,
    ) -> Result<()> {
        let office = &ctx.accounts.office;
        math::validate_rates(office.fee_bps, fee_bps, office.advisor_allotment_bps)?;
        require!(treasury != Pubkey::default(), FeeError::InvalidTreasury);
        let advisor = &mut ctx.accounts.advisor;
        advisor.office = office.key();
        advisor.authority = ctx.accounts.advisor_authority.key();
        advisor.treasury = treasury;
        advisor.fee_bps = fee_bps;
        advisor.bump = ctx.bumps.advisor;
        emit!(AdvisorFeeSet {
            advisor: advisor.key(),
            fee_bps
        });
        Ok(())
    }

    pub fn set_advisor_fee(ctx: Context<SetAdvisorFee>, fee_bps: u16) -> Result<()> {
        let office = &ctx.accounts.office;
        math::validate_rates(office.fee_bps, fee_bps, office.advisor_allotment_bps)?;
        ctx.accounts.advisor.fee_bps = fee_bps;
        emit!(AdvisorFeeSet {
            advisor: ctx.accounts.advisor.key(),
            fee_bps
        });
        Ok(())
    }
}

#[derive(Accounts)]
pub struct TransferConfigAuthority<'info> {
    pub authority: Signer<'info>,
    #[account(mut, has_one = authority, seeds = [b"config"], bump = config.bump)]
    pub config: Account<'info, Config>,
}

#[event]
pub struct AuthorityTransferred {
    pub account: Pubkey,
    pub authority: Pubkey,
}

#[derive(Accounts)]
pub struct InitializeConfig<'info> {
    #[account(mut)]
    pub authority: Signer<'info>,
    #[account(constraint = program.programdata_address()? == Some(program_data.key()))]
    pub program: Program<'info, crate::program::AdvisorFeeVault>,
    #[account(constraint = program_data.upgrade_authority_address == Some(authority.key()) @ FeeError::Unauthorized)]
    pub program_data: Account<'info, ProgramData>,
    #[account(constraint = mint.decimals == 6 @ FeeError::InvalidMint)]
    pub mint: Account<'info, Mint>,
    #[account(init, payer = authority, space = 8 + Config::INIT_SPACE, seeds = [b"config"], bump)]
    pub config: Account<'info, Config>,
    pub system_program: Program<'info, System>,
}

#[derive(Accounts)]
pub struct RegisterOffice<'info> {
    #[account(mut)]
    pub authority: Signer<'info>,
    #[account(has_one = authority, seeds = [b"config"], bump = config.bump)]
    pub config: Account<'info, Config>,
    /// CHECK: Only recorded as an authority; registration does not exercise its privileges.
    pub office_authority: UncheckedAccount<'info>,
    #[account(init, payer = authority, space = 8 + Office::INIT_SPACE,
        seeds = [b"office", config.key().as_ref(), office_authority.key().as_ref()], bump)]
    pub office: Account<'info, Office>,
    pub system_program: Program<'info, System>,
}

#[derive(Accounts)]
pub struct SetOfficeFee<'info> {
    pub authority: Signer<'info>,
    #[account(mut, has_one = authority,
        seeds = [b"office", office.config.as_ref(), office.initial_authority.as_ref()], bump = office.bump)]
    pub office: Account<'info, Office>,
}

#[derive(Accounts)]
pub struct RegisterAdvisor<'info> {
    #[account(mut)]
    pub authority: Signer<'info>,
    #[account(has_one = authority,
        seeds = [b"office", office.config.as_ref(), office.initial_authority.as_ref()], bump = office.bump)]
    pub office: Account<'info, Office>,
    /// CHECK: Only recorded as an authority; registration does not exercise its privileges.
    pub advisor_authority: UncheckedAccount<'info>,
    #[account(init, payer = authority, space = 8 + Advisor::INIT_SPACE,
        seeds = [b"advisor", office.key().as_ref(), advisor_authority.key().as_ref()], bump)]
    pub advisor: Account<'info, Advisor>,
    pub system_program: Program<'info, System>,
}

#[derive(Accounts)]
pub struct SetAdvisorFee<'info> {
    pub authority: Signer<'info>,
    #[account(seeds = [b"office", office.config.as_ref(), office.initial_authority.as_ref()], bump = office.bump)]
    pub office: Account<'info, Office>,
    #[account(mut, has_one = authority, has_one = office,
        seeds = [b"advisor", office.key().as_ref(), authority.key().as_ref()], bump = advisor.bump)]
    pub advisor: Account<'info, Advisor>,
}

#[account]
#[derive(InitSpace)]
pub struct Config {
    pub authority: Pubkey,
    pub mint: Pubkey,
    pub treasury: Pubkey,
    pub settlement_interval: u64,
    pub bump: u8,
}

#[account]
#[derive(InitSpace)]
pub struct Office {
    pub config: Pubkey,
    pub initial_authority: Pubkey,
    pub authority: Pubkey,
    pub treasury: Pubkey,
    pub fee_bps: u16,
    pub advisor_allotment_bps: u16,
    pub bump: u8,
}

#[account]
#[derive(InitSpace)]
pub struct Advisor {
    pub office: Pubkey,
    pub authority: Pubkey,
    pub treasury: Pubkey,
    pub fee_bps: u16,
    pub bump: u8,
}

#[event]
pub struct ConfigCreated {
    pub config: Pubkey,
    pub authority: Pubkey,
    pub demo: bool,
}
#[event]
pub struct OfficeFeeSet {
    pub office: Pubkey,
    pub fee_bps: u16,
    pub advisor_allotment_bps: u16,
}
#[event]
pub struct AdvisorFeeSet {
    pub advisor: Pubkey,
    pub fee_bps: u16,
}

#[error_code]
pub enum FeeError {
    #[msg("Position still owns assets")]
    PositionFunded,
    #[msg("Accepted terms changed before signing")]
    TermsChanged,
    #[msg("Settlement interval has not elapsed")]
    NotDue,
    #[msg("Amount must be positive and within position balance")]
    InvalidAmount,
    #[msg("Minimum output not met")]
    Slippage,
    #[msg("Clock or price index moved backwards")]
    InvalidCheckpoint,

    #[msg("Office fee and advisor allotment must fit within 100 basis points")]
    FeeCeiling,
    #[msg("Advisor fee exceeds the office allotment")]
    AdvisorAllotment,
    #[msg("Arithmetic overflow")]
    Overflow,
    #[msg("Invalid fixed-point input or remainder")]
    InvalidMathInput,
    #[msg("Required authority did not sign")]
    Unauthorized,
    #[msg("Only six-decimal SPL token mints are supported")]
    InvalidMint,
    #[msg("Treasury must be a nonzero public key")]
    InvalidTreasury,
}
