import { describe, expect, it, vi } from "vitest";
import { render, screen } from "@testing-library/react";
import userEvent from "@testing-library/user-event";
import { AmountField, StatusBanner } from "@/components/pool/FormBits";

describe("AmountField", () => {
  it("exposes an accessible labeled input", async () => {
    const user = userEvent.setup();
    const onChange = vi.fn();
    render(
      <AmountField id="amount-x" label="Amount X" value="" onChange={onChange} />
    );
    const input = screen.getByLabelText("Amount X");
    await user.type(input, "1000");
    expect(onChange).toHaveBeenCalled();
  });
});

describe("StatusBanner", () => {
  it("renders a status message", () => {
    render(<StatusBanner message="Pool ready" />);
    expect(screen.getByRole("status")).toHaveTextContent("Pool ready");
  });

  it("hides when message is null", () => {
    const { container } = render(<StatusBanner message={null} />);
    expect(container).toBeEmptyDOMElement();
  });
});
