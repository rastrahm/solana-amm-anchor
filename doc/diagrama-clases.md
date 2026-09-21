# Diagrama de clases — solana-amm-anchor

Modelo estático del programa AMM (Anchor). Representa cuentas de estado, contextos de instrucción, errores y relaciones con interfaces SPL.

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
        +Pubkey seed_related / mint_x
        +Pubkey mint_y
        +Pubkey mint_lp
        +u16 fee_bps
        +u8 config_bump
        +u8 lp_bump
        +u8 ...bumps
        --
        space = 8 + Config::INIT_SPACE
        layout: size-descending fields
    }

    class AmmError {
        <<error_code>>
        IdenticalVaults
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

    AmmProgram --> Initialize : usa
    AmmProgram --> Deposit : usa
    AmmProgram --> Withdraw : usa
    AmmProgram --> Swap : usa
    AmmProgram --> AmmError : lanza

    Initialize --> Config : init
    Initialize --> VaultX : init
    Initialize --> VaultY : init
    Initialize --> MintLp : init

    Deposit --> Config : lee bumps/fee
    Deposit --> VaultX : transfer in
    Deposit --> VaultY : transfer in
    Deposit --> MintLp : mint LP

    Withdraw --> Config : lee bumps
    Withdraw --> VaultX : transfer out
    Withdraw --> VaultY : transfer out
    Withdraw --> MintLp : burn LP

    Swap --> Config : lee fee/bumps
    Swap --> VaultX : in/out
    Swap --> VaultY : in/out

    Initialize ..> TokenInterface : CPI
    Deposit ..> TokenInterface : CPI
    Withdraw ..> TokenInterface : CPI
    Swap ..> TokenInterface : CPI

    note for Config "Orden de campos por tamaño\nOption Pubkey → Pubkey → u64 → u16 → u8\nBump canónico persistido"
```

## Notas de diseño

| Elemento | Regla |
|----------|--------|
| `Config` | `#[account]` + discriminator 8 bytes; anti type-cosplay |
| Espacio | Exactamente `8 + Config::INIT_SPACE` |
| Vaults | Claves distintas; authority = PDA `Config` |
| LP | Primer depósito bloquea liquidez mínima |
| Tokens | Solo `token_interface` + `transfer_checked` |
| Math | Constant product; fee sobre input; `u128` + `checked_*` |
