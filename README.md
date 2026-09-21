# solana-amm-anchor

AMM de producto constante (`x · y = k`) en Solana con **Anchor 0.31**, **SPL Token / Token-2022** (`token_interface`) y mitigaciones Sealevel documentadas en `.cursorrules`.

Program ID (localnet): `DR4UwHAVE9tVSm1kJo89ZiV6Dk1SXVPCPAhg67LT99mD`

## Requisitos

| Herramienta | Versión |
|-------------|---------|
| Rust | 1.75+ (host) |
| Solana CLI | 2.x con `tools-version = v1.52` (ver `Cargo.toml`) |
| Anchor | 0.31.x |
| Node | ≥ 20.18 (`engines` / `.nvmrc`) |

## Estructura

```text
programs/amm/     # Programa on-chain
tests/            # Integration + security (ts-mocha)
app/              # Cliente Next.js (Fase 7)
doc/              # Plan, diagramas, matriz de auditoría
```

## Build

```bash
anchor build
# o
cargo build-sbf -p amm
```

## Tests

```bash
# Suite on-chain (local validator + 28 tests amm + security)
anchor test

# Solo math / unit host
cargo test -p amm

# Frontend
npm run app:test
```

Lint Rust:

```bash
cargo fmt -p amm -- --check
cargo clippy -p amm --lib --tests -- -D warnings
```

## Deploy local

```bash
solana config set --url localhost
solana-keygen new -o ~/.config/solana/id.json   # si no tienes wallet
anchor localnet   # o: solana-test-validator &
anchor deploy
```

Copia el IDL generado a la app si cambias la interfaz:

```bash
cp target/idl/amm.json app/src/lib/idl/
cp target/types/amm.ts app/src/lib/idl/  # si aplica
```

## Uso del programa (instrucciones)

| Instrucción | Parámetros clave | Notas |
|-------------|------------------|--------|
| `initialize` | `seed`, `fee` (bps), `authority?` | Crea Config PDA, mint LP, vaults ATA |
| `deposit` | `amount_x`, `amount_y`, `min_lp` | Primer depósito bloquea `MINIMUM_LIQUIDITY` (1000) |
| `withdraw` | `lp_amount`, `min_x`, `min_y` | Floor division a favor del pool |
| `swap` | `is_x`, `amount_in`, `min_amount_out` | Fee sobre input; invariante `(x+Δin_fee)·(y-Δout) ≥ k` |

## Cliente Next.js

```bash
cd app
cp .env.example .env.local
npm install
npm run dev
```

Ver `app/README.md`.

## Documentación

- [Planificación por fases](doc/planificacion-fases.md)
- [Diagrama de clases](doc/diagrama-clases.md)
- [Diagrama de flujo](doc/diagrama-flujo.md)
- [Flujogramas por instrucción](doc/flujograma.md)
- [Checklist de auditoría](doc/auditoria.md)

## Seguridad (resumen)

- Cuentas tipadas con discriminator Anchor; bumps canónicos en `Config`
- Sin aliasing de vaults/mints; CPI solo vía `Interface<TokenInterface>`
- Aritmética `checked_*` en `u128`; slippage obligatorio en deposit/withdraw/swap
- Liquidez mínima bloqueada en el primer depósito (anti inflación de shares)

Detalle y trazabilidad a tests: [`doc/auditoria.md`](doc/auditoria.md).
