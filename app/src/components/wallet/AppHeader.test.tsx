import { afterEach, beforeEach, describe, expect, it, vi } from "vitest";
import { cleanup, render, screen } from "@testing-library/react";
import userEvent from "@testing-library/user-event";
import { AppHeader } from "@/components/wallet/AppHeader";
import { I18nProvider } from "@/lib/i18n/I18nProvider";
import { ThemeProvider } from "@/lib/theme/ThemeProvider";
import { LOCALE_STORAGE_KEY, THEME_STORAGE_KEY } from "@/lib/i18n/dictionaries";

vi.mock("@solana/wallet-adapter-react-ui", () => ({
  WalletMultiButton: () => <button type="button">Select Wallet</button>,
}));

vi.mock("@/lib/amm/cluster", () => ({
  getClusterName: () => "localnet",
}));

function renderHeader() {
  return render(
    <ThemeProvider>
      <I18nProvider>
        <AppHeader />
      </I18nProvider>
    </ThemeProvider>
  );
}

describe("AppHeader", () => {
  beforeEach(() => {
    window.localStorage.setItem(LOCALE_STORAGE_KEY, "es");
    window.localStorage.setItem(THEME_STORAGE_KEY, "light");
  });

  afterEach(() => {
    cleanup();
  });

  it("shows brand, cluster, wallet, and preference controls", async () => {
    renderHeader();
    expect(screen.getByText("solana-amm")).toBeInTheDocument();
    expect(await screen.findByLabelText("cluster localnet")).toBeInTheDocument();
    expect(
      screen.getByRole("button", { name: /select wallet/i })
    ).toBeInTheDocument();
    expect(
      await screen.findByRole("button", { name: /modo oscuro/i })
    ).toBeInTheDocument();
    expect(
      screen.getByRole("button", { name: /switch to english/i })
    ).toBeInTheDocument();
    expect(
      screen.getByRole("button", { name: /abrir ayuda/i })
    ).toBeInTheDocument();
  });

  it("opens bilingual help dialog", async () => {
    const user = userEvent.setup();
    renderHeader();
    await user.click(await screen.findByRole("button", { name: /abrir ayuda/i }));
    expect(
      screen.getByRole("dialog", { name: /cómo usar esta demo/i })
    ).toBeInTheDocument();
    expect(screen.getByText(/flujo sugerido/i)).toBeInTheDocument();
  });
});
