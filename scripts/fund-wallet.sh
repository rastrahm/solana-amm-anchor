#!/usr/bin/env bash
# Fund a Phantom (or any) wallet on localnet for the AMM demo.
# Usage: ./scripts/fund-wallet.sh <PUBKEY>
set -euo pipefail

PUBKEY="${1:?Usage: $0 <PUBKEY>}"
RPC="${RPC_URL:-http://127.0.0.1:8899}"
MINT_X="${MINT_X:-4WUBsGtoNS4FrwH8DAzabRARDKbcPKRadMntqZQHozkX}"
MINT_Y="${MINT_Y:-7FSzGXSAU8MeG5B1UVDS1QAxZLB7V8FhzrT2nucSd7SK}"

solana config set --url "$RPC" >/dev/null
solana airdrop 10 "$PUBKEY"

spl-token create-account "$MINT_X" --owner "$PUBKEY" --fee-payer ~/.config/solana/id.json || true
spl-token create-account "$MINT_Y" --owner "$PUBKEY" --fee-payer ~/.config/solana/id.json || true
spl-token transfer "$MINT_X" 100000 --recipient-owner "$PUBKEY" --fund-recipient --allow-unfunded-recipient
spl-token transfer "$MINT_Y" 100000 --recipient-owner "$PUBKEY" --fund-recipient --allow-unfunded-recipient

echo "Funded $PUBKEY with 10 SOL + 100000 of each demo mint."
