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
| 1 | Estado `Config` + errores + layout | PENDIENTE | NO | NO |
| 2 | Instrucción `initialize` | PENDIENTE | NO | NO |
| 3 | Instrucción `deposit` (liquidez) | PENDIENTE | NO | NO |
| 4 | Instrucción `withdraw` | PENDIENTE | NO | NO |
| 5 | Instrucción `swap` | PENDIENTE | NO | NO |
| 6 | Suite de seguridad Sealevel | PENDIENTE | NO | NO |
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

**Entregables:**
- `Config` con `#[account]` + `#[derive(InitSpace)]`
- Campos ordenados por tamaño descendente (regla BPF): `Option<Pubkey>` → `Pubkey` → `u64` → `u16` → `u8`
- Espacio exacto: `8 + Config::INIT_SPACE`
- Bumps canónicos almacenados (`config_bump`, `lp_bump`, etc.)
- `AmmError` (`#[error_code]`): vaults idénticos, fee inválido, slippage, overflow, pool pausado, etc.

**Criterios de aceptación:**
- Test de tamaño de cuenta = `8 + Config::INIT_SPACE`
- Documentación `///` del struct y de cada error relevante

**Dependencias:** Fase 0  
**Autorización requerida:** sí

---

## Fase 2 — Instrucción `initialize`

**Objetivo:** crear el pool de forma atómica (PDAs + vaults + mint LP).

**Entregables:**
- Accounts context con `init`, `seeds`, `bump`, `payer`, `space` explícitos
- Vaults X/Y distintos (`require_keys_neq!`)
- Mint LP con autoridad PDA
- Fee en basis points validado
- Authority opcional (`Option<Pubkey>`)
- Tests: init feliz; doble init falla; mints/vaults inválidos fallan

**Criterios de aceptación:**
- Discriminador Anchor presente (anti type-cosplay)
- Bumps guardados en `Config` (no recalcular desde el cliente en instrucciones posteriores)
- `transfer_checked` / interfaces TokenInterface donde aplique

**Dependencias:** Fase 1  
**Autorización requerida:** sí

---

## Fase 3 — Instrucción `deposit` (añadir liquidez)

**Objetivo:** depositar X/Y y acuñar LP; primer depósito bloquea liquidez mínima.

**Entregables:**
- Cálculo de LP con `u128` + `checked_*` (+ `checked_sqrt` en primer depósito)
- Lock de liquidez mínima (p. ej. 1000 LP) quemados o en PDA no gastable
- Depósitos posteriores proporcionales a reservas
- Parámetros de slippage (`min_lp` o equivalentes)
- Tests TDD: primer depósito, depósito subsecuente, amounts=0, overflow, slippage

**Criterios de aceptación:**
- No dilución por inflación de primer depósito
- Vaults y user ATAs distintos y con owner correcto
- Documentación `///` de la instrucción

**Dependencias:** Fase 2  
**Autorización requerida:** sí

---

## Fase 4 — Instrucción `withdraw`

**Objetivo:** quemar LP y retirar X/Y de forma proporcional.

**Entregables:**
- Burn de LP + `transfer_checked` desde vaults
- `min_x` / `min_y` (slippage)
- Prohibir retirar por debajo de la liquidez mínima bloqueada
- Tests: withdraw feliz, slippage, LP insuficiente, pool vacío parcial

**Criterios de aceptación:**
- Invariante de proporción respetada (redondeo a favor del pool)
- Signer y ownership verificados

**Dependencias:** Fase 3  
**Autorización requerida:** sí

---

## Fase 5 — Instrucción `swap`

**Objetivo:** intercambio constant-product con fee e invariante post-swap.

**Entregables:**
- Dirección X→Y / Y→X
- Fee sobre input: `(x + Δx_fee) · (y − Δy) ≥ k`
- `min_amount_out` / `max_amount_in`
- Math en `u128` con `checked_*`
- Tests: swap feliz, fee, sandwich/slippage, reservas insuficientes, dirección inválida

**Criterios de aceptación:**
- Invariante `k` no disminuye tras fee
- Solo `transfer_checked`
- Cuentas duplicadas rechazadas

**Dependencias:** Fase 4  
**Autorización requerida:** sí

---

## Fase 6 — Suite de seguridad Sealevel

**Objetivo:** cubrir vectores del curso de seguridad Solana / reglas del módulo.

**Entregables (tests obligatorios):**
- Type cosplay (cuenta mismo tamaño, discriminator incorrecto)
- Arbitrary CPI (sustituir `token_program`)
- Duplicate accounts (`vault_x == vault_y`)
- Missing signer / wrong authority
- First-deposit inflation (1 token / redondeo)
- Rent / space exacto `8 + Config::INIT_SPACE`
- PDA bump no canónico rechazado

**Criterios de aceptación:**
- Todos los tests de ataque fallan como se espera (assert failure)
- Checklist de mitigaciones marcada en este doc

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
