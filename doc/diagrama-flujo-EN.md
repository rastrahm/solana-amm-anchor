# Flow diagram — solana-amm-anchor

> 🇪🇸 [Versión en español](diagrama-flujo-ES.md) · [Index](../README.md)

Data and component flow between the user, the client, the Solana runtime, and the AMM accounts.

```mermaid
flowchart LR
    subgraph Client
        U[User / Wallet]
        FE[Optional Next.js app]
        TS[Anchor TS tests / CLI]
    end

    subgraph Solana["Sealevel runtime"]
        TX[Transaction + Instruction data]
        RP[Anchor AMM program]
        CPI[TokenInterface CPI]
    end

    subgraph Accounts["On-chain accounts"]
        CFG[(Config PDA)]
        VX[(Vault X)]
        VY[(Vault Y)]
        MLP[(LP Mint)]
        UATA[User ATAs X/Y/LP]
        MX[Mint X]
        MY[Mint Y]
    end

    U --> FE
    U --> TS
    FE --> TX
    TS --> TX
    TX --> RP

    RP -->|validates Accounts + bumps| CFG
    RP -->|reads reserves| VX
    RP -->|reads reserves| VY
    RP -->|mint/burn authority| MLP
    RP -->|reads user balances| UATA
    RP -->|decimals / mint check| MX
    RP -->|decimals / mint check| MY

    RP --> CPI
    CPI -->|transfer_checked / mint_to / burn| VX
    CPI --> VY
    CPI --> UATA
    CPI --> MLP

    CFG -.->|seeds + canonical bump| RP
```

## Per-operation flow (data view)

```mermaid
flowchart TB
    subgraph Init["initialize"]
        I1[Initializer signs + pays rent] --> I2[Create Config PDA]
        I2 --> I3[Create LP Mint]
        I3 --> I4[Create X/Y vault ATAs after checks]
        I4 --> I5[Persist fee, authority, bumps]
    end

    subgraph Dep["deposit"]
        D1[User transfers X and Y to vaults] --> D2[Compute LP with u128]
        D2 --> D3{First deposit?}
        D3 -->|yes| D4[Mint LP − minimum lock]
        D3 -->|no| D5[Mint proportional LP]
        D4 --> D6[Send LP to user or burn lock]
        D5 --> D6
    end

    subgraph Wdr["withdraw"]
        W1[User burns LP] --> W2[Compute X/Y out]
        W2 --> W3[Check min_x / min_y]
        W3 --> W4[transfer_checked vault → user]
    end

    subgraph Swp["swap"]
        S1[User sends amount_in] --> S2[Apply fee bps]
        S2 --> S3[Δout = f x·y=k]
        S3 --> S4[Check min_out and invariant]
        S4 --> S5[transfer_checked in/out]
    end

    Init --> Dep
    Dep --> Wdr
    Dep --> Swp
```

## Trust boundaries

| Layer | Trusts | Does not trust |
|-------|--------|----------------|
| Client | UX, argument serialization | Security validation |
| Program | Anchor constraints, stored PDA bumps | Accounts without `owner`/discriminator |
| Token program | Validated interfaces | Arbitrary program IDs (anti arbitrary CPI) |
