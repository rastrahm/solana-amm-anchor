# Diagrama de flujo — solana-amm-anchor

> 🇬🇧 [English version](diagrama-flujo-EN.md) · [Índice](../README.md)

Flujo de datos y componentes entre usuario, cliente, runtime Solana y cuentas del AMM.

```mermaid
flowchart LR
    subgraph Cliente
        U[Usuario / Wallet]
        FE[App Next.js opcional]
        TS[Tests Anchor TS / CLI]
    end

    subgraph Solana["Runtime Sealevel"]
        TX[Transacción + Instruction data]
        RP[Programa AMM Anchor]
        CPI[CPI TokenInterface]
    end

    subgraph Cuentas["Cuentas on-chain"]
        CFG[(Config PDA)]
        VX[(Vault X)]
        VY[(Vault Y)]
        MLP[(Mint LP)]
        UATA[User ATAs X/Y/LP]
        MX[Mint X]
        MY[Mint Y]
    end

    U --> FE
    U --> TS
    FE --> TX
    TS --> TX
    TX --> RP

    RP -->|valida Accounts + bumps| CFG
    RP -->|lee reservas| VX
    RP -->|lee reservas| VY
    RP -->|mint/burn authority| MLP
    RP -->|lee balances usuario| UATA
    RP -->|decimals / mint check| MX
    RP -->|decimals / mint check| MY

    RP --> CPI
    CPI -->|transfer_checked / mint_to / burn| VX
    CPI --> VY
    CPI --> UATA
    CPI --> MLP

    CFG -.->|seeds + bump canónico| RP
```

## Flujo por operación (vista de datos)

```mermaid
flowchart TB
    subgraph Init["initialize"]
        I1[Initializer firma + paga rent] --> I2[Crear Config PDA]
        I2 --> I3[Crear Mint LP]
        I3 --> IP[Política de mints: sin freeze_authority, extensiones Token-2022 en allowlist]
        IP --> I4[Crear Vault ATAs X/Y tras checks]
        I4 --> I5[Persistir fee, authority, bumps]
    end

    subgraph Dep["deposit"]
        D0[User ofrece max_x, max_y] --> D2[deposit_quote con u128]
        D2 --> D3{¿Supply LP = 0?}
        D3 -->|sí| D4[Cobra max_x, max_y; LP = √xy − lock mínimo]
        D3 -->|no| D5[LP por el lado escaso; cobra ceil de la proporción]
        D4 --> D1[transfer_checked user → vaults]
        D5 --> D1
        D1 --> D6[mint_to LP al user y lock en el primer depósito]
    end

    subgraph Wdr["withdraw"]
        W1[User quema LP] --> W2[Calcular X/Y out]
        W2 --> W3[Comprobar min_x / min_y]
        W3 --> W4[transfer_checked vault → user]
    end

    subgraph Swp["swap"]
        S1[User envía amount_in] --> S2[Aplicar fee bps]
        S2 --> S3[Δout = f x·y=k]
        S3 --> S4[Comprobar min_out e invariante]
        S4 --> S5[transfer_checked in/out]
    end

    subgraph Adm["set_locked / set_authority"]
        A1[config.authority firma] --> A2{¿Some signer?}
        A2 -->|sí| A3[Pausa deposit/swap o transfiere/renuncia admin]
        A2 -->|no| A4[Unauthorized]
    end

    Init --> Dep
    Dep --> Wdr
    Dep --> Swp
    Init --> Adm
    Adm -.->|PoolLocked| Dep
    Adm -.->|PoolLocked| Swp
```

## Límites de confianza

| Capa | Confía en | No confía en |
|------|-----------|--------------|
| Cliente | UX, serialización de args | Validación de seguridad |
| Programa | Constraints Anchor, PDA bumps guardados | Cuentas sin `owner`/discriminator; saldos previos del vault en el primer depósito |
| Mints | SPL Token / Token-2022 sin freeze authority ni extensiones peligrosas | Freeze authority, transfer fee, delegado permanente, hooks |
| Admin | Solo puede pausar deposit/swap o ceder el rol | Nunca puede bloquear retiros ni mover fondos |
| Token program | Interfaces validadas | Program IDs arbitrarios (anti arbitrary CPI) |
