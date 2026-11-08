//! Add liquidity: transfer X/Y into vaults and mint LP shares.

use anchor_lang::prelude::*;
use anchor_spl::{
    associated_token::AssociatedToken,
    token_interface::{
        mint_to, transfer_checked, Mint, MintTo, TokenAccount, TokenInterface, TransferChecked,
    },
};

use crate::errors::AmmError;
use crate::events::LiquidityDeposited;
use crate::helpers::{deposit_quote, MINIMUM_LIQUIDITY};
use crate::state::{Config, CONFIG_SEED, LP_SEED};

/// Accounts for depositing token X/Y and minting LP.
///
/// Large token accounts are `Box`ed to stay under the BPF 4KB stack limit.
#[event_cpi]
#[derive(Accounts)]
pub struct Deposit<'info> {
    /// Liquidity provider funding the deposit and paying ATA rent if needed.
    #[account(mut)]
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

    /// User ATA for LP tokens (created on first deposit if needed).
    #[account(
        init_if_needed,
        payer = user,
        associated_token::mint = mint_lp,
        associated_token::authority = user,
        associated_token::token_program = token_program,
    )]
    pub user_lp: Box<InterfaceAccount<'info, TokenAccount>>,

    /// Locked LP ATA owned by Config (holds `MINIMUM_LIQUIDITY` forever).
    #[account(
        init_if_needed,
        payer = user,
        associated_token::mint = mint_lp,
        associated_token::authority = config,
        associated_token::token_program = token_program,
    )]
    pub lock_lp: Box<InterfaceAccount<'info, TokenAccount>>,

    /// SPL Token or Token-2022 program.
    pub token_program: Interface<'info, TokenInterface>,

    /// Associated Token Account program.
    pub associated_token_program: Program<'info, AssociatedToken>,

    /// System program.
    pub system_program: Program<'info, System>,
}

/// @notice Deposits token X and Y into the pool and mints LP shares.
/// @dev First deposit locks `MINIMUM_LIQUIDITY` LP in `lock_lp` (Config-owned ATA).
///      Later deposits only pull the pool ratio; the excess of an unbalanced offer is not taken.
/// @param ctx Deposit accounts (user, mints, config, vaults, ATAs, token programs).
/// @param max_x Maximum token X to transfer from the user.
/// @param max_y Maximum token Y to transfer from the user.
/// @param min_lp Slippage guard: minimum LP tokens the user must receive.
/// @return Result<()> Ok when transfers and mints succeed.
pub(crate) fn handler(ctx: Context<Deposit>, max_x: u64, max_y: u64, min_lp: u64) -> Result<()> {
    require!(!ctx.accounts.config.locked, AmmError::PoolLocked);
    require!(max_x > 0 && max_y > 0, AmmError::InvalidAmount);
    require_keys_neq!(
        ctx.accounts.user_x.key(),
        ctx.accounts.user_y.key(),
        AmmError::IdenticalVaults
    );

    let reserve_x = ctx.accounts.vault_x.amount;
    let reserve_y = ctx.accounts.vault_y.amount;
    let total_lp = ctx.accounts.mint_lp.supply;

    let quote = deposit_quote(max_x, max_y, reserve_x, reserve_y, total_lp)?;
    require!(quote.lp_to_user >= min_lp, AmmError::SlippageExceeded);

    // Transfer underlying tokens into the pool vaults.
    transfer_checked(
        CpiContext::new(
            ctx.accounts.token_program.to_account_info(),
            TransferChecked {
                from: ctx.accounts.user_x.to_account_info(),
                mint: ctx.accounts.mint_x.to_account_info(),
                to: ctx.accounts.vault_x.to_account_info(),
                authority: ctx.accounts.user.to_account_info(),
            },
        ),
        quote.amount_x,
        ctx.accounts.mint_x.decimals,
    )?;

    transfer_checked(
        CpiContext::new(
            ctx.accounts.token_program.to_account_info(),
            TransferChecked {
                from: ctx.accounts.user_y.to_account_info(),
                mint: ctx.accounts.mint_y.to_account_info(),
                to: ctx.accounts.vault_y.to_account_info(),
                authority: ctx.accounts.user.to_account_info(),
            },
        ),
        quote.amount_y,
        ctx.accounts.mint_y.decimals,
    )?;

    let seed_bytes = ctx.accounts.config.seed.to_le_bytes();
    let bump = [ctx.accounts.config.config_bump];
    let signer_seeds: &[&[&[u8]]] = &[&[CONFIG_SEED, seed_bytes.as_ref(), bump.as_ref()]];

    if quote.lock_minimum {
        mint_to(
            CpiContext::new_with_signer(
                ctx.accounts.token_program.to_account_info(),
                MintTo {
                    mint: ctx.accounts.mint_lp.to_account_info(),
                    to: ctx.accounts.lock_lp.to_account_info(),
                    authority: ctx.accounts.config.to_account_info(),
                },
                signer_seeds,
            ),
            MINIMUM_LIQUIDITY,
        )?;
    }

    mint_to(
        CpiContext::new_with_signer(
            ctx.accounts.token_program.to_account_info(),
            MintTo {
                mint: ctx.accounts.mint_lp.to_account_info(),
                to: ctx.accounts.user_lp.to_account_info(),
                authority: ctx.accounts.config.to_account_info(),
            },
            signer_seeds,
        ),
        quote.lp_to_user,
    )?;

    emit_cpi!(LiquidityDeposited {
        config: ctx.accounts.config.key(),
        user: ctx.accounts.user.key(),
        amount_x: quote.amount_x,
        amount_y: quote.amount_y,
        lp_minted: quote.lp_to_user,
        lp_locked: if quote.lock_minimum {
            MINIMUM_LIQUIDITY
        } else {
            0
        },
        reserve_x: reserve_x
            .checked_add(quote.amount_x)
            .ok_or(AmmError::MathOverflow)?,
        reserve_y: reserve_y
            .checked_add(quote.amount_y)
            .ok_or(AmmError::MathOverflow)?,
    });

    Ok(())
}
