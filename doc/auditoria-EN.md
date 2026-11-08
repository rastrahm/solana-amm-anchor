# Audit checklist — solana-amm-anchor

> 🇪🇸 [Versión en español](auditoria-ES.md) · [Index](../README.md)

Final matrix (Phase 8) mapping Sealevel / Solana security course mitigations ↔ implementation ↔ tests.

**Program:** `programs/amm` · **Suite:** `tests/amm.ts` + `tests/security.ts` (37 passing) · **Lint:** `cargo fmt` + `cargo clippy -D warnings`

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
| First-deposit inflation | `MINIMUM_LIQUIDITY = 1000` lock | `deposit_quote` | security: first-deposit |
| Donation griefing (bricked pool) | First deposit ignores pre-existing vault balances; the donation is absorbed by the pool | `deposit_quote` | unit + security: donation griefing |
| Dangerous Token-2022 extensions | Mint extension allowlist (metadata/group only); rejects TransferFee, PermanentDelegate, TransferHook, etc. → `UnsupportedMintExtension` | `helpers/mint_policy.rs` + `initialize` | security: token-2022 (3 tests) |
| Slippage / MEV sandwich | `min_lp`, `min_x`/`min_y`, `min_amount_out` | deposit/withdraw/swap | amm: slippage rejects |
| Exact rent / space | `space = Config::ACCOUNT_SPACE` (150) | `Config::INIT_SPACE` | security: rent/space |
| Paused pool | `set_locked` (`config.authority` only) → `PoolLocked` on deposit/swap; withdraw is never blocked | `instructions/update_config.rs` | security: admin pause |
| Unauthorized admin | `constraint = config.authority == Some(authority)` → `Unauthorized`; a pool without admin is immutable | `UpdateConfig` | security: admin unauthorized / transfer+renounce |
| Unbalanced deposit | `deposit_quote` pulls `ceil(lp · reserve / supply)` of each token; the excess is not taken | `helpers/math.rs` | unit + security: unbalanced deposit |
| Freezable mint | `freeze_authority` must be `None` → `MintHasFreezeAuthority` | `helpers/mint_policy.rs` | security: freeze authority |
| Invalid fee | `fee <= MAX_FEE_BPS` | initialize | amm: invalid fee |
| Unsafe `transfer` | Only `transfer_checked` + decimals | CPI helpers | static review (grep) |
| False positives in negative tests | `errorBlob` rethrows `AssertionError`: an `expect.fail` inside `try` is no longer swallowed by the `catch` | `tests/*.ts` | whole suite |

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
| `initialize` | High (one-shot) | Several `init` + ATAs + mint extension TLV parsing; not on the trading hot path |
| `deposit` / `withdraw` | Medium | 2–3 token CPIs + proportional math |
| `swap` | Hot | Fee + 1 division + 2 `transfer_checked`; no loops |
| `set_locked` / `set_authority` | Very low | Constraint + `Config` write only; no CPI |

Operational recommendation: measure with `getTransaction` / validator logs after `anchor test` or a local deploy if a strict CU budget is required. There is no O(n) work in the program.

---

## 4. Mathematical invariants

- Pool: \(k = x \cdot y\)
- Post-swap: \((x + \Delta x_{\text{fee}}) \cdot (y - \Delta y) \ge k\) (fee on input)
- Subsequent deposit: \(lp = \min(\lfloor x_{max} L / X \rfloor, \lfloor y_{max} L / Y \rfloor)\); pulls \(\lceil lp \cdot X / L \rceil \le x_{max}\) and \(\lceil lp \cdot Y / L \rceil \le y_{max}\)
- First-deposit LP: \(\lfloor\sqrt{x\cdot y}\rfloor - \texttt{MINIMUM\_LIQUIDITY}\)
- Withdraw: floor in favor of the pool; supply cannot drop below the minimum lock

---

## 5. Phase 8 closeout

- [x] `cargo fmt -p amm -- --check`
- [x] `cargo clippy -p amm --lib --tests -- -D warnings`
- [x] Root README + diagrams aligned
- [x] This audit matrix
- [x] `anchor test` green (37 passing)
- [x] Human sign-off on closeout (2026-09-22)

**No pending security TODOs in the program tree.**

---

## 6. Post-Phase 8 hardening

Fixed with TDD (red test reproducing the attack → fix → green):

- **Donation griefing:** previously, sending 1 unit straight to the predictable vault ATA before the first deposit left `total_lp == 0` with non-zero reserves, bricking the pool forever (`InsufficientLiquidity`). The first deposit now computes LP from the supplied amounts only.
- **Token-2022:** mints with extensions that alter balances or custody (TransferFee, PermanentDelegate, TransferHook, DefaultAccountState, NonTransferable, ConfidentialTransfer…) are rejected in `initialize`. Classic SPL Token mints and Token-2022 mints without extensions (or metadata/group only) remain supported.
- **Test suite:** negative tests could pass falsely because the `expect.fail("expected X")` message matched the `catch` regex.

Second round (also TDD):

- **Unbalanced deposits:** `deposit(max_x, max_y, min_lp)` computes LP from the scarce side and pulls only `ceil(lp · reserve / supply)` of each token (rounding in favor of the pool). The excess stays in the user's wallet.
- **Admin:** new `set_locked` and `set_authority` instructions, restricted to `config.authority`. The pause affects deposit/swap; `withdraw` no longer checks `locked`, so an admin cannot trap funds. `set_authority(None)` renounces irreversibly.
- **Freeze authority:** `initialize` rejects mints with a `freeze_authority` (`MintHasFreezeAuthority`). Consequence: stablecoins with a freeze authority (e.g. USDC) cannot be listed.
