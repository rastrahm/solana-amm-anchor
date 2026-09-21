"use client";

import { PoolWorkbench } from "@/components/pool/PoolWorkbench";

/**
 * @description Home page for the AMM demo: hero copy plus interactive workbench.
 * @returns Landing composition for connect → pool operations.
 */
export default function HomePage() {
  return (
    <main>
      <section className="hero">
        <h1>Trade and provide liquidity on your local pool</h1>
        <p>
          Connect a wallet, initialize a constant-product pool, then deposit,
          swap, or withdraw. Point <code>NEXT_PUBLIC_SOLANA_RPC_URL</code> at
          localnet or devnet.
        </p>
      </section>
      <PoolWorkbench />
    </main>
  );
}
