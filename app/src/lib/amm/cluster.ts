/**
 * @description Cluster / RPC helpers for the demo UI.
 */

export type ClusterName = "localnet" | "devnet";

/**
 * @description Resolves the Solana RPC endpoint from environment.
 * @returns HTTP RPC URL for the wallet adapter connection.
 */
export function getRpcEndpoint(): string {
  return process.env.NEXT_PUBLIC_SOLANA_RPC_URL ?? "http://127.0.0.1:8899";
}

/**
 * @description Resolves the logical cluster name for UI labels.
 * @returns Cluster name shown in the header.
 */
export function getClusterName(): ClusterName {
  const value = process.env.NEXT_PUBLIC_SOLANA_CLUSTER ?? "localnet";
  return value === "devnet" ? "devnet" : "localnet";
}
