import type { Metadata } from "next";
import type { ReactNode } from "react";
import { AppProviders } from "@/components/providers/AppProviders";
import { AppHeader } from "@/components/wallet/AppHeader";
import { THEME_STORAGE_KEY } from "@/lib/i18n/dictionaries";
import "./globals.css";

export const metadata: Metadata = {
  title: "solana-amm",
  description: "Constant-product AMM demo client for the Anchor program",
};

export type RootLayoutProps = {
  children: ReactNode;
};

const themeBootScript = `
(function(){
  try {
    var k=${JSON.stringify(THEME_STORAGE_KEY)};
    var t=localStorage.getItem(k);
    if(t!=="light"&&t!=="dark"){
      t=window.matchMedia("(prefers-color-scheme: dark)").matches?"dark":"light";
    }
    document.documentElement.setAttribute("data-theme", t);
  } catch (e) {}
})();
`;

/**
 * @description Root App Router layout with wallet / theme / i18n providers and global chrome.
 * @param props.children Page content.
 * @returns HTML document shell.
 */
export default function RootLayout({ children }: RootLayoutProps) {
  return (
    <html lang="es" suppressHydrationWarning>
      <head>
        <script dangerouslySetInnerHTML={{ __html: themeBootScript }} />
      </head>
      <body>
        <AppProviders>
          <div className="app-shell">
            <AppHeader />
            {children}
          </div>
        </AppProviders>
      </body>
    </html>
  );
}
