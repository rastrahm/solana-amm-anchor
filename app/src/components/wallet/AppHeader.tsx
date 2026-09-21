"use client";

import { WalletMultiButton } from "@solana/wallet-adapter-react-ui";
import { getClusterName } from "@/lib/amm/cluster";

/**
 * @description Header bar with product brand, cluster label, and wallet connect button.
 * @returns Top navigation composition for the AMM demo.
 */
export function AppHeader() {
  const cluster = getClusterName();

  return (
    <header className="app-header">
      <div className="brand-block">
        <p className="brand">solana-amm</p>
        <p className="cluster" aria-label={`cluster ${cluster}`}>
          {cluster}
        </p>
      </div>
      <WalletMultiButton />
    </header>
  );
}
