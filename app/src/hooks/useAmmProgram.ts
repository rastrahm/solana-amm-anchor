"use client";

import { useMemo } from "react";
import { AnchorProvider, Program } from "@coral-xyz/anchor";
import { useConnection, useWallet } from "@solana/wallet-adapter-react";
import idl from "@/lib/idl/amm.json";
import type { Amm } from "@/lib/idl/amm";
import { PROGRAM_ID } from "@/lib/amm/pdas";

/**
 * @description Builds an Anchor `Program<Amm>` bound to the connected wallet.
 * @returns Program instance when a wallet is connected; otherwise `null`.
 */
export function useAmmProgram(): Program<Amm> | null {
  const { connection } = useConnection();
  const wallet = useWallet();

  return useMemo(() => {
    if (!wallet.publicKey || !wallet.signTransaction || !wallet.signAllTransactions) {
      return null;
    }

    const provider = new AnchorProvider(
      connection,
      {
        publicKey: wallet.publicKey,
        signTransaction: wallet.signTransaction,
        signAllTransactions: wallet.signAllTransactions,
      },
      { commitment: "confirmed" }
    );

    return new Program<Amm>(idl as Amm, provider);
  }, [connection, wallet]);
}

/**
 * @description Exposes the on-chain program id used by the UI.
 * @returns Program public key.
 */
export function useProgramId() {
  return PROGRAM_ID;
}
