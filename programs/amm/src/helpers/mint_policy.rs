//! Pool mint admission policy: no freeze authority, Token-2022 extension allowlist.

use anchor_lang::prelude::*;
use anchor_spl::{
    token_2022::spl_token_2022::{
        self,
        extension::{BaseStateWithExtensions, ExtensionType, StateWithExtensions},
    },
    token_interface::Mint,
};

use crate::errors::AmmError;

/// Extensions that only carry descriptive data and never alter balances, transfers or custody.
///
/// Anything else (TransferFeeConfig, PermanentDelegate, TransferHook, DefaultAccountState,
/// NonTransferable, ConfidentialTransfer*, ...) breaks the `vault.amount` reserve accounting
/// or lets a third party move/freeze pool funds.
const ALLOWED_MINT_EXTENSIONS: [ExtensionType; 6] = [
    ExtensionType::MetadataPointer,
    ExtensionType::TokenMetadata,
    ExtensionType::GroupPointer,
    ExtensionType::TokenGroup,
    ExtensionType::GroupMemberPointer,
    ExtensionType::TokenGroupMember,
];

/// Rejects mints that could freeze the vaults or break reserve accounting.
///
/// - Any mint with a `freeze_authority` → `MintHasFreezeAuthority` (the holder could freeze
///   a vault and brick swaps/withdrawals for every LP).
/// - Token-2022 mints with extensions outside `ALLOWED_MINT_EXTENSIONS` →
///   `UnsupportedMintExtension`. Legacy SPL Token mints have no extensions.
///
/// @param mint Mint account (already owner-checked by `InterfaceAccount<Mint>`).
/// @return Ok if the mint can safely back a pool vault.
pub fn ensure_supported_mint(mint: &InterfaceAccount<Mint>) -> Result<()> {
    require!(
        mint.freeze_authority.is_none(),
        AmmError::MintHasFreezeAuthority
    );

    let info = mint.to_account_info();
    if *info.owner != spl_token_2022::ID {
        return Ok(());
    }

    let data = info.try_borrow_data()?;
    let state = StateWithExtensions::<spl_token_2022::state::Mint>::unpack(&data)
        .map_err(|_| error!(AmmError::UnsupportedMintExtension))?;
    let extensions = state
        .get_extension_types()
        .map_err(|_| error!(AmmError::UnsupportedMintExtension))?;

    require!(
        extensions
            .iter()
            .all(|ext| ALLOWED_MINT_EXTENSIONS.contains(ext)),
        AmmError::UnsupportedMintExtension
    );
    Ok(())
}
