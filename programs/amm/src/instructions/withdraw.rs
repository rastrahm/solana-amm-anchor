//! Remove liquidity: burn LP and return proportional X/Y from vaults.

use anchor_lang::prelude::*;
use anchor_spl::token_interface::{
    burn, transfer_checked, Burn, Mint, TokenAccount, TokenInterface, TransferChecked,
};

use crate::errors::AmmError;
use crate::helpers::amounts_for_withdraw;
use crate::state::{Config, CONFIG_SEED, LP_SEED};

/// Accounts for withdrawing liquidity by burning LP tokens.
///
/// Large token accounts are `Box`ed to stay under the BPF 4KB stack limit.
#[derive(Accounts)]
pub struct Withdraw<'info> {
    /// Liquidity provider burning LP and receiving X/Y.
    pub user: Signer<'info>,

    /// Mint of token X (must match `config.mint_x`).
    pub mint_x: Box<InterfaceAccount<'info, Mint>>,

    /// Mint of token Y (must match `config.mint_y`).
    pub mint_y: Box<InterfaceAccount<'info, Mint>>,

    /// LP mint controlled by the Config PDA.
    #[account(
        mut,
        seeds = [LP_SEED, config.key().as_ref()],
        bump = config.lp_bump,
    )]
    pub mint_lp: Box<InterfaceAccount<'info, Mint>>,

    /// Pool config PDA. Uses stored canonical bumps (no client-supplied bump).
    #[account(
        has_one = mint_x,
        has_one = mint_y,
        has_one = mint_lp,
        seeds = [CONFIG_SEED, config.seed.to_le_bytes().as_ref()],
        bump = config.config_bump,
    )]
    pub config: Box<Account<'info, Config>>,

    /// Pool vault for token X.
    #[account(
        mut,
        associated_token::mint = mint_x,
        associated_token::authority = config,
        associated_token::token_program = token_program,
    )]
    pub vault_x: Box<InterfaceAccount<'info, TokenAccount>>,

    /// Pool vault for token Y.
    #[account(
        mut,
        associated_token::mint = mint_y,
        associated_token::authority = config,
        associated_token::token_program = token_program,
        constraint = vault_x.key() != vault_y.key() @ AmmError::IdenticalVaults
    )]
    pub vault_y: Box<InterfaceAccount<'info, TokenAccount>>,

    /// User ATA for token X.
    #[account(
        mut,
        associated_token::mint = mint_x,
        associated_token::authority = user,
        associated_token::token_program = token_program,
    )]
    pub user_x: Box<InterfaceAccount<'info, TokenAccount>>,

    /// User ATA for token Y.
    #[account(
        mut,
        associated_token::mint = mint_y,
        associated_token::authority = user,
        associated_token::token_program = token_program,
    )]
    pub user_y: Box<InterfaceAccount<'info, TokenAccount>>,

    /// User ATA holding LP tokens to burn.
    #[account(
        mut,
        associated_token::mint = mint_lp,
        associated_token::authority = user,
        associated_token::token_program = token_program,
    )]
    pub user_lp: Box<InterfaceAccount<'info, TokenAccount>>,

    /// SPL Token or Token-2022 program.
    pub token_program: Interface<'info, TokenInterface>,
}

/// @notice Burns LP tokens and withdraws proportional X/Y from the pool vaults.
/// @dev Floor division favors the pool; supply cannot fall below `MINIMUM_LIQUIDITY`.
/// @param ctx Withdraw accounts (user, mints, config, vaults, user ATAs, token program).
/// @param lp_amount Exact LP amount to burn from the user.
/// @param min_x Slippage guard: minimum token X received.
/// @param min_y Slippage guard: minimum token Y received.
/// @return Result<()> Ok when burn and transfers succeed.
pub(crate) fn handler(
    ctx: Context<Withdraw>,
    lp_amount: u64,
    min_x: u64,
    min_y: u64,
) -> Result<()> {
    require!(!ctx.accounts.config.locked, AmmError::PoolLocked);
    require!(lp_amount > 0, AmmError::InvalidAmount);
    require!(
        ctx.accounts.user_lp.amount >= lp_amount,
        AmmError::InsufficientLiquidity
    );
    require_keys_neq!(
        ctx.accounts.user_x.key(),
        ctx.accounts.user_y.key(),
        AmmError::IdenticalVaults
    );

    let reserve_x = ctx.accounts.vault_x.amount;
    let reserve_y = ctx.accounts.vault_y.amount;
    let total_lp = ctx.accounts.mint_lp.supply;

    let (amount_x, amount_y) = amounts_for_withdraw(lp_amount, reserve_x, reserve_y, total_lp)?;
    require!(
        amount_x >= min_x && amount_y >= min_y,
        AmmError::SlippageExceeded
    );

    // Burn LP from the user first, then send underlying tokens.
    burn(
        CpiContext::new(
            ctx.accounts.token_program.to_account_info(),
            Burn {
                mint: ctx.accounts.mint_lp.to_account_info(),
                from: ctx.accounts.user_lp.to_account_info(),
                authority: ctx.accounts.user.to_account_info(),
            },
        ),
        lp_amount,
    )?;

    let seed_bytes = ctx.accounts.config.seed.to_le_bytes();
    let bump = [ctx.accounts.config.config_bump];
    let signer_seeds: &[&[&[u8]]] = &[&[CONFIG_SEED, seed_bytes.as_ref(), bump.as_ref()]];

    transfer_checked(
        CpiContext::new_with_signer(
            ctx.accounts.token_program.to_account_info(),
            TransferChecked {
                from: ctx.accounts.vault_x.to_account_info(),
                mint: ctx.accounts.mint_x.to_account_info(),
                to: ctx.accounts.user_x.to_account_info(),
                authority: ctx.accounts.config.to_account_info(),
            },
            signer_seeds,
        ),
        amount_x,
        ctx.accounts.mint_x.decimals,
    )?;

    transfer_checked(
        CpiContext::new_with_signer(
            ctx.accounts.token_program.to_account_info(),
            TransferChecked {
                from: ctx.accounts.vault_y.to_account_info(),
                mint: ctx.accounts.mint_y.to_account_info(),
                to: ctx.accounts.user_y.to_account_info(),
                authority: ctx.accounts.config.to_account_info(),
            },
            signer_seeds,
        ),
        amount_y,
        ctx.accounts.mint_y.decimals,
    )?;

    Ok(())
}
