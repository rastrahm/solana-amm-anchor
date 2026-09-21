import { describe, expect, it } from "vitest";
import {
  depositSchema,
  initializeSchema,
  swapSchema,
  withdrawSchema,
} from "@/schemas/pool";

describe("pool schemas", () => {
  it("accepts a valid initialize payload", () => {
    const parsed = initializeSchema.safeParse({
      seed: "42",
      feeBps: "30",
      mintX: "So11111111111111111111111111111111111111112",
      mintY: "EPjFWdd5AufqSSqeM2qN1xzybapC8G4wEGGkZwyTDt1v",
    });
    expect(parsed.success).toBe(true);
  });

  it("rejects fee above 10000 bps", () => {
    const parsed = initializeSchema.safeParse({
      seed: "1",
      feeBps: "10001",
      mintX: "So11111111111111111111111111111111111111112",
      mintY: "EPjFWdd5AufqSSqeM2qN1xzybapC8G4wEGGkZwyTDt1v",
    });
    expect(parsed.success).toBe(false);
  });

  it("rejects zero deposit amounts", () => {
    const parsed = depositSchema.safeParse({
      seed: "1",
      amountX: "0",
      amountY: "100",
      minLp: "0",
    });
    expect(parsed.success).toBe(false);
  });

  it("accepts a valid swap payload", () => {
    const parsed = swapSchema.safeParse({
      seed: "1",
      isX: true,
      amountIn: "1000",
      minAmountOut: "1",
    });
    expect(parsed.success).toBe(true);
  });

  it("accepts a valid withdraw payload", () => {
    const parsed = withdrawSchema.safeParse({
      seed: "1",
      lpAmount: "500",
      minX: "0",
      minY: "0",
    });
    expect(parsed.success).toBe(true);
  });
});
