"use client";

import { useState } from "react";
import { PublicKey, SystemProgram } from "@solana/web3.js";
import {
  ASSOCIATED_TOKEN_PROGRAM_ID,
  TOKEN_PROGRAM_ID,
  getAssociatedTokenAddressSync,
} from "@solana/spl-token";
import { useWallet } from "@solana/wallet-adapter-react";
import { AmountField, StatusBanner, onFormSubmit } from "@/components/pool/FormBits";
import { useAmmProgram } from "@/hooks/useAmmProgram";
import { derivePoolPdas, toBn } from "@/lib/amm/pdas";
import { depositSchema } from "@/schemas/pool";
import { useI18n } from "@/lib/i18n/I18nProvider";

export type DepositPanelProps = {
  mintX: string;
  mintY: string;
};

/**
 * @description Form to deposit X/Y liquidity into an existing pool.
 * @param props.mintX Mint X pubkey string for the active pool.
 * @param props.mintY Mint Y pubkey string for the active pool.
 * @returns Deposit liquidity panel.
 */
export function DepositPanel({ mintX, mintY }: DepositPanelProps) {
  const { t } = useI18n();
  const program = useAmmProgram();
  const { publicKey } = useWallet();
  const [seed, setSeed] = useState("1");
  const [amountX, setAmountX] = useState("1000000");
  const [amountY, setAmountY] = useState("1000000");
  const [minLp, setMinLp] = useState("0");
  const [status, setStatus] = useState<string | null>(null);
  const [error, setError] = useState<string | null>(null);

  async function submit() {
    setError(null);
    setStatus(null);
    if (!program || !publicKey) {
      setError(t.connectWalletFirst);
      return;
    }
    const parsed = depositSchema.safeParse({ seed, amountX, amountY, minLp });
    if (!parsed.success) {
      setError(parsed.error.issues[0]?.message ?? t.invalidInput);
      return;
    }

    try {
      const mintXKey = new PublicKey(mintX);
      const mintYKey = new PublicKey(mintY);
      const seedBn = toBn(parsed.data.seed);
      const { config, mintLp } = derivePoolPdas(seedBn);
      const vaultX = getAssociatedTokenAddressSync(mintXKey, config, true);
      const vaultY = getAssociatedTokenAddressSync(mintYKey, config, true);
      const userX = getAssociatedTokenAddressSync(mintXKey, publicKey);
      const userY = getAssociatedTokenAddressSync(mintYKey, publicKey);
      const userLp = getAssociatedTokenAddressSync(mintLp, publicKey);
      const lockLp = getAssociatedTokenAddressSync(mintLp, config, true);

      const sig = await program.methods
        .deposit(toBn(parsed.data.amountX), toBn(parsed.data.amountY), toBn(parsed.data.minLp))
        .accountsPartial({
          user: publicKey,
          mintX: mintXKey,
          mintY: mintYKey,
          mintLp,
          config,
          vaultX,
          vaultY,
          userX,
          userY,
          userLp,
          lockLp,
          tokenProgram: TOKEN_PROGRAM_ID,
          associatedTokenProgram: ASSOCIATED_TOKEN_PROGRAM_ID,
          systemProgram: SystemProgram.programId,
        })
        .rpc();
      setStatus(t.deposited(sig.slice(0, 8)));
    } catch (err: unknown) {
      setError(err instanceof Error ? err.message : t.depositFailed);
    }
  }

  return (
    <section className="panel" aria-labelledby="deposit-heading">
      <h2 id="deposit-heading">{t.depositTitle}</h2>
      <form onSubmit={(event) => onFormSubmit(event, submit)}>
        <AmountField id="dep-seed" label={t.seed} value={seed} onChange={setSeed} />
        <AmountField id="dep-x" label={t.amountX} value={amountX} onChange={setAmountX} />
        <AmountField id="dep-y" label={t.amountY} value={amountY} onChange={setAmountY} />
        <AmountField id="dep-min-lp" label={t.minLp} value={minLp} onChange={setMinLp} />
        <button type="submit">{t.actionDeposit}</button>
      </form>
      <StatusBanner message={status} />
      <StatusBanner message={error} tone="error" />
    </section>
  );
}
