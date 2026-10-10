#!/usr/bin/env bash
# Fund a Phantom (or any) wallet on localnet for the AMM demo.
# Usage: ./scripts/fund-wallet.sh <PUBKEY>
set -eu

PUBKEY="${1:?Usage: $0 <PUBKEY>}"
RPC="${RPC_URL:-http://127.0.0.1:8899}"
MINT_X="${MINT_X:-2bLvSSNZTD8GQ1MJiw2U6rW9cbNR7C6qrjLLVxszGLSZ}"
MINT_Y="${MINT_Y:-AovraHMmHYj24vPYJiVjhFzYjg1FcWpTLb9RgqxK6g23}"

solana config set --url "$RPC" >/dev/null
solana airdrop 10 "$PUBKEY" >/dev/null

spl-token transfer "$MINT_X" 100000 "$PUBKEY" --fund-recipient --allow-unfunded-recipient
spl-token transfer "$MINT_Y" 100000 "$PUBKEY" --fund-recipient --allow-unfunded-recipient

echo "Funded $PUBKEY with 10 SOL + 100000 of each demo mint."
