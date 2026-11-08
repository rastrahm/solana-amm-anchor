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
# On-chain suite (local validator + 43 amm + security + event tests)
anchor test

# Math / host unit tests only (22 tests: 16 unit + 6 proptest properties)
cargo test -p amm

# Frontend (14 Vitest tests)
npm run app:test
```

Mutation testing (checks that the tests catch deliberately injected bugs):

```bash
# Math: cargo-mutants (cargo install --locked cargo-mutants)
cargo mutants -p amm --file programs/amm/src/helpers/math.rs
# On-chain validations (require!/constraint) against anchor test; ~2 min per mutant
python3 scripts/mutation-onchain.py
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

Pool mints must be created **without** a freeze authority (`spl-token create-token` only sets one with `--enable-freeze`) and, for Token-2022, without balance-altering extensions.

Copy the generated IDL into the app if you change the interface:

```bash
cp target/idl/amm.json app/src/lib/idl/
cp target/types/amm.ts app/src/lib/idl/  # if applicable
```

## Program usage (instructions)

| Instruction | Key parameters | Notes |
|-------------|----------------|-------|
| `initialize` | `seed`, `fee` (bps), `authority?` | Creates the Config PDA, LP mint, and vault ATAs |
| `deposit` | `max_x`, `max_y`, `min_lp` | First deposit locks `MINIMUM_LIQUIDITY` (1000); later deposits only pull the pool ratio |
| `withdraw` | `lp_amount`, `min_x`, `min_y` | Floor division in favor of the pool |
| `swap` | `is_x`, `amount_in`, `min_amount_out` | Fee on input; invariant `(x+Δin_fee)·(y-Δout) ≥ k` |
| `set_locked` | `locked` | `config.authority` only; pauses deposit/swap, withdraw always open |
| `set_authority` | `new_authority?` | `config.authority` only; `None` renounces forever |

## Events

Every instruction emits an event with `emit_cpi!` (`event-cpi` feature): the program calls itself with the event as instruction data, signed by the `__event_authority` PDA. Unlike `emit!` (`Program data:` log lines), these events are not truncated by the log limit and nobody else can emit them on the program's behalf.

| Instruction | Event | Main fields |
|-------------|-------|-------------|
| `initialize` | `PoolInitialized` | `authority`, `config`, mints, `seed`, `fee` |
| `deposit` | `LiquidityDeposited` | `user`, pulled `amount_x/y`, `lp_minted`, `lp_locked`, post-operation reserves |
| `withdraw` | `LiquidityWithdrawn` | `user`, `lp_burned`, `amount_x/y`, post-operation reserves |
| `swap` | `SwapExecuted` | `user`, `is_x`, `amount_in`, `amount_out`, post-operation reserves |
| `set_locked` | `LockedUpdated` | `authority`, `locked` |
| `set_authority` | `AuthorityUpdated` | `previous_authority`, `new_authority` |

Each instruction gets two extra accounts (`event_authority` and `program`); with `accountsPartial` the Anchor client resolves them from the IDL. To read events, take the transaction's inner instructions addressed to the program, drop the 8-byte `EVENT_IX_TAG_LE` prefix and decode the rest with `program.coder.events.decode` (see `eventsOf` in `tests/security.ts`). Cost: ≈ 5–12k extra compute units per instruction.

Real example with raw logs, bytes and the decoded event for each instruction: [`doc/ejemplo-logs-eventos.md`](doc/ejemplo-logs-eventos.md) (regenerate it with `scripts/event-logs-example.ts`).

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
- Minimum liquidity locked on the first deposit (anti share-inflation); prior donations to the vault cannot brick the pool
- Mints without a `freeze_authority` (`MintHasFreezeAuthority`), so nobody can freeze a vault
- Unbalanced deposits: only the pool ratio is pulled; the excess stays in the wallet
- Optional admin: pauses deposit/swap without being able to block withdrawals; transferable or renounceable
- Events via PDA-signed self-CPI: they cannot be forged or truncated
- Token-2022: only mints without extensions or with metadata/group extensions. TransferFee, PermanentDelegate, TransferHook and other extensions that alter balances or custody are rejected (`UnsupportedMintExtension`)

Details and traceability to tests: [`doc/auditoria-EN.md`](doc/auditoria-EN.md).
