import { describe, expect, it, vi } from "vitest";
import { render, screen } from "@testing-library/react";
import { PoolWorkbench } from "@/components/pool/PoolWorkbench";
import { I18nProvider } from "@/lib/i18n/I18nProvider";

vi.mock("@/components/pool/InitializePanel", () => ({
  InitializePanel: () => <section aria-label="initialize-stub">Initialize</section>,
}));
vi.mock("@/components/pool/DepositPanel", () => ({
  DepositPanel: () => <section aria-label="deposit-stub">Deposit</section>,
}));
vi.mock("@/components/pool/WithdrawPanel", () => ({
  WithdrawPanel: () => <section aria-label="withdraw-stub">Withdraw</section>,
}));
vi.mock("@/components/pool/SwapPanel", () => ({
  SwapPanel: () => <section aria-label="swap-stub">Swap</section>,
}));

describe("PoolWorkbench", () => {
  it("exposes mint fields and operation panels", () => {
    render(
      <I18nProvider>
        <PoolWorkbench />
      </I18nProvider>
    );
    expect(screen.getByLabelText("Mint X")).toBeInTheDocument();
    expect(screen.getByLabelText("Mint Y")).toBeInTheDocument();
    expect(screen.getByLabelText("initialize-stub")).toBeInTheDocument();
    expect(screen.getByLabelText("deposit-stub")).toBeInTheDocument();
    expect(screen.getByLabelText("swap-stub")).toBeInTheDocument();
    expect(screen.getByLabelText("withdraw-stub")).toBeInTheDocument();
  });
});
