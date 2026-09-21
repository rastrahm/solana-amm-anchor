# Diagrama de flujo — solana-amm-anchor

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
        I2 --> I3[Crear Vault X / Vault Y]
        I3 --> I4[Crear Mint LP]
        I4 --> I5[Persistir fee, authority, bumps]
    end

    subgraph Dep["deposit"]
        D1[User transfiere X e Y a vaults] --> D2[Calcular LP con u128]
        D2 --> D3{¿Primer depósito?}
        D3 -->|sí| D4[Acuñar LP − lock mínimo]
        D3 -->|no| D5[Acuñar LP proporcional]
        D4 --> D6[Enviar LP a user o burn lock]
        D5 --> D6
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

    Init --> Dep
    Dep --> Wdr
    Dep --> Swp
```

## Límites de confianza

| Capa | Confía en | No confía en |
|------|-----------|--------------|
| Cliente | UX, serialización de args | Validación de seguridad |
| Programa | Constraints Anchor, PDA bumps guardados | Cuentas sin `owner`/discriminator |
| Token program | Interfaces validadas | Program IDs arbitrarios (anti arbitrary CPI) |
