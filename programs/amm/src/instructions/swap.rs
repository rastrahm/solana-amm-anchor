//! Constant-product swap with fee on input.

use anchor_lang::prelude::*;
use anchor_spl::token_interface::{
    transfer_checked, Mint, TokenAccount, TokenInterface, TransferChecked,
};

use crate::errors::AmmError;
use crate::helpers::amount_out_for_swap;
use crate::state::{Config, CONFIG_SEED};

/// Accounts for swapping between token X and token Y.
///
/// Large token accounts are `Box`ed to stay under the BPF 4KB stack limit.
#[derive(Accounts)]
pub struct Swap<'info> {
    /// Trader funding the input transfer and receiving the output.
    pub user: Signer<'info>,

    /// Mint of token X (must match `config.mint_x`).
    pub mint_x: Box<InterfaceAccount<'info, Mint>>,

    /// Mint of token Y (must match `config.mint_y`).
    pub mint_y: Box<InterfaceAccount<'info, Mint>>,

    /// Pool config PDA. Uses stored canonical bumps (no client-supplied bump).
    #[account(
        has_one = mint_x,
        has_one = mint_y,
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
        constraint = user_x.key() != user_y.key() @ AmmError::IdenticalVaults
    )]
    pub user_y: Box<InterfaceAccount<'info, TokenAccount>>,

    /// SPL Token or Token-2022 program.
    pub token_program: Interface<'info, TokenInterface>,
}

/// @notice Swaps an exact input amount for the other token in the pool.
/// @dev Fee is taken on input; post-swap invariant `(x+dx)*(y-dy) >= x*y` is enforced.
/// @param ctx Swap accounts (user, mints, config, vaults, user ATAs, token program).
/// @param is_x `true` swaps X→Y; `false` swaps Y→X.
/// @param amount_in Exact input amount transferred from the user.
/// @param min_amount_out Slippage guard on the output amount.
/// @return Result<()> Ok when both transfers succeed.
pub(crate) fn handler(
    ctx: Context<Swap>,
    is_x: bool,
    amount_in: u64,
    min_amount_out: u64,
) -> Result<()> {
    require!(!ctx.accounts.config.locked, AmmError::PoolLocked);
    require!(amount_in > 0, AmmError::InvalidAmount);

    let (reserve_in, reserve_out, mint_in_decimals, mint_out_decimals) = if is_x {
        (
            ctx.accounts.vault_x.amount,
            ctx.accounts.vault_y.amount,
            ctx.accounts.mint_x.decimals,
            ctx.accounts.mint_y.decimals,
        )
    } else {
        (
            ctx.accounts.vault_y.amount,
            ctx.accounts.vault_x.amount,
            ctx.accounts.mint_y.decimals,
            ctx.accounts.mint_x.decimals,
        )
    };

    let amount_out =
        amount_out_for_swap(amount_in, reserve_in, reserve_out, ctx.accounts.config.fee)?;
    require!(amount_out >= min_amount_out, AmmError::SlippageExceeded);

    let seed_bytes = ctx.accounts.config.seed.to_le_bytes();
    let bump = [ctx.accounts.config.config_bump];
    let signer_seeds: &[&[&[u8]]] = &[&[CONFIG_SEED, seed_bytes.as_ref(), bump.as_ref()]];

    if is_x {
        // User sends X, receives Y.
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
            amount_in,
            mint_in_decimals,
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
            amount_out,
            mint_out_decimals,
        )?;
    } else {
        // User sends Y, receives X.
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
            amount_in,
            mint_in_decimals,
        )?;

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
            amount_out,
            mint_out_decimals,
        )?;
    }

    Ok(())
}
