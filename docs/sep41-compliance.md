# SEP-41 Token Interface Compliance

> **Last audited:** 2026-07-27  
> Resolves [#563](https://github.com/Lead-Studios/veritix-contract/issues/563)

This document maps every function required by the [SEP-41 Token Interface](https://github.com/stellar/stellar-protocol/blob/master/ecosystem/sep-0041.md) standard against the current implementation in `veritixpay/contract/token/src/contract.rs`.

---

## Compliance Table

| SEP-41 Function | Status | Notes |
|---|---|---|
| `name() → String` | ✅ Implemented | `VeritixToken::name` (line 295) |
| `symbol() → String` | ✅ Implemented | `VeritixToken::symbol` (line 298) |
| `decimals() → u32` | ✅ Implemented | `VeritixToken::decimals` (line 292) |
| `balance(id: Address) → i128` | ✅ Implemented | `VeritixToken::balance` (line 247) |
| `spendable_balance(id: Address) → i128` | ❌ Missing | See note below |
| `authorized(id: Address) → bool` | ❌ Missing | See note below |
| `transfer(from, to, amount)` | ✅ Implemented | `VeritixToken::transfer` (line 177) |
| `transfer_from(spender, from, to, amount)` | ✅ Implemented | `VeritixToken::transfer_from` (line 189) |
| `burn(from, amount)` | ✅ Implemented | `VeritixToken::burn` (line 131) |
| `burn_from(spender, from, amount)` | ✅ Implemented | `VeritixToken::burn_from` (line 139) |
| `clawback(from, amount)` | ✅ Implemented | `VeritixToken::clawback` (line 154) |
| `set_authorized(id, authorize: bool)` | ❌ Missing | See note below |
| `mint(to, amount)` | ✅ Implemented | `VeritixToken::mint` (line 120) |
| `set_admin(new_admin)` | ✅ Implemented | `VeritixToken::set_admin` (line 92) |

**Compliance score: 11 / 14 (78.6%)**

---

## Missing Functions

### `spendable_balance(id: Address) → i128`

SEP-41 requires `spendable_balance` to return the portion of a holder's balance that can actually be transferred (i.e. excluding any frozen or liened amount). Currently the contract exposes `balance` (total balance) but does not separate spendable from held funds.

**Impact:** Wallets and DEXs that use `spendable_balance` to size swap amounts may operate on stale data and create failed transactions for frozen accounts.

**Resolution:** Open an issue to implement `spendable_balance` that returns `balance - frozen_amount` (or `0` for frozen accounts).

---

### `authorized(id: Address) → bool`

SEP-41 requires `authorized` to report whether a given account is permitted to send and receive tokens. The contract implements freeze (`freeze_account`, `is_frozen`) but does not expose the canonical `authorized` name expected by the SEP-41 ABI.

**Impact:** Tools that query `authorized` (e.g., Stellar Laboratory, indexers) will receive a missing-function error.

**Resolution:** Add `pub fn authorized(e: Env, id: Address) -> bool` as an alias for `!is_frozen(e, id)`.

---

### `set_authorized(id: Address, authorize: bool)`

SEP-41 requires `set_authorized` to enable or disable token transfers for a specific account. The contract exposes `freeze(target)` and `unfreeze(target)` separately, but does not expose the unified `set_authorized(id, true/false)` entry point expected by the SEP-41 ABI.

**Impact:** Token control tools that call `set_authorized` will not find the function in the ABI and will fall back to manual freeze/unfreeze lookups.

**Resolution:** Add `pub fn set_authorized(e: Env, admin: Address, id: Address, authorize: bool)` that calls `freeze_account` or `unfreeze_account` based on the `authorize` flag.

---

## Extended Functions (Beyond SEP-41)

The following functions are implemented in addition to the SEP-41 baseline and provide Veritix-specific functionality:

| Function | Purpose |
|---|---|
| `initialize_with_max_supply` | Supply-capped initialization |
| `transfer_with_memo` | Memo-carrying transfer (Veritix ticket ref) |
| `approve` / `allowance` | ERC-20-style allowance |
| `freeze` / `unfreeze` | Account freeze (equivalent to `set_authorized`) |
| `freeze_batch` / `unfreeze_batch` | Bulk freeze for compliance sweeps |
| `clawback_batch` | Bulk clawback |
| `approve_batch` | Bulk approval |
| `transfer_batch_with_memo` | Multi-recipient transfer with memo |
| `mint` (admin-gated) | Admin-only mint (standard SEP-41 allows open mint) |
| `create_escrow` / `release_escrow` / `refund_escrow` | Escrow lifecycle |
| `admin_settle_escrow` | Admin dispute resolution |
| `open_dispute` / `resolve_dispute` / `expire_dispute` | Dispute workflow |
| `create_split` / `distribute` / `cancel_split` | Payment splitting |
| `setup_recurring` / `execute_recurring` / `cancel_recurring` | Recurring payments |
| `pause` / `unpause` | Emergency pause |
| `token_info` | Composite metadata view |
| `contract_stats` | Aggregate on-chain statistics |

---

## References

- [SEP-41 Token Interface Specification](https://github.com/stellar/stellar-protocol/blob/master/ecosystem/sep-0041.md)
- [Soroban Token Example (reference implementation)](https://github.com/stellar/soroban-examples/tree/main/token)
- Related issues: [#563](https://github.com/Lead-Studios/veritix-contract/issues/563)
