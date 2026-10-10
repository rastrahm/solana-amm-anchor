# Flowcharts — solana-amm-anchor

> 🇪🇸 [Versión en español](flujograma-ES.md) · [Index](../README.md)

Per-instruction decision flowcharts (error handling, Sealevel validations, and math).

---

## 1. initialize

```mermaid
flowchart TD
    A([Start initialize]) --> B{Is initializer a Signer?}
    B -->|no| E1[/Error: Missing signer/]
    B -->|yes| C{mint_x ≠ mint_y?}
    C -->|no| E2[/AmmError::IdenticalMints/]
    C -->|yes| D{fee_bps within valid range?}
    D -->|no| E3[/AmmError::InvalidFee/]
    D -->|yes| F[init Config PDA space = ACCOUNT_SPACE 150]
    F --> G[init LP Mint mint_authority = Config]
    G --> H[Create X/Y vault ATAs authority = Config]
    H --> I{vault_x ≠ vault_y?}
    I -->|no| E4[/AmmError::IdenticalVaults/]
    I -->|yes| J[Store canonical bumps + authority + fee]
    J --> K([Ok])
```


---

## 2. deposit

```mermaid
flowchart TD
    A([Start deposit]) --> B{User is Signer?}
    B -->|no| E1[/Missing signer/]
    B -->|yes| C{Account owners + Config seeds/bump OK?}
    C -->|no| E2[/Constraint / Unauthorized/]
    C -->|yes| D{amount_x and amount_y > 0?}
    D -->|no| E3[/InvalidAmount/]
    D -->|yes| E{Current reserves = 0?}
    E -->|yes, first deposit| F[LP = sqrt amount_x · amount_y]
    F --> G{LP > MINIMUM_LIQUIDITY?}
    G -->|no| E4[/InsufficientLiquidity/]
    G -->|yes| H[Mint LP; lock MINIMUM_LIQUIDITY]
    E -->|no| I[LP = min of ratios with checked u128]
    I --> J{LP >= min_lp slippage?}
    H --> J
    J -->|no| E5[/SlippageExceeded/]
    J -->|yes| K[transfer_checked user → vaults]
    K --> L[mint_to LP to user]
    L --> M([Ok])
```

---

## 3. withdraw

```mermaid
flowchart TD
    A([Start withdraw]) --> B{User is Signer?}
    B -->|no| E1[/Missing signer/]
    B -->|yes| C{PDA / vaults / mint_lp constraints OK?}
    C -->|no| E2[/Constraint error/]
    C -->|yes| D{lp_amount > 0 and sufficient balance?}
    D -->|no| E3[/InsufficientLiquidity/]
    D -->|yes| E[Compute proportional out_x, out_y u128]
    E --> F{out_x >= min_x and out_y >= min_y?}
    F -->|no| E4[/SlippageExceeded/]
    F -->|yes| G{Locked minimum liquidity still intact?}
    G -->|no| E5[/InsufficientLiquidity/]
    G -->|yes| H[burn user LP]
    H --> I[transfer_checked vaults → user]
    I --> J([Ok])
```

---

## 4. swap

```mermaid
flowchart TD
    A([Start swap]) --> B{User is Signer?}
    B -->|no| E1[/Missing signer/]
    B -->|yes| C{token_program is Token or Token-2022?}
    C -->|no| E2[/Arbitrary CPI rejected/]
    C -->|yes| D{vault_x ≠ vault_y and ATAs consistent?}
    D -->|no| E3[/IdenticalVaults / invalid accounts/]
    D -->|yes| E{amount_in > 0?}
    E -->|no| E4[/InvalidAmount/]
    E -->|yes| F[amount_in_after_fee = apply fee_bps]
    F --> G[Δout = reserve_out · ain_fee / reserve_in + ain_fee]
    G --> H{Δout >= min_amount_out?}
    H -->|no| E5[/SlippageExceeded/]
    H -->|yes| I{Post-swap k invariant OK?}
    I -->|no| E6[/MathOverflow / invariant/]
    I -->|yes| J[transfer_checked in → vault]
    J --> K[transfer_checked vault → out]
    K --> L([Ok])
```

---

## 5. Cross-cutting security path (every state-mutating instruction)

```mermaid
flowchart TD
    A([Instruction entry]) --> B[Deserialize with Anchor discriminator]
    B --> C{Valid discriminator?}
    C -->|no| E1[/Type cosplay fail/]
    C -->|yes| D[Validate seeds + bump = Config.stored_bump]
    D --> E{Canonical bump?}
    E -->|no| E2[/Invalid bump/]
    E -->|yes| F[Validate signers + owners]
    F --> G[Validate no account aliasing]
    G --> H[Checked u128 math]
    H --> I[CPI only to Token Interface]
    I --> J([Continue business logic])
```

---

## Common errors legend

| Code / condition | Mitigation |
|------------------|------------|
| Missing signer | `Signer<'info>` |
| Type cosplay | `#[account]` + discriminator |
| IdenticalVaults | `require_keys_neq!` |
| Arbitrary CPI | `Interface<'info, TokenInterface>` |
| SlippageExceeded | `min_out` / `min_lp` / `min_x|y` |
| First deposit inflation | `MINIMUM_LIQUIDITY` lock |
| Overflow | `checked_*` + `u128` |
