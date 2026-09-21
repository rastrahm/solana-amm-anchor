"use client";

import { useState } from "react";
import { PublicKey } from "@solana/web3.js";
import {
  TOKEN_PROGRAM_ID,
  ASSOCIATED_TOKEN_PROGRAM_ID,
  getAssociatedTokenAddressSync,
} from "@solana/spl-token";
import { useWallet } from "@solana/wallet-adapter-react";
import { SystemProgram } from "@solana/web3.js";
import { AmountField, StatusBanner, onFormSubmit } from "@/components/pool/FormBits";
import { useAmmProgram } from "@/hooks/useAmmProgram";
import { derivePoolPdas, toBn } from "@/lib/amm/pdas";
import { initializeSchema } from "@/schemas/pool";

/**
 * @description Form to initialize a new constant-product pool on-chain.
 * @returns Initialize pool panel.
 */
export function InitializePanel() {
  const program = useAmmProgram();
  const { publicKey } = useWallet();
  const [seed, setSeed] = useState("1");
  const [feeBps, setFeeBps] = useState("30");
  const [mintX, setMintX] = useState("");
  const [mintY, setMintY] = useState("");
  const [status, setStatus] = useState<string | null>(null);
  const [error, setError] = useState<string | null>(null);

  async function submit() {
    setError(null);
    setStatus(null);
    if (!program || !publicKey) {
      setError("Connect a wallet first");
      return;
    }

    const parsed = initializeSchema.safeParse({ seed, feeBps, mintX, mintY });
    if (!parsed.success) {
      setError(parsed.error.issues[0]?.message ?? "Invalid input");
      return;
    }

    try {
      const mintXKey = new PublicKey(parsed.data.mintX);
      const mintYKey = new PublicKey(parsed.data.mintY);
      const seedBn = toBn(parsed.data.seed);
      const { config, mintLp } = derivePoolPdas(seedBn);
      const vaultX = getAssociatedTokenAddressSync(mintXKey, config, true);
      const vaultY = getAssociatedTokenAddressSync(mintYKey, config, true);

      const sig = await program.methods
        .initialize(seedBn, Number(parsed.data.feeBps), null)
        .accountsPartial({
          initializer: publicKey,
          mintX: mintXKey,
          mintY: mintYKey,
          config,
          mintLp,
          vaultX,
          vaultY,
          tokenProgram: TOKEN_PROGRAM_ID,
          associatedTokenProgram: ASSOCIATED_TOKEN_PROGRAM_ID,
          systemProgram: SystemProgram.programId,
        })
        .rpc();
      setStatus(`Initialized. Config ${config.toBase58()} · tx ${sig.slice(0, 8)}…`);
    } catch (err: unknown) {
      setError(err instanceof Error ? err.message : "Initialize failed");
    }
  }

  return (
    <section className="panel" aria-labelledby="initialize-heading">
      <h2 id="initialize-heading">Initialize pool</h2>
      <form onSubmit={(event) => onFormSubmit(event, submit)}>
        <AmountField id="init-seed" label="Seed" value={seed} onChange={setSeed} />
        <AmountField id="init-fee" label="Fee (bps)" value={feeBps} onChange={setFeeBps} />
        <AmountField id="init-mint-x" label="Mint X" value={mintX} onChange={setMintX} placeholder="Pubkey" />
        <AmountField id="init-mint-y" label="Mint Y" value={mintY} onChange={setMintY} placeholder="Pubkey" />
        <button type="submit">Initialize</button>
      </form>
      <StatusBanner message={status} />
      <StatusBanner message={error} tone="error" />
    </section>
  );
}
