"use client";

import { useState } from "react";
import { WalletMultiButton } from "@solana/wallet-adapter-react-ui";
import { getClusterName } from "@/lib/amm/cluster";
import { useI18n } from "@/lib/i18n/I18nProvider";
import { useTheme } from "@/lib/theme/ThemeProvider";
import { HelpDialog } from "@/components/ui/HelpDialog";

/**
 * @description Header with brand, cluster, theme/locale/help controls, and wallet connect.
 * @returns Top navigation composition for the AMM demo.
 */
export function AppHeader() {
  const cluster = getClusterName();
  const { locale, t, toggleLocale } = useI18n();
  const { theme, toggleTheme } = useTheme();
  const [helpOpen, setHelpOpen] = useState(false);

  return (
    <header className="app-header">
      <div className="brand-block">
        <p className="brand">solana-amm</p>
        <p className="cluster" aria-label={t.brandCluster(cluster)}>
          {cluster}
        </p>
      </div>
      <div className="header-actions">
        <button
          type="button"
          className="icon-btn locale-btn"
          onClick={toggleTheme}
          aria-label={theme === "dark" ? t.themeToLight : t.themeToDark}
          title={theme === "dark" ? t.themeToLight : t.themeToDark}
        >
          {theme === "dark" ? t.themeLightLabel : t.themeDarkLabel}
        </button>
        <button
          type="button"
          className="icon-btn locale-btn"
          onClick={toggleLocale}
          aria-label={locale === "es" ? t.localeToEn : t.localeToEs}
          title={locale === "es" ? t.localeToEn : t.localeToEs}
        >
          {locale === "es" ? "EN" : "ES"}
        </button>
        <button
          type="button"
          className="icon-btn"
          onClick={() => setHelpOpen(true)}
          aria-label={t.helpOpen}
          title={t.helpOpen}
        >
          ?
        </button>
        <WalletMultiButton />
      </div>
      <HelpDialog open={helpOpen} onClose={() => setHelpOpen(false)} />
    </header>
  );
}
