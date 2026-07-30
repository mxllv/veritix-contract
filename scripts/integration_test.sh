#!/usr/bin/env bash
set -euo pipefail

PASS=0
FAIL=0

pass() {
  echo "PASS: $1"
  PASS=$((PASS + 1))
}

fail() {
  echo "FAIL: $1"
  FAIL=$((FAIL + 1))
}

echo "=== VeriTix Contract Integration Test ==="
echo ""

# 1. Deploy the contract
echo "--- Step 1: Deploy contract ---"
WASM=$(ls target/wasm32v1-none/release/*.wasm 2>/dev/null | head -1)
if [ -z "$WASM" ]; then
  fail "No WASM file found. Run 'make build' first."
else
  CONTRACT_ID=$(soroban contract deploy --wasm "$WASM" 2>&1) && pass "Contract deployed: $CONTRACT_ID" || fail "Deploy failed: $CONTRACT_ID"
fi

# 2. Initialize the contract
echo "--- Step 2: Initialize contract ---"
ADMIN="GAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAWHF"
if soroban contract invoke --id "$CONTRACT_ID" -- initialize --admin "$ADMIN" 2>&1; then
  pass "Contract initialized"
else
  fail "Initialization failed"
fi

# 3. Mint tokens
echo "--- Step 3: Mint tokens ---"
USER="GAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAQ"
if soroban contract invoke --id "$CONTRACT_ID" -- mint --admin "$ADMIN" --to "$USER" --amount 1000 2>&1; then
  pass "Minted 1000 tokens"
else
  fail "Mint failed"
fi

# 4. Check balance
echo "--- Step 4: Check balance ---"
BALANCE=$(soroban contract invoke --id "$CONTRACT_ID" -- balance --account "$USER" 2>&1)
if echo "$BALANCE" | grep -q "1000"; then
  pass "Balance is 1000"
else
  fail "Expected balance 1000, got: $BALANCE"
fi

# 5. Transfer tokens
echo "--- Step 5: Transfer tokens ---"
RECIPIENT="GAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAQ2"
MEMO="00"
if soroban contract invoke --id "$CONTRACT_ID" -- transfer_with_memo --from "$USER" --to "$RECIPIENT" --amount 100 --memo "$MEMO" 2>&1; then
  pass "Transferred 100 tokens"
else
  fail "Transfer failed"
fi

# 6. Create escrow
echo "--- Step 6: Create escrow ---"
BENEFICIARY="GAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAQ3"
EXPIRY=$(( $(soroban ledger sequence 2>/dev/null || echo 1000) + 1000 ))
if soroban contract invoke --id "$CONTRACT_ID" -- create_escrow --depositor "$USER" --beneficiary "$BENEFICIARY" --token "$USER" --amount 500 --expiry_ledger "$EXPIRY" --memo "00" 2>&1; then
  pass "Escrow created"
else
  fail "Escrow creation failed"
fi

# Summary
echo ""
echo "=== Results ==="
echo "PASS: $PASS"
echo "FAIL: $FAIL"

if [ "$FAIL" -gt 0 ]; then
  exit 1
fi
