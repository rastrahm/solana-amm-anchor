"use client";

import type { ReactNode } from "react";
import { WalletContextProvider } from "@/components/wallet/WalletContextProvider";
import { I18nProvider } from "@/lib/i18n/I18nProvider";
import { ThemeProvider } from "@/lib/theme/ThemeProvider";

export type AppProvidersProps = {
  children: ReactNode;
};

/**
 * @description Composes theme, i18n, and Solana wallet providers for the app shell.
 * @param props.children App content.
 * @returns Nested client providers.
 */
export function AppProviders({ children }: AppProvidersProps) {
  return (
    <ThemeProvider>
      <I18nProvider>
        <WalletContextProvider>{children}</WalletContextProvider>
      </I18nProvider>
    </ThemeProvider>
  );
}
