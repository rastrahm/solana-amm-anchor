# Audit checklist — solana-amm-anchor

> 🇪🇸 [Versión en español](auditoria-ES.md) · [Index](../README.md)

Final matrix (Phase 8) mapping Sealevel / Solana security course mitigations ↔ implementation ↔ tests.

**Program:** `programs/amm` · **Suite:** `tests/amm.ts` + `tests/security.ts` (43 passing) · **Lint:** `cargo fmt` + `cargo clippy -D warnings`

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
| Forged or truncated events | `emit_cpi!`: self-CPI signed by the `__event_authority` PDA; an external call without that signature → `ConstraintSigner` | `events.rs` + `#[event_cpi]` | security: events (4 tests) |

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

Measured on localnet with `getTransaction` (`consumed N of 200000` logs), same test flows before and after adding events (Phase 10):

| Path | Without events | With events | Notes |
|------|----------------|-------------|-------|
| `initialize` | 74–84k | 90–99k | One-shot: several `init` + ATAs + mint TLV parsing; not on the hot path |
| `deposit` (first) | ~134k | ~139k | Creates `user_lp` and `lock_lp` with `init_if_needed` |
| `deposit` (later) | ~56k | ~67k | 2 `transfer_checked` + `mint_to` + proportional math |
| `withdraw` | ~49k | ~61k | `burn` + 2 `transfer_checked` |
| `swap` | ~44k | ~49k | Fee + 1 division + 2 `transfer_checked`; no loops |
| `set_locked` / `set_authority` | ~3.6k | ~13k | Constraint + `Config` write + event |

The event self-CPI consumes ~3.6k inside the program; the rest of the overhead (≈ 5–12k total) is the `invoke` and deriving the `__event_authority` PDA. Run-to-run variation comes from the ATA bump search (it depends on the mint addresses). Everything stays well below the 200k limit. There is no O(n) work in the program.

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
- [x] `anchor test` green (43 passing)
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

---

## 7. Test quality (mutation testing)

A mutant is a copy of the program with a deliberately injected bug. If no test fails, the mutant "survives" and exposes a gap in the suite.

| Scope | Tool | Mutants | Caught | Survived |
|-------|------|---------|--------|----------|
| `helpers/math.rs` | `cargo mutants` + 22 tests (16 unit + 6 `proptest`) | 23 | 20 + 2 timeouts (1 unviable) | 0 |
| On-chain validations (`require!` / `constraint`) | `scripts/mutation-onchain.py` + `anchor test` | 23 | 13 (1 by timeout) | 10 equivalent |

Real gaps found and closed with new tests:

- `withdraw.rs` — `user_lp.amount >= lp_amount`: the existing test asked for more LP than the total supply, so the math stopped it. New test: a second LP tries to burn more than it holds, within what the pool can pay.
- `initialize.rs` — `vault_x != vault_y`: there was no test. New test with the same vault in both slots → `IdenticalVaults`.

Equivalent survivors (an earlier validation rejects the same case; kept as defense in depth):

| Validation | Why it is unreachable |
|------------|-----------------------|
| `vault_x != vault_y` in deposit / swap / withdraw | `associated_token::mint` forces each vault to be its mint's ATA; mints differ, so addresses cannot match |
| `user_x != user_y` in deposit / swap / withdraw | Same with the user ATAs |
| Canonical ATA `require_keys_eq!` in `initialize` | The ATA program already refuses to create an account at a non-canonical address |
| `amount_in > 0` (swap) and `lp_amount > 0` (withdraw) | `amount_out_for_swap` / `amounts_for_withdraw` repeat the check with the same error |

Also, `errorBlob` rethrows `AssertionError` and every negative test uses `expectError(action, /Pattern/)`, which removes the false positives of the `try { …; expect.fail() } catch` pattern.

---

## 8. Events (Phase 10)

Every instruction emits an event with `emit_cpi!`: the program invokes itself with the event data and the `__event_authority` PDA signs that invocation.

- **Cannot be forged:** the event dispatcher generated by Anchor requires `event_authority` to sign and to be the canonical PDA. Only the program itself can produce that signature; a third party calling directly gets `ConstraintSigner` (test: *events cannot be forged*).
- **Not truncated:** the data travels as inner-instruction data, not as log lines (~10 KB limit per transaction).
- **No new powers:** the added accounts (`event_authority`, `program`) are read-only and the self-CPI receives no other accounts, so it cannot move funds or write state.
- **Verified data:** the tests compare each event's amounts and post-operation reserves with the on-chain vault balances. Reserves are computed with `checked_add` / `checked_sub` → `MathOverflow`.
- **Red before green:** the 4 tests were run against the program without events and failed before implementing.

Limitation: events live in the RPC transaction history, not in on-chain state. Full history requires an archival RPC or a custom indexer.
