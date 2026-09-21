# Checklist de auditoría — solana-amm-anchor

Matriz final (Fase 8) de mitigaciones Sealevel / curso de seguridad Solana ↔ implementación ↔ tests.

**Programa:** `programs/amm` · **Suite:** `tests/amm.ts` + `tests/security.ts` (28 passing) · **Lint:** `cargo fmt` + `cargo clippy -D warnings`

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
| First-deposit inflation | `MINIMUM_LIQUIDITY = 1000` lock | `lp_tokens_for_deposit` | security: first-deposit |
| Slippage / MEV sandwich | `min_lp`, `min_x`/`min_y`, `min_amount_out` | deposit/withdraw/swap | amm: slippage rejects |
| Rent / space exacto | `space = Config::ACCOUNT_SPACE` (150) | `Config::INIT_SPACE` | security: rent/space |
| Pool pausado | `config.locked` → `PoolLocked` | handlers | amm (si cubierto) / error path |
| Fee inválido | `fee <= MAX_FEE_BPS` | initialize | amm: invalid fee |
| `transfer` inseguro | Solo `transfer_checked` + decimals | CPI helpers | revisión estática (grep) |

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
| `initialize` | Alto (one-shot) | Varios `init` + ATAs; no está en hot path de trading |
| `deposit` / `withdraw` | Medio | 2–3 CPI token + math proporcional |
| `swap` | Caliente | Fee + 1 división + 2 `transfer_checked`; sin loops |

Recomendación operativa: medir con `getTransaction` / logs del validator tras `anchor test` o deploy local si se necesita presupuesto CU estricto. No hay work O(n) en el programa.

---

## 4. Invariantes matemáticas

- Pool: \(k = x \cdot y\)
- Post-swap: \((x + \Delta x_{\text{fee}}) \cdot (y - \Delta y) \ge k\) (fee sobre input)
- LP primer depósito: \(\lfloor\sqrt{x\cdot y}\rfloor - \texttt{MINIMUM\_LIQUIDITY}\)
- Withdraw: floor a favor del pool; supply no puede caer por debajo del lock mínimo

---

## 5. Cierre Fase 8

- [x] `cargo fmt -p amm -- --check`
- [x] `cargo clippy -p amm --lib --tests -- -D warnings`
- [x] README raíz + diagramas alineados
- [x] Esta matriz de auditoría
- [x] `anchor test` verde (28 passing)
- [ ] Visto bueno humano de cierre

**Sin TODOs de seguridad pendientes en el árbol del programa.**
