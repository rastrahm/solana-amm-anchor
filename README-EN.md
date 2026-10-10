# solana-amm-anchor

> 🇪🇸 [Versión en español](README-ES.md) · [Index](README.md)

Constant-product AMM (`x · y = k`) on Solana built with **Anchor 0.31**, **SPL Token / Token-2022** (`token_interface`), and the Sealevel mitigations documented in `.cursorrules`.

Program ID (localnet): `DR4UwHAVE9tVSm1kJo89ZiV6Dk1SXVPCPAhg67LT99mD`

## Requirements

| Tool | Version |
|------|---------|
| Rust | 1.75+ (host) |
| Solana CLI | 2.x with `tools-version = v1.52` (see `Cargo.toml`) |
| Anchor | 0.31.x |
| Node | ≥ 20.18 (`engines` / `.nvmrc`) |

## Structure

```text
programs/amm/     # On-chain program
tests/            # Integration + security (ts-mocha)
app/              # Next.js client (Phase 7)
doc/              # Plan, diagrams, audit matrix
```

## Build

```bash
anchor build
# or
cargo build-sbf -p amm
```

## Tests

```bash
# On-chain suite (local validator + 28 amm + security tests)
anchor test

# Math / host unit tests only
cargo test -p amm

# Frontend
npm run app:test
```

Rust lint:

```bash
cargo fmt -p amm -- --check
cargo clippy -p amm --lib --tests -- -D warnings
```

## Local deploy

```bash
solana config set --url localhost
solana-keygen new -o ~/.config/solana/id.json   # if you don't have a wallet
anchor localnet   # or: solana-test-validator &
anchor deploy
```

Copy the generated IDL into the app if you change the interface:

```bash
cp target/idl/amm.json app/src/lib/idl/
cp target/types/amm.ts app/src/lib/idl/  # if applicable
```

## Program usage (instructions)

| Instruction | Key parameters | Notes |
|-------------|----------------|-------|
| `initialize` | `seed`, `fee` (bps), `authority?` | Creates the Config PDA, LP mint, and vault ATAs |
| `deposit` | `amount_x`, `amount_y`, `min_lp` | First deposit locks `MINIMUM_LIQUIDITY` (1000) |
| `withdraw` | `lp_amount`, `min_x`, `min_y` | Floor division in favor of the pool |
| `swap` | `is_x`, `amount_in`, `min_amount_out` | Fee on input; invariant `(x+Δin_fee)·(y-Δout) ≥ k` |

## Next.js client

```bash
cd app
cp .env.example .env.local
npm install
npm run dev
```

See `app/README.md`.

## Documentation

- [Phased planning](doc/planificacion-fases-EN.md)
- [Class diagram](doc/diagrama-clases-EN.md)
- [Flow diagram](doc/diagrama-flujo-EN.md)
- [Per-instruction flowcharts](doc/flujograma-EN.md)
- [Audit checklist](doc/auditoria-EN.md)

## Security (summary)

- Typed accounts with Anchor discriminators; canonical bumps stored in `Config`
- No vault/mint aliasing; CPI only through `Interface<TokenInterface>`
- `checked_*` arithmetic on `u128`; mandatory slippage bounds on deposit/withdraw/swap
- Minimum liquidity locked on the first deposit (anti share-inflation)

Details and traceability to tests: [`doc/auditoria-EN.md`](doc/auditoria-EN.md).
