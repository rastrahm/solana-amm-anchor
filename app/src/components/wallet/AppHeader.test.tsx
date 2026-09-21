import { describe, expect, it, vi } from "vitest";
import { render, screen } from "@testing-library/react";
import { AppHeader } from "@/components/wallet/AppHeader";

vi.mock("@solana/wallet-adapter-react-ui", () => ({
  WalletMultiButton: () => <button type="button">Select Wallet</button>,
}));

vi.mock("@/lib/amm/cluster", () => ({
  getClusterName: () => "localnet",
}));

describe("AppHeader", () => {
  it("shows brand and cluster, and a wallet control", () => {
    render(<AppHeader />);
    expect(screen.getByText("solana-amm")).toBeInTheDocument();
    expect(screen.getByLabelText("cluster localnet")).toBeInTheDocument();
    expect(
      screen.getByRole("button", { name: /select wallet/i })
    ).toBeInTheDocument();
  });
});
