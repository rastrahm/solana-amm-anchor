import { PublicKey } from "@solana/web3.js";
import BN from "bn.js";

export const PROGRAM_ID = new PublicKey(
  "DR4UwHAVE9tVSm1kJo89ZiV6Dk1SXVPCPAhg67LT99mD"
);

export const CONFIG_SEED = new TextEncoder().encode("config");
export const LP_SEED = new TextEncoder().encode("lp");

/**
 * @description Encodes a u64 seed as 8 little-endian bytes for PDA derivation.
 * @param seed Pool seed.
 * @returns 8-byte little-endian array.
 */
export function seedToLeBytes(seed: BN): Uint8Array {
  return Uint8Array.from(seed.toArray("le", 8));
}

/**
 * @description Derives Config and LP mint PDAs for a pool seed.
 * @param seed Pool seed used at initialize time.
 * @param programId AMM program id (defaults to declared id).
 * @returns Config PDA, LP mint PDA, and their bumps.
 */
export function derivePoolPdas(
  seed: BN,
  programId: PublicKey = PROGRAM_ID
): {
  config: PublicKey;
  configBump: number;
  mintLp: PublicKey;
  lpBump: number;
} {
  const [config, configBump] = PublicKey.findProgramAddressSync(
    [CONFIG_SEED, seedToLeBytes(seed)],
    programId
  );
  const [mintLp, lpBump] = PublicKey.findProgramAddressSync(
    [LP_SEED, config.toBytes()],
    programId
  );
  return { config, configBump, mintLp, lpBump };
}

/**
 * @description Parses a decimal string into a BN for Anchor instruction args.
 * @param value Non-negative integer string.
 * @returns BN representation.
 */
export function toBn(value: string): BN {
  return new BN(value, 10);
}
