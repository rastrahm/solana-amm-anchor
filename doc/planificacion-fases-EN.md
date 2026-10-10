# Phased planning — solana-amm-anchor

> 🇪🇸 [Versión en español](planificacion-fases-ES.md) · [Index](../README.md)

Constant-product AMM (`x · y = k`) on Solana with Anchor 0.30+, Rust BPF/SBF, and `token_interface` (SPL Token / Token-2022).

## Governance rules

1. **Each phase requires your explicit authorization** before implementation starts.
2. Work does not move to phase `N+1` until phase `N` is marked **COMPLETED** and you authorize the next one.
3. **TDD** applies within each phase: tests first (TypeScript `anchor test` or `program-test`), then implementation.
4. Rust code: no `.unwrap()`/`.expect()` in production; arithmetic with `checked_*` and `u128`; `///` docs on public instructions.
5. Frontend (if applicable): Next.js App Router, Zod, Vitest + RTL, no `any`.

### How to authorize

Reply in the chat with something like:

```text
I authorize Phase N — <name>
```

---

## Global status

| Phase | Name | Status | Authorized | Completed |
|-------|------|--------|------------|-----------|
| 0 | Workspace scaffold | COMPLETED | YES | YES |
| 1 | `Config` state + errors + layout | COMPLETED | YES | YES |
| 2 | `initialize` instruction | COMPLETED | YES | YES |
| 3 | `deposit` instruction (liquidity) | COMPLETED | YES | YES |
| 4 | `withdraw` instruction | COMPLETED | YES | YES |
| 5 | `swap` instruction | COMPLETED | YES | YES |
| 6 | Sealevel security suite | COMPLETED | YES | YES |
| 7 | Next.js client / frontend | COMPLETED | YES | YES |
| 8 | Hardening, final docs, and audit checklist | COMPLETED | YES | YES |

---

## Phase 0 — Workspace scaffold

**Goal:** leave the repo buildable with Anchor, local keys ignored, and a modular structure.

**Status:** COMPLETED (authorized 2026-09-21)

**Deliverables:**
- `Anchor.toml`, `Cargo.toml` (workspace), program in `programs/amm/`
- `declare_id!` aligned with the local deploy keypair (`DR4UwHAVE9tVSm1kJo89ZiV6Dk1SXVPCPAhg67LT99mD`)
- Folders: `state/`, `instructions/`, `errors/`, `tests/`
- `anchor build` / `anchor test` scripts green (smoke)

**Acceptance criteria:**
- [x] `anchor build` OK
- [x] Module structure documented in `lib.rs`
- [x] No secrets or keypairs in git (`.gitignore` + keypair only in `target/deploy/`)

**Environment notes:**
- Node `>=20.18` (see `.nvmrc`)
- `[workspace.metadata.solana] tools-version = "v1.52"` to avoid `edition2024` failures with platform-tools v1.48

**Dependencies:** none  
**Authorization required:** yes

---

## Phase 1 — `Config` state + errors + byte layout

**Goal:** define the pool configuration account with a static layout and typed errors.

**Status:** COMPLETED (authorized 2026-09-21)

**Deliverables:**
- [x] `Config` with `#[account]` + `#[derive(InitSpace)]`
- [x] Fields ordered by descending size: `authority` → `mint_*` → `seed` → `fee` → `locked` / bumps
- [x] Exact space: `Config::ACCOUNT_SPACE = 8 + Config::INIT_SPACE` (= 150)
- [x] Canonical bumps: `config_bump`, `lp_bump`
- [x] `AmmError` (`#[error_code]`): IdenticalVaults, IdenticalMints, InvalidFee, SlippageExceeded, MathOverflow, InsufficientLiquidity, Unauthorized, PoolLocked, InvalidAmount

**Acceptance criteria:**
- [x] Rust test: `INIT_SPACE == 142` and `ACCOUNT_SPACE == 8 + INIT_SPACE`
- [x] TS test: documents the same size (150)
- [x] `///` docs for the struct and each error

**Dependencies:** Phase 0  
**Authorization required:** yes

---

## Phase 2 — `initialize` instruction

**Goal:** create the pool atomically (PDAs + vaults + LP mint).

**Status:** COMPLETED (authorized 2026-09-21)

**Deliverables:**
- [x] Accounts: `init` Config + LP mint; vault ATAs created in the handler after validations
- [x] Distinct vaults / mints (`require_keys_neq!` + canonical ATA address check)
- [x] LP mint with authority = Config PDA
- [x] Fee `<= MAX_FEE_BPS`
- [x] Optional authority (`Option<Pubkey>`)
- [x] Tests: happy init; double init; identical mints; invalid fee

**Acceptance criteria:**
- [x] Anchor discriminator on Config (anti type-cosplay)
- [x] `config_bump` / `lp_bump` persisted
- [x] `token_program: Interface<TokenInterface>` (anti arbitrary CPI)
- [x] Config space = 150 verified on-chain

**Dependencies:** Phase 1  
**Authorization required:** yes

---

## Phase 3 — `deposit` instruction (add liquidity)

**Goal:** deposit X/Y and mint LP; the first deposit locks minimum liquidity.

**Status:** COMPLETED (authorized 2026-09-21)

**Deliverables:**
- [x] LP math in `helpers/math.rs` (`u128` + `checked_*` + `integer_sqrt`)
- [x] `MINIMUM_LIQUIDITY = 1000` locked in the `lock_lp` ATA (authority = Config)
- [x] Proportional subsequent deposits `min(dx·L/x, dy·L/y)`
- [x] `min_lp` slippage
- [x] Tests: first deposit, subsequent, amount=0, slippage, minimum liquidity
- [x] `Box`ed accounts for the 4KB BPF stack limit
- [x] `init-if-needed` feature for LP ATAs

**Acceptance criteria:**
- [x] First-deposit anti-inflation (1000 LP lock)
- [x] `transfer_checked` + stored canonical bumps
- [x] `///` docs for the instruction

**Dependencies:** Phase 2  
**Authorization required:** yes

---

## Phase 4 — `withdraw` instruction

**Goal:** burn LP and withdraw X/Y proportionally.

**Status:** COMPLETED (authorized 2026-09-21)

**Deliverables:**
- [x] `amounts_for_withdraw` (floor `lp · reserve / total_lp`)
- [x] Burn LP + `transfer_checked` vault → user (Config signs with canonical bump)
- [x] `min_x` / `min_y` slippage
- [x] Post-burn supply `>= MINIMUM_LIQUIDITY`
- [x] Tests: happy path, slippage, insufficient LP, cannot burn the lock, withdraw all user LP

**Acceptance criteria:**
- [x] Rounding in favor of the pool
- [x] Signer + ownership via ATA constraints

**Dependencies:** Phase 3  
**Authorization required:** yes

---

## Phase 5 — `swap` instruction

**Goal:** constant-product swap with fee and post-swap invariant.

**Status:** COMPLETED (authorized 2026-09-21)

**Deliverables:**
- [x] X→Y / Y→X direction (`is_x`)
- [x] Fee on input (Uniswap v2 style, bps / 10_000)
- [x] Invariant `(x+Δx)·(y−Δy) ≥ x·y`
- [x] `min_amount_out` slippage (exact-in)
- [x] `u128` + `checked_*` math in `amount_out_for_swap`
- [x] Tests: X→Y, Y→X, fee vs 0, slippage, amount_in=0

**Acceptance criteria:**
- [x] `k` does not decrease after the swap
- [x] Only `transfer_checked`
- [x] Distinct vaults / user ATAs

**Dependencies:** Phase 4  
**Authorization required:** yes

---

## Phase 6 — Sealevel security suite

**Goal:** cover the Solana security course / module rule attack vectors.

**Status:** COMPLETED (authorized 2026-09-21)

**Deliverables (tests in `tests/security.ts`):**
- [x] Type cosplay — 150-byte account with wrong discriminator
- [x] Arbitrary CPI — `token_program = SystemProgram`
- [x] Duplicate accounts — `vault_x == vault_y`
- [x] Missing signer — impostor `user` without signature
- [x] Wrong authority — withdraw another wallet's LP
- [x] First-deposit inflation — 1-token bootstrap rejected; 1000 lock on the minimum viable deposit
- [x] Rent / space — on-chain `data.len == 150`
- [x] PDA bump / seeds — another pool's Config rejected

**Acceptance criteria:**
- [x] All 8 vectors fail as expected
- [x] Total suite: 28 passing (`amm` + `security`)

**Mitigations ↔ tests checklist**

| Mitigation | Test |
|------------|------|
| `#[account]` discriminator | type cosplay |
| `Interface<TokenInterface>` | arbitrary CPI |
| `require_keys_neq!` / ATA constraints | duplicate vaults |
| `Signer<'info>` | missing signer |
| `associated_token::authority` | wrong authority |
| `MINIMUM_LIQUIDITY` | first-deposit inflation |
| `Config::ACCOUNT_SPACE` | rent/space |
| `seeds` + `bump = config.config_bump` | PDA bump |

**Dependencies:** Phases 2–5  
**Authorization required:** yes

---

## Phase 7 — Next.js client / frontend (optional product phase)

**Goal:** minimal UI to interact with the AMM (wallet, pool, swap, LP).

**Status:** COMPLETED (authorized 2026-09-21)

**Deliverables:**
- [x] App in `app/` (Next.js 15 App Router)
- [x] Explicit `'use client'` / server layout
- [x] Zod (`schemas/pool.ts`) for initialize / deposit / withdraw / swap
- [x] Vitest + RTL (12 tests)
- [x] `error.tsx` / `not-found.tsx`
- [x] Wallet adapter (Phantom) + Init / Deposit / Swap / Withdraw panels
- [x] IDL copied to `app/src/lib/idl/`

**Acceptance criteria:**
- [x] Strict typing (no `any`)
- [x] JSDoc on public components/hooks
- [x] `npm run test` and `npm run build` OK in `app/`

**Quick start:** `cd app && cp .env.example .env.local && npm install && npm run dev`

**Dependencies:** Phases 0–6  
**Authorization required:** yes

---

## Phase 8 — Hardening, final docs, and audit checklist

**Goal:** production-ready quality closeout.

**Status:** COMPLETED AND APPROVED (authorized 2026-09-21; signed off 2026-09-22)

**Deliverables:**
- [x] Clean Clippy + rustfmt (`handlers` as `pub(crate)` to avoid ambiguous glob re-exports)
- [x] Qualitative compute unit review (`doc/auditoria-EN.md` §3)
- [x] Root README (build, test, local deploy)
- [x] Diagrams aligned with init (LP mint → vault ATAs)
- [x] Final mitigations vs tests matrix (`doc/auditoria-EN.md`)

**Acceptance criteria:**
- [x] `cargo fmt` / `cargo clippy -D warnings` OK
- [x] `anchor test` green (28 passing)
- [x] No open security TODOs in `programs/`
- [x] Closeout sign-off (localnet demo: init / deposit verified)

**Dependencies:** Phase 6 (and 7 if authorized)  
**Authorization required:** yes

---

## Suggested work order (summary)

```text
Phase 0 → 1 → 2 → 3 → 4 → 5 → 6 → [7 optional] → 8
          ↑
   authorize one at a time
```

## Rule sources applied

- `.cursorrules` (AMM, layout, Sealevel, math)
- `solana.cursorrules` (Anchor, PDAs, TDD, docs)
- `rust.cursorrules` (ownership, Result, clippy, TDD)
- `nextjs.cursorrules` (Phase 7 only)
