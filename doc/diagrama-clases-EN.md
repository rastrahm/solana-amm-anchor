# Class diagram — solana-amm-anchor

> 🇪🇸 [Versión en español](diagrama-clases-ES.md) · [Index](../README.md)

Static model of the AMM program (Anchor). It shows state accounts, instruction contexts, errors, and relationships with the SPL interfaces.

```mermaid
classDiagram
    direction TB

    class AmmProgram {
        <<program>>
        +initialize(seed, fee, authority) Result
        +deposit(amount_x, amount_y, min_lp) Result
        +withdraw(lp_amount, min_x, min_y) Result
        +swap(is_x, amount_in, min_out) Result
    }

    class Config {
        <<account>>
        +Option~Pubkey~ authority
        +Pubkey mint_x
        +Pubkey mint_y
        +Pubkey mint_lp
        +u64 seed
        +u16 fee
        +bool locked
        +u8 config_bump
        +u8 lp_bump
        --
        INIT_SPACE = 142
        ACCOUNT_SPACE = 8 + INIT_SPACE = 150
        layout: size-descending fields
    }

    class AmmError {
        <<error_code>>
        IdenticalVaults
        IdenticalMints
        InvalidFee
        SlippageExceeded
        MathOverflow
        InsufficientLiquidity
        Unauthorized
        PoolLocked
        InvalidAmount
    }

    class Initialize {
        <<Accounts>>
        +Signer initializer
        +InterfaceAccount mint_x
        +InterfaceAccount mint_y
        +Account Config config
        +InterfaceAccount mint_lp
        +InterfaceAccount vault_x
        +InterfaceAccount vault_y
        +Interface token_program
        +Program associated_token_program
        +Program system_program
    }

    class Deposit {
        <<Accounts>>
        +Signer user
        +Account Config config
        +InterfaceAccount vault_x
        +InterfaceAccount vault_y
        +InterfaceAccount user_x
        +InterfaceAccount user_y
        +InterfaceAccount user_lp
        +InterfaceAccount mint_lp
        +Interface token_program
    }

    class Withdraw {
        <<Accounts>>
        +Signer user
        +Account Config config
        +InterfaceAccount vault_x
        +InterfaceAccount vault_y
        +InterfaceAccount user_x
        +InterfaceAccount user_y
        +InterfaceAccount user_lp
        +InterfaceAccount mint_lp
        +Interface token_program
    }

    class Swap {
        <<Accounts>>
        +Signer user
        +Account Config config
        +InterfaceAccount vault_x
        +InterfaceAccount vault_y
        +InterfaceAccount user_x
        +InterfaceAccount user_y
        +Interface token_program
    }

    class VaultX {
        <<TokenAccount PDA>>
        authority = Config PDA
        mint = mint_x
    }

    class VaultY {
        <<TokenAccount PDA>>
        authority = Config PDA
        mint = mint_y
    }

    class MintLp {
        <<Mint PDA>>
        mint_authority = Config PDA
    }

    class TokenInterface {
        <<external>>
        Token / Token-2022
        transfer_checked()
        mint_to()
        burn()
    }

    AmmProgram --> Initialize : uses
    AmmProgram --> Deposit : uses
    AmmProgram --> Withdraw : uses
    AmmProgram --> Swap : uses
    AmmProgram --> AmmError : throws

    Initialize --> Config : init
    Initialize --> VaultX : init
    Initialize --> VaultY : init
    Initialize --> MintLp : init

    Deposit --> Config : reads bumps/fee
    Deposit --> VaultX : transfer in
    Deposit --> VaultY : transfer in
    Deposit --> MintLp : mint LP

    Withdraw --> Config : reads bumps
    Withdraw --> VaultX : transfer out
    Withdraw --> VaultY : transfer out
    Withdraw --> MintLp : burn LP

    Swap --> Config : reads fee/bumps
    Swap --> VaultX : in/out
    Swap --> VaultY : in/out

    Initialize ..> TokenInterface : CPI
    Deposit ..> TokenInterface : CPI
    Withdraw ..> TokenInterface : CPI
    Swap ..> TokenInterface : CPI

    note for Config "Fields ordered by size\nOption Pubkey → Pubkey → u64 → u16 → u8\nCanonical bump persisted"
```

## Design notes

| Element | Rule |
|---------|------|
| `Config` | `#[account]` + 8-byte discriminator; anti type-cosplay |
| Space | Exactly `8 + Config::INIT_SPACE` |
| Vaults | Distinct keys; authority = `Config` PDA |
| LP | First deposit locks minimum liquidity |
| Tokens | Only `token_interface` + `transfer_checked` |
| Math | Constant product; fee on input; `u128` + `checked_*` |
