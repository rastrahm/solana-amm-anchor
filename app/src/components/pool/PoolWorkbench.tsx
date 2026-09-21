"use client";

import { useState } from "react";
import { InitializePanel } from "@/components/pool/InitializePanel";
import { DepositPanel } from "@/components/pool/DepositPanel";
import { WithdrawPanel } from "@/components/pool/WithdrawPanel";
import { SwapPanel } from "@/components/pool/SwapPanel";
import { AmountField } from "@/components/pool/FormBits";

/**
 * @description Composes the demo AMM workbench: shared mint context + operation panels.
 * @returns Client workbench for initialize / deposit / withdraw / swap.
 */
export function PoolWorkbench() {
  const [mintX, setMintX] = useState("");
  const [mintY, setMintY] = useState("");

  return (
    <div className="workbench">
      <section className="panel shared" aria-labelledby="mints-heading">
        <h2 id="mints-heading">Active mints</h2>
        <p className="hint">
          Paste SPL mint addresses used by your pool. Create them with the CLI on
          localnet before depositing or swapping.
        </p>
        <AmountField id="shared-mint-x" label="Mint X" value={mintX} onChange={setMintX} placeholder="Pubkey" />
        <AmountField id="shared-mint-y" label="Mint Y" value={mintY} onChange={setMintY} placeholder="Pubkey" />
      </section>

      <InitializePanel />
      <DepositPanel mintX={mintX} mintY={mintY} />
      <SwapPanel mintX={mintX} mintY={mintY} />
      <WithdrawPanel mintX={mintX} mintY={mintY} />
    </div>
  );
}
