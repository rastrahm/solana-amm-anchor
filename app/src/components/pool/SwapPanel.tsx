"use client";

import { useState } from "react";
import { PublicKey } from "@solana/web3.js";
import { TOKEN_PROGRAM_ID, getAssociatedTokenAddressSync } from "@solana/spl-token";
import { useWallet } from "@solana/wallet-adapter-react";
import { AmountField, StatusBanner, onFormSubmit } from "@/components/pool/FormBits";
import { useAmmProgram } from "@/hooks/useAmmProgram";
import { derivePoolPdas, toBn } from "@/lib/amm/pdas";
import { swapSchema } from "@/schemas/pool";

export type SwapPanelProps = {
  mintX: string;
  mintY: string;
};

/**
 * @description Form to swap an exact input amount across the pool.
 * @param props.mintX Mint X pubkey string for the active pool.
 * @param props.mintY Mint Y pubkey string for the active pool.
 * @returns Swap panel.
 */
export function SwapPanel({ mintX, mintY }: SwapPanelProps) {
  const program = useAmmProgram();
  const { publicKey } = useWallet();
  const [seed, setSeed] = useState("1");
  const [isX, setIsX] = useState(true);
  const [amountIn, setAmountIn] = useState("10000");
  const [minOut, setMinOut] = useState("1");
  const [status, setStatus] = useState<string | null>(null);
  const [error, setError] = useState<string | null>(null);

  async function submit() {
    setError(null);
    setStatus(null);
    if (!program || !publicKey) {
      setError("Connect a wallet first");
      return;
    }
    const parsed = swapSchema.safeParse({
      seed,
      isX,
      amountIn,
      minAmountOut: minOut,
    });
    if (!parsed.success) {
      setError(parsed.error.issues[0]?.message ?? "Invalid input");
      return;
    }

    try {
      const mintXKey = new PublicKey(mintX);
      const mintYKey = new PublicKey(mintY);
      const seedBn = toBn(parsed.data.seed);
      const { config } = derivePoolPdas(seedBn);
      const sig = await program.methods
        .swap(parsed.data.isX, toBn(parsed.data.amountIn), toBn(parsed.data.minAmountOut))
        .accountsPartial({
          user: publicKey,
          mintX: mintXKey,
          mintY: mintYKey,
          config,
          vaultX: getAssociatedTokenAddressSync(mintXKey, config, true),
          vaultY: getAssociatedTokenAddressSync(mintYKey, config, true),
          userX: getAssociatedTokenAddressSync(mintXKey, publicKey),
          userY: getAssociatedTokenAddressSync(mintYKey, publicKey),
          tokenProgram: TOKEN_PROGRAM_ID,
        })
        .rpc();
      setStatus(`Swapped · tx ${sig.slice(0, 8)}…`);
    } catch (err: unknown) {
      setError(err instanceof Error ? err.message : "Swap failed");
    }
  }

  return (
    <section className="panel" aria-labelledby="swap-heading">
      <h2 id="swap-heading">Swap</h2>
      <form onSubmit={(event) => onFormSubmit(event, submit)}>
        <AmountField id="sw-seed" label="Seed" value={seed} onChange={setSeed} />
        <fieldset className="direction">
          <legend>Direction</legend>
          <label>
            <input
              type="radio"
              name="direction"
              checked={isX}
              onChange={() => setIsX(true)}
              aria-label="Swap X to Y"
            />
            X → Y
          </label>
          <label>
            <input
              type="radio"
              name="direction"
              checked={!isX}
              onChange={() => setIsX(false)}
              aria-label="Swap Y to X"
            />
            Y → X
          </label>
        </fieldset>
        <AmountField id="sw-in" label="Amount in" value={amountIn} onChange={setAmountIn} />
        <AmountField id="sw-min-out" label="Min out" value={minOut} onChange={setMinOut} />
        <button type="submit">Swap</button>
      </form>
      <StatusBanner message={status} />
      <StatusBanner message={error} tone="error" />
    </section>
  );
}
