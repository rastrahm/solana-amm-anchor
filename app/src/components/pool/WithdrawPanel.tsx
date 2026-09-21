"use client";

import { useState } from "react";
import { PublicKey } from "@solana/web3.js";
import { TOKEN_PROGRAM_ID, getAssociatedTokenAddressSync } from "@solana/spl-token";
import { useWallet } from "@solana/wallet-adapter-react";
import { AmountField, StatusBanner, onFormSubmit } from "@/components/pool/FormBits";
import { useAmmProgram } from "@/hooks/useAmmProgram";
import { derivePoolPdas, toBn } from "@/lib/amm/pdas";
import { withdrawSchema } from "@/schemas/pool";

export type WithdrawPanelProps = {
  mintX: string;
  mintY: string;
};

/**
 * @description Form to burn LP and withdraw proportional X/Y.
 * @param props.mintX Mint X pubkey string for the active pool.
 * @param props.mintY Mint Y pubkey string for the active pool.
 * @returns Withdraw liquidity panel.
 */
export function WithdrawPanel({ mintX, mintY }: WithdrawPanelProps) {
  const program = useAmmProgram();
  const { publicKey } = useWallet();
  const [seed, setSeed] = useState("1");
  const [lpAmount, setLpAmount] = useState("1000");
  const [minX, setMinX] = useState("0");
  const [minY, setMinY] = useState("0");
  const [status, setStatus] = useState<string | null>(null);
  const [error, setError] = useState<string | null>(null);

  async function submit() {
    setError(null);
    setStatus(null);
    if (!program || !publicKey) {
      setError("Connect a wallet first");
      return;
    }
    const parsed = withdrawSchema.safeParse({ seed, lpAmount, minX, minY });
    if (!parsed.success) {
      setError(parsed.error.issues[0]?.message ?? "Invalid input");
      return;
    }

    try {
      const mintXKey = new PublicKey(mintX);
      const mintYKey = new PublicKey(mintY);
      const seedBn = toBn(parsed.data.seed);
      const { config, mintLp } = derivePoolPdas(seedBn);
      const sig = await program.methods
        .withdraw(toBn(parsed.data.lpAmount), toBn(parsed.data.minX), toBn(parsed.data.minY))
        .accountsPartial({
          user: publicKey,
          mintX: mintXKey,
          mintY: mintYKey,
          mintLp,
          config,
          vaultX: getAssociatedTokenAddressSync(mintXKey, config, true),
          vaultY: getAssociatedTokenAddressSync(mintYKey, config, true),
          userX: getAssociatedTokenAddressSync(mintXKey, publicKey),
          userY: getAssociatedTokenAddressSync(mintYKey, publicKey),
          userLp: getAssociatedTokenAddressSync(mintLp, publicKey),
          tokenProgram: TOKEN_PROGRAM_ID,
        })
        .rpc();
      setStatus(`Withdrawn · tx ${sig.slice(0, 8)}…`);
    } catch (err: unknown) {
      setError(err instanceof Error ? err.message : "Withdraw failed");
    }
  }

  return (
    <section className="panel" aria-labelledby="withdraw-heading">
      <h2 id="withdraw-heading">Withdraw</h2>
      <form onSubmit={(event) => onFormSubmit(event, submit)}>
        <AmountField id="wd-seed" label="Seed" value={seed} onChange={setSeed} />
        <AmountField id="wd-lp" label="LP amount" value={lpAmount} onChange={setLpAmount} />
        <AmountField id="wd-min-x" label="Min X" value={minX} onChange={setMinX} />
        <AmountField id="wd-min-y" label="Min Y" value={minY} onChange={setMinY} />
        <button type="submit">Withdraw</button>
      </form>
      <StatusBanner message={status} />
      <StatusBanner message={error} tone="error" />
    </section>
  );
}
