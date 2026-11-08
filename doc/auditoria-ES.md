# Checklist de auditoría — solana-amm-anchor

> 🇬🇧 [English version](auditoria-EN.md) · [Índice](../README.md)

Matriz final (Fase 8) de mitigaciones Sealevel / curso de seguridad Solana ↔ implementación ↔ tests.

**Programa:** `programs/amm` · **Suite:** `tests/amm.ts` + `tests/security.ts` (37 passing) · **Lint:** `cargo fmt` + `cargo clippy -D warnings`

---

## 1. Matriz mitigaciones ↔ código ↔ tests

| Vector / riesgo | Mitigación en código | Evidencia | Test |
|-----------------|----------------------|-----------|------|
| Missing signer | `Signer<'info>` en initializer / user | `instructions/*` Accounts | security: missing signer |
| Owner / token program incorrecto | `InterfaceAccount` + `Interface<TokenInterface>` | constraints ATA / mint | security: arbitrary CPI |
| Type cosplay | `#[account]` + discriminator 8 B | `state/config.rs` | security: type cosplay |
| Re-init / unchecked init | `init` + PDA seeds únicos | `Initialize` | amm: initialize happy path |
| Arbitrary CPI | No `AccountInfo` libre como program; solo `TokenInterface` | Swap/Deposit/Withdraw | security: arbitrary CPI |
| PDA bump no canónico | Bumps guardados en `Config`; `bump = config.*_bump` | deposit/withdraw/swap | security: PDA bump |
| Duplicate / aliasing cuentas | `require_keys_neq!` vaults, mints, user ATAs | init + mutators | security: duplicate vaults |
| Wrong vault authority | `associated_token::authority = config` | Accounts constraints | security: wrong authority |
| Overflow / precision | `u128` + `checked_*` / `integer_sqrt` | `helpers/math.rs` | unit + amm edge cases |
| First-deposit inflation | `MINIMUM_LIQUIDITY = 1000` lock | `deposit_quote` | security: first-deposit |
| Donation griefing (pool bloqueado) | Primer depósito ignora saldo previo en vaults; la donación queda absorbida por el pool | `deposit_quote` | unit + security: donation griefing |
| Extensiones Token-2022 peligrosas | Allowlist de extensiones de mint (solo metadata/grupo); rechaza TransferFee, PermanentDelegate, TransferHook, etc. → `UnsupportedMintExtension` | `helpers/mint_policy.rs` + `initialize` | security: token-2022 (3 tests) |
| Slippage / MEV sandwich | `min_lp`, `min_x`/`min_y`, `min_amount_out` | deposit/withdraw/swap | amm: slippage rejects |
| Rent / space exacto | `space = Config::ACCOUNT_SPACE` (150) | `Config::INIT_SPACE` | security: rent/space |
| Pool pausado | `set_locked` (solo `config.authority`) → `PoolLocked` en deposit/swap; withdraw nunca se bloquea | `instructions/update_config.rs` | security: admin pause |
| Admin no autorizado | `constraint = config.authority == Some(authority)` → `Unauthorized`; pool sin admin es inmutable | `UpdateConfig` | security: admin unauthorized / transfer+renounce |
| Depósito desbalanceado | `deposit_quote` cobra `ceil(lp · reserva / supply)` de cada token; el excedente no se toma | `helpers/math.rs` | unit + security: unbalanced deposit |
| Mint congelable | `freeze_authority` debe ser `None` → `MintHasFreezeAuthority` | `helpers/mint_policy.rs` | security: freeze authority |
| Fee inválido | `fee <= MAX_FEE_BPS` | initialize | amm: invalid fee |
| `transfer` inseguro | Solo `transfer_checked` + decimals | CPI helpers | revisión estática (grep) |
| Falsos positivos en tests negativos | `errorBlob` relanza `AssertionError`: un `expect.fail` dentro del `try` ya no es tragado por el `catch` | `tests/*.ts` | toda la suite |

---

## 2. Layout y BPF

| Check | Valor esperado | Estado |
|-------|----------------|--------|
| Orden campos `Config` | size-descending | OK (`state/config.rs`) |
| `INIT_SPACE` | 142 | unit test |
| `ACCOUNT_SPACE` | 150 = 8 + 142 | unit + security |
| Stack BPF | `Box` en contextos pesados donde aplicó | Fase 3–5 |
| TODOs de seguridad abiertos | ninguno en `programs/` | grepped Fase 8 |

---

## 3. Compute units (caminos calientes)

Revisión cualitativa (sin microbenchmark en CI):

| Camino | Coste relativo | Notas |
|--------|----------------|-------|
| `initialize` | Alto (one-shot) | Varios `init` + ATAs + parseo TLV de extensiones del mint; no está en hot path de trading |
| `deposit` / `withdraw` | Medio | 2–3 CPI token + math proporcional |
| `swap` | Caliente | Fee + 1 división + 2 `transfer_checked`; sin loops |
| `set_locked` / `set_authority` | Muy bajo | Solo constraint + escritura en `Config`; sin CPI |

Recomendación operativa: medir con `getTransaction` / logs del validator tras `anchor test` o deploy local si se necesita presupuesto CU estricto. No hay work O(n) en el programa.

---

## 4. Invariantes matemáticas

- Pool: \(k = x \cdot y\)
- Post-swap: \((x + \Delta x_{\text{fee}}) \cdot (y - \Delta y) \ge k\) (fee sobre input)
- Depósito posterior: \(lp = \min(\lfloor x_{max} L / X \rfloor, \lfloor y_{max} L / Y \rfloor)\); se cobra \(\lceil lp \cdot X / L \rceil \le x_{max}\) y \(\lceil lp \cdot Y / L \rceil \le y_{max}\)
- LP primer depósito: \(\lfloor\sqrt{x\cdot y}\rfloor - \texttt{MINIMUM\_LIQUIDITY}\)
- Withdraw: floor a favor del pool; supply no puede caer por debajo del lock mínimo

---

## 5. Cierre Fase 8

- [x] `cargo fmt -p amm -- --check`
- [x] `cargo clippy -p amm --lib --tests -- -D warnings`
- [x] README raíz + diagramas alineados
- [x] Esta matriz de auditoría
- [x] `anchor test` verde (37 passing)
- [x] Visto bueno humano de cierre (2026-09-22)

**Sin TODOs de seguridad pendientes en el árbol del programa.**

---

## 6. Hardening post-Fase 8

Corregido con TDD (test en rojo que reproduce el ataque → fix → verde):

- **Donation griefing:** antes, transferir 1 unidad directa al ATA predecible del vault antes del primer depósito dejaba `total_lp == 0` con reservas ≠ 0 y el pool quedaba inutilizable para siempre (`InsufficientLiquidity`). Ahora el primer depósito calcula LP solo con los montos aportados.
- **Token-2022:** mints con extensiones que alteran saldos o custodia (TransferFee, PermanentDelegate, TransferHook, DefaultAccountState, NonTransferable, ConfidentialTransfer…) se rechazan en `initialize`. Mints SPL Token clásicos y Token-2022 sin extensiones (o solo metadata/grupo) siguen soportados.
- **Suite de tests:** los tests negativos podían pasar en falso porque el mensaje de `expect.fail("expected X")` coincidía con la regex del `catch`.

Segunda ronda (también con TDD):

- **Depósitos desbalanceados:** `deposit(max_x, max_y, min_lp)` calcula el LP por el lado escaso y cobra solo `ceil(lp · reserva / supply)` de cada token (redondeo a favor del pool). El excedente queda en la wallet del usuario.
- **Admin:** nuevas instrucciones `set_locked` y `set_authority`, solo para `config.authority`. La pausa afecta deposit/swap; `withdraw` ya no consulta `locked`, así un admin no puede atrapar fondos. `set_authority(None)` renuncia de forma irreversible.
- **Freeze authority:** `initialize` rechaza mints con `freeze_authority` (`MintHasFreezeAuthority`). Consecuencia: stablecoins con freeze authority (p. ej. USDC) no se pueden listar.
