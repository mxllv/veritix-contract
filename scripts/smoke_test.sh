#!/usr/bin/env bash
# Testnet smoke test: deploy the compiled WASM and exercise core ops
# against the real Soroban network. Closes #558.
#
# Prerequisites: stellar CLI configured, contract already built
# (`make -C veritixpay/contract/token build`).
set -euo pipefail

NETWORK="testnet"
ACCOUNT="${STELLAR_ACCOUNT:-smoke-test}"
WASM_PATH="$(cd "$(dirname "$0")/.." && pwd)/target/wasm32v1-none/release/veritixpay.wasm"

echo "==> Funding $ACCOUNT via Friendbot..."
stellar keys generate "$ACCOUNT" --network "$NETWORK" --fund --overwrite
ADMIN=$(stellar keys address "$ACCOUNT" --network "$NETWORK")

echo "==> Deploying WASM..."
CONTRACT_ID=$(stellar contract deploy --wasm "$WASM_PATH" --source "$ACCOUNT" --network "$NETWORK")

invoke() { stellar contract invoke --id "$CONTRACT_ID" --source "$ACCOUNT" --network "$NETWORK" -- "$@"; }

echo "==> initialize..."
invoke initialize --admin "$ADMIN" --name SmokeTest --symbol SMK --decimal 7

echo "==> mint..."
invoke mint --admin "$ADMIN" --to "$ADMIN" --amount 1000000000
balance=$(invoke balance --id "$ADMIN")
[ "$balance" = "1000000000" ] || { echo "FAIL: balance after mint = $balance"; exit 1; }

echo "==> transfer..."
invoke transfer --from "$ADMIN" --to "$ADMIN" --amount 1
balance_after=$(invoke balance --id "$ADMIN")
[ "$balance_after" = "1000000000" ] || { echo "FAIL: balance after self-transfer = $balance_after"; exit 1; }

echo "==> create_escrow..."
expiry=$(($(date +%s) / 5 + 100000))
escrow_id=$(invoke create_escrow --depositor "$ADMIN" --beneficiary "$ADMIN" --amount 1000 --expiry_ledger "$expiry")
[ -n "$escrow_id" ] || { echo "FAIL: create_escrow returned no id"; exit 1; }

echo "Smoke test PASSED"
