/** @vitest-environment node */
import { describe, expect, it } from "vitest";
import BN from "bn.js";
import { derivePoolPdas, PROGRAM_ID } from "@/lib/amm/pdas";

describe("derivePoolPdas", () => {
  it("returns stable PDAs for the same seed", () => {
    const a = derivePoolPdas(new BN(42));
    const b = derivePoolPdas(new BN(42));
    expect(a.config.equals(b.config)).toBe(true);
    expect(a.mintLp.equals(b.mintLp)).toBe(true);
    expect(a.configBump).toBe(b.configBump);
  });

  it("uses the declared program id", () => {
    const { config } = derivePoolPdas(new BN(1), PROGRAM_ID);
    expect(config.toBase58().length).toBeGreaterThan(30);
  });
});
