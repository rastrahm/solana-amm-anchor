# Planificación por fases — solana-amm-anchor

AMM de producto constante (`x · y = k`) en Solana con Anchor 0.30+, Rust BPF/SBF y `token_interface` (SPL Token / Token-2022).

## Reglas de gobernanza

1. **Cada fase requiere autorización explícita tuya** antes de empezar a implementarla.
2. No se avanza a la fase `N+1` hasta que la fase `N` esté marcada como **COMPLETADA** y tú autorices la siguiente.
3. Dentro de cada fase se aplica **TDD**: tests primero (TypeScript `anchor test` o `program-test`), luego implementación.
4. Código Rust: sin `.unwrap()`/`.expect()` en producción; aritmética con `checked_*` y `u128`; docs `///` en instrucciones públicas.
5. Frontend (si aplica): Next.js App Router, Zod, Vitest + RTL, sin `any`.

### Cómo autorizar

Responde en el chat con algo como:

```text
Autorizo Fase N — <nombre>
```

---

## Estado global

| Fase | Nombre | Estado | Autorizada | Completada |
|------|--------|--------|------------|------------|
| 0 | Scaffold del workspace | COMPLETADA | SÍ | SÍ |
| 1 | Estado `Config` + errores + layout | COMPLETADA | SÍ | SÍ |
| 2 | Instrucción `initialize` | COMPLETADA | SÍ | SÍ |
| 3 | Instrucción `deposit` (liquidez) | COMPLETADA | SÍ | SÍ |
| 4 | Instrucción `withdraw` | COMPLETADA | SÍ | SÍ |
| 5 | Instrucción `swap` | COMPLETADA | SÍ | SÍ |
| 6 | Suite de seguridad Sealevel | COMPLETADA | SÍ | SÍ |
| 7 | Cliente / frontend Next.js | PENDIENTE | NO | NO |
| 8 | Hardening, docs finales y checklist de auditoría | PENDIENTE | NO | NO |

---

## Fase 0 — Scaffold del workspace

**Objetivo:** dejar el repo compilable con Anchor, keys locales ignoradas y estructura modular.

**Estado:** COMPLETADA (autorizada 2026-09-21)

**Entregables:**
- `Anchor.toml`, `Cargo.toml` (workspace), programa en `programs/amm/`
- `declare_id!` alineado con keypair de despliegue local (`DR4UwHAVE9tVSm1kJo89ZiV6Dk1SXVPCPAhg67LT99mD`)
- Carpetas: `state/`, `instructions/`, `errors/`, `tests/`
- Scripts `anchor build` / `anchor test` verdes (smoke)

**Criterios de aceptación:**
- [x] `anchor build` OK
- [x] Estructura de módulos documentada en `lib.rs`
- [x] Sin secretos ni keypairs en git (`.gitignore` + keypair solo en `target/deploy/`)

**Notas de entorno:**
- Node `>=20.18` (ver `.nvmrc`)
- `[workspace.metadata.solana] tools-version = "v1.52"` para evitar fallos de `edition2024` con platform-tools v1.48

**Dependencias:** ninguna  
**Autorización requerida:** sí

---

## Fase 1 — Estado `Config` + errores + layout de bytes

**Objetivo:** definir la cuenta de configuración del pool con layout estático y errores tipados.

**Estado:** COMPLETADA (autorizada 2026-09-21)

**Entregables:**
- [x] `Config` con `#[account]` + `#[derive(InitSpace)]`
- [x] Campos ordenados por tamaño descendente: `authority` → `mint_*` → `seed` → `fee` → `locked` / bumps
- [x] Espacio exacto: `Config::ACCOUNT_SPACE = 8 + Config::INIT_SPACE` (= 150)
- [x] Bumps canónicos: `config_bump`, `lp_bump`
- [x] `AmmError` (`#[error_code]`): IdenticalVaults, IdenticalMints, InvalidFee, SlippageExceeded, MathOverflow, InsufficientLiquidity, Unauthorized, PoolLocked, InvalidAmount

**Criterios de aceptación:**
- [x] Test Rust: `INIT_SPACE == 142` y `ACCOUNT_SPACE == 8 + INIT_SPACE`
- [x] Test TS: documenta el mismo tamaño (150)
- [x] Documentación `///` del struct y de cada error

**Dependencias:** Fase 0  
**Autorización requerida:** sí

---

## Fase 2 — Instrucción `initialize`

**Objetivo:** crear el pool de forma atómica (PDAs + vaults + mint LP).

**Estado:** COMPLETADA (autorizada 2026-09-21)

**Entregables:**
- [x] Accounts: `init` Config + mint LP; vaults ATA creados en handler tras validaciones
- [x] Vaults / mints distintos (`require_keys_neq!` + chequeo de dirección ATA canónica)
- [x] Mint LP con autoridad = Config PDA
- [x] Fee `<= MAX_FEE_BPS`
- [x] Authority opcional (`Option<Pubkey>`)
- [x] Tests: init feliz; doble init; mints idénticos; fee inválido

**Criterios de aceptación:**
- [x] Discriminador Anchor en Config (anti type-cosplay)
- [x] Bumps `config_bump` / `lp_bump` persistidos
- [x] `token_program: Interface<TokenInterface>` (anti arbitrary CPI)
- [x] Espacio Config = 150 verificado on-chain

**Dependencias:** Fase 1  
**Autorización requerida:** sí

---

## Fase 3 — Instrucción `deposit` (añadir liquidez)

**Objetivo:** depositar X/Y y acuñar LP; primer depósito bloquea liquidez mínima.

**Estado:** COMPLETADA (autorizada 2026-09-21)

**Entregables:**
- [x] Math LP en `helpers/math.rs` (`u128` + `checked_*` + `integer_sqrt`)
- [x] `MINIMUM_LIQUIDITY = 1000` bloqueado en ATA `lock_lp` (authority = Config)
- [x] Depósitos posteriores proporcionales `min(dx·L/x, dy·L/y)`
- [x] Slippage `min_lp`
- [x] Tests: primer depósito, subsecuente, amount=0, slippage, liquidez mínima
- [x] Cuentas `Box`adas para límite de stack BPF 4KB
- [x] Feature `init-if-needed` para ATAs LP

**Criterios de aceptación:**
- [x] Anti inflación primer depósito (lock 1000 LP)
- [x] `transfer_checked` + bumps canónicos almacenados
- [x] Documentación `///` de la instrucción

**Dependencias:** Fase 2  
**Autorización requerida:** sí

---

## Fase 4 — Instrucción `withdraw`

**Objetivo:** quemar LP y retirar X/Y de forma proporcional.

**Estado:** COMPLETADA (autorizada 2026-09-21)

**Entregables:**
- [x] `amounts_for_withdraw` (floor `lp · reserve / total_lp`)
- [x] Burn LP + `transfer_checked` vault → user (Config firma con bump canónico)
- [x] Slippage `min_x` / `min_y`
- [x] Supply post-burn `>= MINIMUM_LIQUIDITY`
- [x] Tests: feliz, slippage, LP insuficiente, no quemar el lock, retirar todo el LP del user

**Criterios de aceptación:**
- [x] Redondeo a favor del pool
- [x] Signer + ownership vía constraints ATA

**Dependencias:** Fase 3  
**Autorización requerida:** sí

---

## Fase 5 — Instrucción `swap`

**Objetivo:** intercambio constant-product con fee e invariante post-swap.

**Estado:** COMPLETADA (autorizada 2026-09-21)

**Entregables:**
- [x] Dirección X→Y / Y→X (`is_x`)
- [x] Fee sobre input (estilo Uniswap v2, bps / 10_000)
- [x] Invariante `(x+Δx)·(y−Δy) ≥ x·y`
- [x] Slippage `min_amount_out` (exact-in)
- [x] Math `u128` + `checked_*` en `amount_out_for_swap`
- [x] Tests: X→Y, Y→X, fee vs 0, slippage, amount_in=0

**Criterios de aceptación:**
- [x] `k` no disminuye tras el swap
- [x] Solo `transfer_checked`
- [x] Vaults / user ATAs distintos

**Dependencias:** Fase 4  
**Autorización requerida:** sí

---

## Fase 6 — Suite de seguridad Sealevel

**Objetivo:** cubrir vectores del curso de seguridad Solana / reglas del módulo.

**Estado:** COMPLETADA (autorizada 2026-09-21)

**Entregables (tests en `tests/security.ts`):**
- [x] Type cosplay — cuenta tamaño 150, discriminator incorrecto
- [x] Arbitrary CPI — `token_program = SystemProgram`
- [x] Duplicate accounts — `vault_x == vault_y`
- [x] Missing signer — `user` impostor sin firma
- [x] Wrong authority — withdraw del LP de otra wallet
- [x] First-deposit inflation — bootstrap 1 token rechazado; lock 1000 en depósito mínimo viable
- [x] Rent / space — on-chain `data.len == 150`
- [x] PDA bump / seeds — Config de otro pool rechazado

**Criterios de aceptación:**
- [x] Los 8 vectores fallan como se espera
- [x] Suite total: 28 passing (`amm` + `security`)

**Checklist de mitigaciones ↔ tests**

| Mitigación | Test |
|------------|------|
| Discriminator `#[account]` | type cosplay |
| `Interface<TokenInterface>` | arbitrary CPI |
| `require_keys_neq!` / constraints ATA | duplicate vaults |
| `Signer<'info>` | missing signer |
| `associated_token::authority` | wrong authority |
| `MINIMUM_LIQUIDITY` | first-deposit inflation |
| `Config::ACCOUNT_SPACE` | rent/space |
| `seeds` + `bump = config.config_bump` | PDA bump |

**Dependencias:** Fases 2–5  
**Autorización requerida:** sí

---

## Fase 7 — Cliente / frontend Next.js (opcional de producto)

**Objetivo:** UI mínima para interactuar con el AMM (wallet, pool, swap, LP).

**Entregables:**
- App Router: `'use client'` / `'use server'` explícitos
- Zod en inputs de formularios y parámetros
- Vitest + RTL antes de componentes
- Componentes ≤ ~60 líneas cuando sea razonable
- `error.tsx` / `not-found.tsx` en rutas principales

**Criterios de aceptación:**
- Tipado estricto (cero `any`)
- JSDoc en componentes/actions/hooks públicos
- Flujo demo: connect → init/deposit/swap/withdraw en localnet/devnet

**Dependencias:** Fases 0–5 (idealmente también 6)  
**Autorización requerida:** sí

---

## Fase 8 — Hardening, docs finales y checklist de auditoría

**Objetivo:** cierre de calidad producción-ready.

**Entregables:**
- Clippy + rustfmt limpios
- Revisión de compute units en caminos calientes
- README de uso (build, test, deploy local)
- Actualizar diagramas si el diseño divergió
- Matriz final de mitigaciones vs tests

**Criterios de aceptación:**
- `anchor test` verde
- Sin TODOs de seguridad abiertos sin justificación
- Tú das el visto bueno de cierre

**Dependencias:** Fase 6 (y 7 si se autorizó)  
**Autorización requerida:** sí

---

## Orden de trabajo sugerido (resumen)

```text
Fase 0 → 1 → 2 → 3 → 4 → 5 → 6 → [7 opcional] → 8
         ↑
   autorizar una a una
```

## Fuentes de reglas aplicadas

- `.cursorrules` (AMM, layout, Sealevel, math)
- `solana.cursorrules` (Anchor, PDAs, TDD, docs)
- `rust.cursorrules` (ownership, Result, clippy, TDD)
- `nextjs.cursorrules` (solo Fase 7)
