# Next.js AMM demo client

## Setup

```bash
cd app
cp .env.example .env.local
npm install
npm run dev
```

Open http://localhost:3000

## Prerequisites

1. Local validator / Anchor workspace deployed (`anchor localnet` or `anchor test --skip-deploy` with a running validator and program deployed).
2. Browser wallet (Phantom) funded on the same cluster.
3. SPL mints created for the pool pair; paste their addresses into **Active mints**.

## Scripts

| Command | Purpose |
|---------|---------|
| `npm run dev` | Next.js dev server |
| `npm run build` | Production build |
| `npm run test` | Vitest (Zod schemas + UI) |

## Flow

Connect wallet → Initialize (seed, fee, mints) → Deposit → Swap / Withdraw.
