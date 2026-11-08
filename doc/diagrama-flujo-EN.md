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
        I3 --> IP[Mint policy: no freeze_authority, Token-2022 extensions in allowlist]
        IP --> I4[Create X/Y vault ATAs after checks]
        I4 --> I5[Persist fee, authority, bumps]
    end

    subgraph Dep["deposit"]
        D0[User offers max_x, max_y] --> D2[deposit_quote with u128]
        D2 --> D3{LP supply = 0?}
        D3 -->|yes| D4[Pull max_x, max_y; LP = √xy − minimum lock]
        D3 -->|no| D5[LP from the scarce side; pull ceil of the ratio]
        D4 --> D1[transfer_checked user → vaults]
        D5 --> D1
        D1 --> D6[mint_to LP to user and lock on first deposit]
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

    subgraph Adm["set_locked / set_authority"]
        A1[config.authority signs] --> A2{Some signer?}
        A2 -->|yes| A3[Pause deposit/swap or transfer/renounce admin]
        A2 -->|no| A4[Unauthorized]
    end

    Init --> Dep
    Dep --> Wdr
    Dep --> Swp
    Init --> Adm
    Adm -.->|PoolLocked| Dep
    Adm -.->|PoolLocked| Swp
```

## Trust boundaries

| Layer | Trusts | Does not trust |
|-------|--------|----------------|
| Client | UX, argument serialization | Security validation |
| Program | Anchor constraints, stored PDA bumps | Accounts without `owner`/discriminator; pre-existing vault balances on the first deposit |
| Mints | SPL Token / Token-2022 without freeze authority or dangerous extensions | Freeze authority, transfer fee, permanent delegate, hooks |
| Admin | Can only pause deposit/swap or hand over the role | Can never block withdrawals or move funds |
| Token program | Validated interfaces | Arbitrary program IDs (anti arbitrary CPI) |
