import { z } from "zod";

/**
 * @description Positive integer string used for on-chain u64 amounts.
 */
const positiveAmount = z
  .string()
  .trim()
  .regex(/^\d+$/, "Must be a non-negative integer")
  .refine((value) => BigInt(value) > 0n, "Must be greater than zero");

/**
 * @description Schema for pool initialization form values.
 */
export const initializeSchema = z.object({
  seed: z
    .string()
    .trim()
    .regex(/^\d+$/, "Seed must be an integer")
    .refine((value) => BigInt(value) >= 0n, "Seed must be >= 0"),
  feeBps: z
    .string()
    .trim()
    .regex(/^\d+$/, "Fee must be an integer")
    .refine((value) => {
      const fee = Number(value);
      return fee >= 0 && fee <= 10_000;
    }, "Fee must be between 0 and 10000 bps"),
  mintX: z.string().trim().min(32, "Mint X is required"),
  mintY: z.string().trim().min(32, "Mint Y is required"),
});

/**
 * @description Schema for deposit liquidity form values.
 */
export const depositSchema = z.object({
  seed: z.string().trim().regex(/^\d+$/, "Seed must be an integer"),
  amountX: positiveAmount,
  amountY: positiveAmount,
  minLp: z
    .string()
    .trim()
    .regex(/^\d+$/, "minLp must be an integer"),
});

/**
 * @description Schema for withdraw liquidity form values.
 */
export const withdrawSchema = z.object({
  seed: z.string().trim().regex(/^\d+$/, "Seed must be an integer"),
  lpAmount: positiveAmount,
  minX: z.string().trim().regex(/^\d+$/, "minX must be an integer"),
  minY: z.string().trim().regex(/^\d+$/, "minY must be an integer"),
});

/**
 * @description Schema for swap form values.
 */
export const swapSchema = z.object({
  seed: z.string().trim().regex(/^\d+$/, "Seed must be an integer"),
  isX: z.boolean(),
  amountIn: positiveAmount,
  minAmountOut: z.string().trim().regex(/^\d+$/, "minAmountOut must be an integer"),
});

export type InitializeInput = z.infer<typeof initializeSchema>;
export type DepositInput = z.infer<typeof depositSchema>;
export type WithdrawInput = z.infer<typeof withdrawSchema>;
export type SwapInput = z.infer<typeof swapSchema>;
