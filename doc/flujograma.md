# Flujograma — solana-amm-anchor

Flujogramas de decisión por instrucción (control de errores, validaciones Sealevel y math).

---

## 1. initialize

```mermaid
flowchart TD
    A([Inicio initialize]) --> B{¿Initializer es Signer?}
    B -->|no| E1[/Error: Missing signer/]
    B -->|sí| C{¿mint_x ≠ mint_y?}
    C -->|no| E2[/AmmError::IdenticalMints/]
    C -->|sí| D{¿fee_bps en rango válido?}
    D -->|no| E3[/AmmError::InvalidFee/]
    D -->|sí| F[init Config PDA space = ACCOUNT_SPACE 150]
    F --> G[init Mint LP mint_authority = Config]
    G --> H[Crear vault ATAs X/Y authority = Config]
    H --> I{¿vault_x ≠ vault_y?}
    I -->|no| E4[/AmmError::IdenticalVaults/]
    I -->|sí| J[Guardar bumps canónicos + authority + fee]
    J --> K([Ok])
```


---

## 2. deposit

```mermaid
flowchart TD
    A([Inicio deposit]) --> B{¿User Signer?}
    B -->|no| E1[/Missing signer/]
    B -->|sí| C{¿Cuentas owner + seeds/bump de Config OK?}
    C -->|no| E2[/Constraint / Unauthorized/]
    C -->|sí| D{¿amount_x y amount_y > 0?}
    D -->|no| E3[/InvalidAmount/]
    D -->|sí| E{¿Reservas actuales = 0?}
    E -->|sí primer depósito| F[LP = sqrt amount_x · amount_y]
    F --> G{¿LP > MINIMUM_LIQUIDITY?}
    G -->|no| E4[/InsufficientLiquidity/]
    G -->|sí| H[Acuñar LP; lock MINIMUM_LIQUIDITY]
    E -->|no| I[LP = min proporciones con u128 checked]
    I --> J{¿LP >= min_lp slippage?}
    H --> J
    J -->|no| E5[/SlippageExceeded/]
    J -->|sí| K[transfer_checked user → vaults]
    K --> L[mint_to LP al usuario]
    L --> M([Ok])
```

---

## 3. withdraw

```mermaid
flowchart TD
    A([Inicio withdraw]) --> B{¿User Signer?}
    B -->|no| E1[/Missing signer/]
    B -->|sí| C{¿Constraints PDA / vaults / mint_lp OK?}
    C -->|no| E2[/Constraint error/]
    C -->|sí| D{¿lp_amount > 0 y balance suficiente?}
    D -->|no| E3[/InsufficientLiquidity/]
    D -->|sí| E[Calcular out_x, out_y proporcionales u128]
    E --> F{¿out_x >= min_x y out_y >= min_y?}
    F -->|no| E4[/SlippageExceeded/]
    F -->|sí| G{¿Queda liquidez mínima bloqueada intacta?}
    G -->|no| E5[/InsufficientLiquidity/]
    G -->|sí| H[burn LP del usuario]
    H --> I[transfer_checked vaults → user]
    I --> J([Ok])
```

---

## 4. swap

```mermaid
flowchart TD
    A([Inicio swap]) --> B{¿User Signer?}
    B -->|no| E1[/Missing signer/]
    B -->|sí| C{¿token_program es Token o Token-2022?}
    C -->|no| E2[/Arbitrary CPI rechazado/]
    C -->|sí| D{¿vault_x ≠ vault_y y ATAs coherentes?}
    D -->|no| E3[/IdenticalVaults / invalid accounts/]
    D -->|sí| E{¿amount_in > 0?}
    E -->|no| E4[/InvalidAmount/]
    E -->|sí| F[amount_in_menos_fee = apply fee_bps]
    F --> G[Δout = reserva_out · ain_fee / reserva_in + ain_fee]
    G --> H{¿Δout >= min_amount_out?}
    H -->|no| E5[/SlippageExceeded/]
    H -->|sí| I{¿invariante k post-swap OK?}
    I -->|no| E6[/MathOverflow / invariant/]
    I -->|sí| J[transfer_checked in → vault]
    J --> K[transfer_checked vault → out]
    K --> L([Ok])
```

---

## 5. Camino de seguridad transversal (toda instrucción que muta estado)

```mermaid
flowchart TD
    A([Entrada de instrucción]) --> B[Deserializar con discriminator Anchor]
    B --> C{¿Discriminator válido?}
    C -->|no| E1[/Type cosplay fail/]
    C -->|sí| D[Validar seeds + bump = Config.stored_bump]
    D --> E{¿Bump canónico?}
    E -->|no| E2[/Invalid bump/]
    E -->|sí| F[Validar signers + owners]
    F --> G[Validar no aliasing de cuentas]
    G --> H[Math checked u128]
    H --> I[CPI solo a Interface Token]
    I --> J([Continuar lógica de negocio])
```

---

## Leyenda de errores frecuentes

| Código / condición | Mitigación |
|--------------------|------------|
| Missing signer | `Signer<'info>` |
| Type cosplay | `#[account]` + discriminator |
| IdenticalVaults | `require_keys_neq!` |
| Arbitrary CPI | `Interface<'info, TokenInterface>` |
| SlippageExceeded | `min_out` / `min_lp` / `min_x|y` |
| First deposit inflation | `MINIMUM_LIQUIDITY` lock |
| Overflow | `checked_*` + `u128` |
