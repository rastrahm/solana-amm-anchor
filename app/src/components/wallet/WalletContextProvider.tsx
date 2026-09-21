"use client";

import { useMemo, type ReactNode } from "react";
import {
  ConnectionProvider,
  WalletProvider,
} from "@solana/wallet-adapter-react";
import { WalletModalProvider } from "@solana/wallet-adapter-react-ui";
import { PhantomWalletAdapter } from "@solana/wallet-adapter-phantom";
import { getRpcEndpoint } from "@/lib/amm/cluster";

import "@solana/wallet-adapter-react-ui/styles.css";

export type WalletContextProviderProps = {
  children: ReactNode;
};

/**
 * @description Provides Solana connection + wallet adapter context to the app tree.
 * @param props.children Descendant React nodes that need wallet access.
 * @returns Provider wrapper for connection, wallets, and modal UI.
 */
export function WalletContextProvider({
  children,
}: WalletContextProviderProps) {
  const endpoint = useMemo(() => getRpcEndpoint(), []);
  const wallets = useMemo(() => [new PhantomWalletAdapter()], []);

  return (
    <ConnectionProvider endpoint={endpoint}>
      <WalletProvider wallets={wallets} autoConnect>
        <WalletModalProvider>{children}</WalletModalProvider>
      </WalletProvider>
    </ConnectionProvider>
  );
}
