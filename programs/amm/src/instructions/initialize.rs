//! Pool initialization: Config PDA, LP mint, and X/Y vaults.

use anchor_lang::prelude::*;
use anchor_spl::{
    associated_token::{self, AssociatedToken},
    token_interface::{Mint, TokenInterface},
};

use crate::errors::AmmError;
use crate::state::{Config, CONFIG_SEED, LP_SEED, MAX_FEE_BPS};

/// Accounts required to initialize a new constant-product pool.
///
/// Creates the `Config` PDA and LP mint in-constraints, then creates vault ATAs
/// in the handler **after** mint/fee checks (so invalid inputs never spend CPI
/// on vault creation and surface `AmmError` cleanly).
#[derive(Accounts)]
#[instruction(seed: u64)]
pub struct Initialize<'info> {
    /// Payer and transaction signer that funds account creation.
    #[account(mut)]
    pub initializer: Signer<'info>,

    /// Mint of token X in the trading pair.
    pub mint_x: InterfaceAccount<'info, Mint>,

    /// Mint of token Y in the trading pair.
    pub mint_y: InterfaceAccount<'info, Mint>,

    /// Pool configuration PDA. Space is exactly `Config::ACCOUNT_SPACE`.
    #[account(
        init,
        payer = initializer,
        seeds = [CONFIG_SEED, seed.to_le_bytes().as_ref()],
        bump,
        space = Config::ACCOUNT_SPACE,
    )]
    pub config: Account<'info, Config>,

    /// LP share mint. Mint authority is the Config PDA.
    #[account(
        init,
        payer = initializer,
        seeds = [LP_SEED, config.key().as_ref()],
        bump,
        mint::decimals = 6,
        mint::authority = config,
        mint::token_program = token_program,
    )]
    pub mint_lp: InterfaceAccount<'info, Mint>,

    /// Vault holding reserves of token X. Created in the handler via ATA CPI.
    /// CHECK: validated and initialized in `handler` after mint distinctness checks.
    #[account(mut)]
    pub vault_x: UncheckedAccount<'info>,

    /// Vault holding reserves of token Y. Created in the handler via ATA CPI.
    /// CHECK: validated and initialized in `handler` after mint distinctness checks.
    #[account(mut)]
    pub vault_y: UncheckedAccount<'info>,

    /// SPL Token or Token-2022 program (validated Interface — anti arbitrary CPI).
    pub token_program: Interface<'info, TokenInterface>,

    /// Associated Token Account program.
    pub associated_token_program: Program<'info, AssociatedToken>,

    /// System program for account creation.
    pub system_program: Program<'info, System>,
}

/// @notice Initializes a new AMM pool (Config, LP mint, vaults X/Y).
/// @dev Persists canonical bumps in `Config` for later instructions.
/// @param ctx Accounts: initializer, mints, config PDA, mint_lp, vaults, token programs.
/// @param seed Client seed used in the Config PDA derivation.
/// @param fee Swap fee in basis points (`<= MAX_FEE_BPS`).
/// @param authority Optional admin pubkey; `None` leaves the pool without an admin.
/// @return Result<()> Ok if accounts were created and Config was written.
pub(crate) fn handler(
    ctx: Context<Initialize>,
    seed: u64,
    fee: u16,
    authority: Option<Pubkey>,
) -> Result<()> {
    require!(fee <= MAX_FEE_BPS, AmmError::InvalidFee);
    require_keys_neq!(
        ctx.accounts.mint_x.key(),
        ctx.accounts.mint_y.key(),
        AmmError::IdenticalMints
    );
    require_keys_neq!(
        ctx.accounts.vault_x.key(),
        ctx.accounts.vault_y.key(),
        AmmError::IdenticalVaults
    );

    // Create vault ATAs only after mint/fee validation so error codes stay precise.
    associated_token::create(CpiContext::new(
        ctx.accounts.associated_token_program.to_account_info(),
        associated_token::Create {
            payer: ctx.accounts.initializer.to_account_info(),
            associated_token: ctx.accounts.vault_x.to_account_info(),
            authority: ctx.accounts.config.to_account_info(),
            mint: ctx.accounts.mint_x.to_account_info(),
            system_program: ctx.accounts.system_program.to_account_info(),
            token_program: ctx.accounts.token_program.to_account_info(),
        },
    ))?;

    associated_token::create(CpiContext::new(
        ctx.accounts.associated_token_program.to_account_info(),
        associated_token::Create {
            payer: ctx.accounts.initializer.to_account_info(),
            associated_token: ctx.accounts.vault_y.to_account_info(),
            authority: ctx.accounts.config.to_account_info(),
            mint: ctx.accounts.mint_y.to_account_info(),
            system_program: ctx.accounts.system_program.to_account_info(),
            token_program: ctx.accounts.token_program.to_account_info(),
        },
    ))?;

    // Ensure client-supplied vault addresses match the canonical ATAs.
    let expected_vault_x = associated_token::get_associated_token_address_with_program_id(
        &ctx.accounts.config.key(),
        &ctx.accounts.mint_x.key(),
        &ctx.accounts.token_program.key(),
    );
    let expected_vault_y = associated_token::get_associated_token_address_with_program_id(
        &ctx.accounts.config.key(),
        &ctx.accounts.mint_y.key(),
        &ctx.accounts.token_program.key(),
    );
    require_keys_eq!(ctx.accounts.vault_x.key(), expected_vault_x);
    require_keys_eq!(ctx.accounts.vault_y.key(), expected_vault_y);

    let config = &mut ctx.accounts.config;
    config.authority = authority;
    config.mint_x = ctx.accounts.mint_x.key();
    config.mint_y = ctx.accounts.mint_y.key();
    config.mint_lp = ctx.accounts.mint_lp.key();
    config.seed = seed;
    config.fee = fee;
    config.locked = false;
    config.config_bump = ctx.bumps.config;
    config.lp_bump = ctx.bumps.mint_lp;

    Ok(())
}
