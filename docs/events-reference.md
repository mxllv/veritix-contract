# Events Reference

All events emitted by the contract in `src/`, for off-chain indexers
(NestJS backend, Horizon SSE listeners). Closes #560.

Note: `splitter.rs` and `recurring.rs`'s `execute_recurring` do not currently
emit events — only the events below exist on-chain today.

## Escrow module (`src/escrow.rs`)

| Event | Topics `(name, depositor, beneficiary)` | Data | When |
| --- | --- | --- | --- |
| `escrow_cr` | `("escrow_cr", depositor, beneficiary)` | `(amount, memo)` | `create_escrow` succeeds |
| `escrow_rl` | `("escrow_rl", depositor, beneficiary)` | `(remaining, memo)` | `release_escrow` / `release_partial_escrow` |
| `escrow_rf` | `("escrow_rf", depositor, beneficiary)` | `(refundable, memo)` | `refund_escrow` succeeds |

Example (`escrow_cr`):
```
topics: ["escrow_cr", <depositor addr>, <beneficiary addr>]
data:   [1_000_0000000, "invoice-42"]
```

## Dispute module (`src/dispute.rs`)

| Event | Topics `(name,)` | Data | When |
| --- | --- | --- | --- |
| `dispute` | `("dispute",)` | `(caller, escrow_id)` | `open_dispute` opens a new dispute |
| `dis_res` | `("dis_res",)` | `(resolver, escrow_id, winner)` | `resolve_dispute` picks a winner |

## Admin module (`src/admin.rs`)

| Event | Topics `(name,)` | Data | When |
| --- | --- | --- | --- |
| `ownership` | `("ownership",)` | `(current_admin, new_admin)` | `transfer_ownership` proposes a new admin |
| `admin_set` | `("admin_set",)` | `(new_admin, activation_ledger)` | `accept_admin` activates the proposed admin |
