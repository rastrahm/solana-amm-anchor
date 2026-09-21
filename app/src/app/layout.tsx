import type { Metadata } from "next";
import type { ReactNode } from "react";
import { WalletContextProvider } from "@/components/wallet/WalletContextProvider";
import { AppHeader } from "@/components/wallet/AppHeader";
import "./globals.css";

export const metadata: Metadata = {
  title: "solana-amm",
  description: "Constant-product AMM demo client for the Anchor program",
};

export type RootLayoutProps = {
  children: ReactNode;
};

/**
 * @description Root App Router layout with wallet providers and global chrome.
 * @param props.children Page content.
 * @returns HTML document shell.
 */
export default function RootLayout({ children }: RootLayoutProps) {
  return (
    <html lang="en">
      <body>
        <WalletContextProvider>
          <div className="app-shell">
            <AppHeader />
            {children}
          </div>
        </WalletContextProvider>
      </body>
    </html>
  );
}
