//! Admin instructions: pause/resume the pool and transfer/renounce the authority.

use anchor_lang::prelude::*;

use crate::errors::AmmError;
use crate::events::{AuthorityUpdated, LockedUpdated};
use crate::state::{Config, CONFIG_SEED};

/// Accounts for admin updates on an existing pool.
///
/// Only the current `config.authority` may sign; pools initialized (or renounced) with
/// `authority = None` are immutable and always fail with `Unauthorized`.
#[event_cpi]
#[derive(Accounts)]
pub struct UpdateConfig<'info> {
    /// Current pool admin.
    pub authority: Signer<'info>,

    /// Pool config PDA. Uses the stored canonical bump.
    #[account(
        mut,
        seeds = [CONFIG_SEED, config.seed.to_le_bytes().as_ref()],
        bump = config.config_bump,
        constraint = config.authority == Some(authority.key()) @ AmmError::Unauthorized,
    )]
    pub config: Account<'info, Config>,
}

/// @notice Pauses (`true`) or resumes (`false`) deposits and swaps.
/// @dev Withdrawals ignore `locked` so a pause can never trap LP funds.
/// @param ctx `UpdateConfig` accounts.
/// @param locked New pause flag.
/// @return Result<()> Ok when stored.
pub(crate) fn set_locked_handler(ctx: Context<UpdateConfig>, locked: bool) -> Result<()> {
    ctx.accounts.config.locked = locked;

    emit_cpi!(LockedUpdated {
        config: ctx.accounts.config.key(),
        authority: ctx.accounts.authority.key(),
        locked,
    });

    Ok(())
}

/// @notice Transfers the admin role, or renounces it with `None` (irreversible).
/// @param ctx `UpdateConfig` accounts.
/// @param new_authority Next admin or `None`.
/// @return Result<()> Ok when stored.
pub(crate) fn set_authority_handler(
    ctx: Context<UpdateConfig>,
    new_authority: Option<Pubkey>,
) -> Result<()> {
    let previous_authority = ctx.accounts.config.authority;
    ctx.accounts.config.authority = new_authority;

    emit_cpi!(AuthorityUpdated {
        previous_authority,
        new_authority,
        config: ctx.accounts.config.key(),
    });

    Ok(())
}
