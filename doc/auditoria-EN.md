# Audit checklist — solana-amm-anchor

> 🇪🇸 [Versión en español](auditoria-ES.md) · [Index](../README.md)

Final matrix (Phase 8) mapping Sealevel / Solana security course mitigations ↔ implementation ↔ tests.

**Program:** `programs/amm` · **Suite:** `tests/amm.ts` + `tests/security.ts` (28 passing) · **Lint:** `cargo fmt` + `cargo clippy -D warnings`

---

## 1. Mitigations ↔ code ↔ tests matrix

| Vector / risk | Mitigation in code | Evidence | Test |
|---------------|--------------------|----------|------|
| Missing signer | `Signer<'info>` on initializer / user | `instructions/*` Accounts | security: missing signer |
| Wrong owner / token program | `InterfaceAccount` + `Interface<TokenInterface>` | ATA / mint constraints | security: arbitrary CPI |
| Type cosplay | `#[account]` + 8-byte discriminator | `state/config.rs` | security: type cosplay |
| Re-init / unchecked init | `init` + unique PDA seeds | `Initialize` | amm: initialize happy path |
| Arbitrary CPI | No free `AccountInfo` as program; only `TokenInterface` | Swap/Deposit/Withdraw | security: arbitrary CPI |
| Non-canonical PDA bump | Bumps stored in `Config`; `bump = config.*_bump` | deposit/withdraw/swap | security: PDA bump |
| Duplicate / aliased accounts | `require_keys_neq!` on vaults, mints, user ATAs | init + mutators | security: duplicate vaults |
| Wrong vault authority | `associated_token::authority = config` | Accounts constraints | security: wrong authority |
| Overflow / precision | `u128` + `checked_*` / `integer_sqrt` | `helpers/math.rs` | unit + amm edge cases |
| First-deposit inflation | `MINIMUM_LIQUIDITY = 1000` lock | `lp_tokens_for_deposit` | security: first-deposit |
| Slippage / MEV sandwich | `min_lp`, `min_x`/`min_y`, `min_amount_out` | deposit/withdraw/swap | amm: slippage rejects |
| Exact rent / space | `space = Config::ACCOUNT_SPACE` (150) | `Config::INIT_SPACE` | security: rent/space |
| Paused pool | `config.locked` → `PoolLocked` | handlers | amm (if covered) / error path |
| Invalid fee | `fee <= MAX_FEE_BPS` | initialize | amm: invalid fee |
| Unsafe `transfer` | Only `transfer_checked` + decimals | CPI helpers | static review (grep) |

---

## 2. Layout and BPF

| Check | Expected value | Status |
|-------|----------------|--------|
| `Config` field order | size-descending | OK (`state/config.rs`) |
| `INIT_SPACE` | 142 | unit test |
| `ACCOUNT_SPACE` | 150 = 8 + 142 | unit + security |
| BPF stack | `Box` on heavy contexts where applicable | Phases 3–5 |
| Open security TODOs | none in `programs/` | grepped in Phase 8 |

---

## 3. Compute units (hot paths)

Qualitative review (no microbenchmark in CI):

| Path | Relative cost | Notes |
|------|---------------|-------|
| `initialize` | High (one-shot) | Several `init` + ATAs; not on the trading hot path |
| `deposit` / `withdraw` | Medium | 2–3 token CPIs + proportional math |
| `swap` | Hot | Fee + 1 division + 2 `transfer_checked`; no loops |

Operational recommendation: measure with `getTransaction` / validator logs after `anchor test` or a local deploy if a strict CU budget is required. There is no O(n) work in the program.

---

## 4. Mathematical invariants

- Pool: \(k = x \cdot y\)
- Post-swap: \((x + \Delta x_{\text{fee}}) \cdot (y - \Delta y) \ge k\) (fee on input)
- First-deposit LP: \(\lfloor\sqrt{x\cdot y}\rfloor - \texttt{MINIMUM\_LIQUIDITY}\)
- Withdraw: floor in favor of the pool; supply cannot drop below the minimum lock

---

## 5. Phase 8 closeout

- [x] `cargo fmt -p amm -- --check`
- [x] `cargo clippy -p amm --lib --tests -- -D warnings`
- [x] Root README + diagrams aligned
- [x] This audit matrix
- [x] `anchor test` green (28 passing)
- [x] Human sign-off on closeout (2026-09-22)

**No pending security TODOs in the program tree.**
