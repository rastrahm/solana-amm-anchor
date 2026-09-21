use anchor_lang::prelude::*;

/// Accounts for the Phase 0 smoke `initialize` instruction.
///
/// Later phases will expand this context with Config PDA, vaults, and mints.
#[derive(Accounts)]
pub struct Initialize {}

/// @notice Smoke handler that confirms the program is reachable on-chain.
/// @dev No state writes; replaced in Phase 2 with real pool initialization.
/// @param _ctx Empty `Initialize` accounts context.
/// @return Result<()> Always Ok on successful invocation.
pub fn handler(_ctx: Context<Initialize>) -> Result<()> {
    msg!("amm scaffold ok: {}", crate::ID);
    Ok(())
}
