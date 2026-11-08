# Flujograma — solana-amm-anchor

> 🇬🇧 [English version](flujograma-EN.md) · [Índice](../README.md)

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
    G --> I{¿vault_x ≠ vault_y?}
    I -->|no| E4[/AmmError::IdenticalVaults/]
    I -->|sí| FZ{¿Mints sin freeze_authority?}
    FZ -->|no| E6[/AmmError::MintHasFreezeAuthority/]
    FZ -->|sí| X{¿Extensiones Token-2022 del mint en allowlist?}
    X -->|no| E5[/AmmError::UnsupportedMintExtension/]
    X -->|sí| H[Crear vault ATAs X/Y authority = Config]
    H --> J[Guardar bumps canónicos + authority + fee]
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
    C -->|sí| PL{¿Pool pausado config.locked?}
    PL -->|sí| E6[/PoolLocked/]
    PL -->|no| D{¿max_x y max_y > 0?}
    D -->|no| E3[/InvalidAmount/]
    D -->|sí| E{¿Supply LP = 0?<br/>donaciones al vault se ignoran}
    E -->|sí primer depósito| F[LP = sqrt max_x · max_y<br/>se cobran max_x, max_y]
    F --> G{¿LP > MINIMUM_LIQUIDITY?}
    G -->|no| E4[/InsufficientLiquidity/]
    G -->|sí| H[Acuñar LP; lock MINIMUM_LIQUIDITY]
    E -->|no| I[LP = min proporciones floor<br/>cobra ceil LP · reserva / supply<br/>excedente queda al usuario]
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

> `withdraw` no consulta `config.locked`: una pausa nunca atrapa los fondos de los LP.

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
    D -->|sí| PL{¿Pool pausado config.locked?}
    PL -->|sí| E7[/PoolLocked/]
    PL -->|no| E{¿amount_in > 0?}
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

## 5. set_locked / set_authority (admin)

```mermaid
flowchart TD
    A([Inicio set_locked / set_authority]) --> B{¿authority es Signer?}
    B -->|no| E1[/Missing signer/]
    B -->|sí| C{¿Seeds + bump de Config OK?}
    C -->|no| E2[/ConstraintSeeds/]
    C -->|sí| D{¿config.authority == Some signer?}
    D -->|no / pool sin admin| E3[/AmmError::Unauthorized/]
    D -->|sí| F[set_locked: guarda locked<br/>set_authority: guarda nuevo admin o None irreversible]
    F --> G([Ok])
```

---

## 6. Camino de seguridad transversal (toda instrucción que muta estado)

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
| Depósito desbalanceado | Solo se cobra la proporción del pool (`deposit_quote`) |
| PoolLocked | Pausa del admin en deposit/swap; withdraw siempre abierto |
| Unauthorized | `config.authority == Some(signer)` en instrucciones admin |
| MintHasFreezeAuthority / UnsupportedMintExtension | Política de mints en `initialize` |
| Overflow | `checked_*` + `u128` |
