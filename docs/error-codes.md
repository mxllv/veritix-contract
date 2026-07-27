# Error and panic code catalog

This catalog documents the panic strings emitted by the contract modules so backend engineers can map Soroban failures to their cause and the recommended caller handling without grepping the Rust source.

## Conventions

- Panic strings are listed exactly as emitted by the contract.
- The recommended handling is a practical guide for callers and integrators.
- In general:
  - Retry: transient state or timing issue, usually after waiting for a ledger or re-submitting once the precondition is satisfied.
  - User error: invalid input, unauthorized action, or a business-rule violation that should be surfaced to the user.
  - System error: unexpected contract state, missing records, or a bug-like condition that should be logged and investigated.

## Token

| Panic string | Module / function(s) | Cause | Caller handling |
| --- | --- | --- | --- |
| `AlreadyInitialized` | contract initialization | The contract was initialized more than once. | System error; treat as a deployment/configuration issue. |
| `InvalidRecipient: cannot transfer directly to the contract address — use create_escrow instead` | contract transfer helpers | A transfer target was the contract address. | User error; use the escrow flow instead. |
| `InvalidClawback: cannot clawback from the contract address` | contract clawback flow | The clawback target was the contract address. | User error; choose a valid recipient. |
| `EvidenceTooLong: evidence cannot exceed 128 bytes` | dispute::open_dispute | The dispute evidence payload exceeded the size limit. | User error; shorten the evidence payload. |
| `InvalidExpiry: expiry_ledger must be in the future` | dispute::open_dispute | The requested dispute expiry ledger was not strictly in the future. | User error; pass a future ledger. |
| `DisputeAlreadyOpen: An open dispute already exists for this escrow` | dispute::open_dispute | A second dispute was opened for the same unresolved escrow. | User error; wait for the existing dispute to resolve or expire. |
| `AlreadyResolved: This dispute has already been resolved` | dispute::resolve_dispute / resolve_dispute_with_note | The dispute was already resolved or expired. | User error; avoid duplicate resolution attempts. |
| `UnauthorizedResolver: Only the designated resolver can resolve this` | dispute::resolve_dispute / resolve_dispute_with_note | The caller was not the designated resolver. | User error; use the configured resolver. |
| `NotExpired: expiry ledger has not been reached` | dispute::expire_dispute | The dispute was expired before the expiry ledger. | Retry or user error; wait until the expiry ledger is reached. |
| `NotResolved: dispute must be resolved before it can be appealed` | dispute::appeal_dispute | An appeal was attempted before the dispute had a terminal state. | User error; resolve the dispute first. |
| `NotAppealed: dispute is not in appealed state` | dispute::resolve_appeal | An appeal resolution was attempted on a non-appealed dispute. | User error; the dispute is not in appeal state. |
| `Unauthorized: only the claimant can appeal` | dispute::appeal_dispute | A non-claimant attempted to appeal. | User error; only the claimant can appeal. |
| `InvalidResolver: resolver cannot be the claimant` | dispute::open_dispute / appeal_dispute | The selected resolver matched the claimant. | User error; choose a different resolver. |
| `InvalidResolver: resolver cannot be the depositor` | dispute::open_dispute | The selected resolver matched the depositor. | User error; choose a different resolver. |
| `InvalidResolver: resolver cannot be the beneficiary` | dispute::open_dispute | The selected resolver matched the beneficiary. | User error; choose a different resolver. |
| `Unauthorized: only escrow parties can open a dispute` | dispute::open_dispute | The claimant was neither the depositor nor beneficiary. | User error; only escrow parties can file disputes. |
| `AlreadySettled: escrow is already settled` | dispute::settle_escrow_by_outcome | The escrow was already released or refunded. | User error; the escrow state is already finalized. |
| `SupplyCap: max supply reached` | balance minting | The token supply cap was reached. | System error or user error depending on policy; likely requires a supply policy change. |
| `supply cannot be negative` | balance minting | An invalid negative mint or burn operation was attempted. | User error; reject the request. |
| `invalid nonce` | permit | The permit nonce was invalid. | User error; request a fresh permit. |
| `allowance is expired` | allowance | The allowance had expired. | User error; re-approve with a valid expiry. |
| `insufficient allowance` | allowance | The attempted spend exceeded the allowance. | User error; request a higher allowance. |
| `ContractPaused: all transfers are currently paused` | pause | Transfers were attempted while the contract was paused. | Retry later or surface as a service outage. |
| `InvalidFreeze: cannot freeze the admin address` | freeze | An attempt was made to freeze the admin address. | User error; pick a different account. |
| `AlreadyFrozen: account is already frozen` | freeze | Freeze was attempted on an already-frozen account. | User error; no-op or ignore. |
| `NotFrozen: account is not frozen` | freeze | Unfreeze was attempted on a non-frozen account. | User error; no-op or ignore. |
| `InvalidInterval: interval must be at least 1` | recurring | The recurring interval was zero or invalid. | User error; use a positive interval. |
| `InvalidRecurring: payer and payee cannot be the same address` | recurring | The payer and payee were the same address. | User error; choose different addresses. |
| `recurring payment is not active` | recurring | The recurring payment was inactive. | User error; activate the recurring payment first. |
| `recurring payment is paused` | recurring | The payment was paused. | Retry later or resume it. |
| `interval has not elapsed` | recurring | The recurring payment interval had not yet elapsed. | Retry later. |
| `InsufficientBalance: payer has insufficient balance` | recurring | The payer lacked enough balance for the charge. | Retry after funding or surface as a funding issue. |
| `recipients list cannot be empty` | splitter | The split had no recipients. | User error; provide at least one recipient. |
| `TooManyRecipients: maximum 20 recipients allowed` | splitter | The splitter exceeded the recipient limit. | User error; reduce the recipient count. |
| `recipient share_bps cannot be zero` | splitter | A recipient share had zero basis points. | User error; provide a positive share. |
| `duplicate recipient address` | splitter | The split included the same address twice. | User error; deduplicate recipients. |
| `InvalidShares: recipient shares must sum to exactly 10000 bps` | splitter | The recipient weights did not sum to 10000. | User error; correct the allocation. |
| `BulkLimit: maximum 10 split IDs per batch` | splitter | A batch exceeded the limit. | User error; split the request into smaller batches. |
| `MemoTooLong: memo cannot exceed 64 bytes` | escrow / splitter | A memo exceeded the configured size. | User error; shorten the memo. |

## Escrow

| Panic string | Module / function(s) | Cause | Caller handling |
| --- | --- | --- | --- |
| `InvalidEscrow: depositor and beneficiary cannot be the same address` | escrow::create_escrow | The depositor and beneficiary were identical. | User error; choose different parties. |
| `DisputeOpen: cannot refund while an active dispute is pending resolution` | escrow::try_refund_escrow | An open dispute existed for the escrow. | User error; the dispute must resolve or expire before refunding. |
| `escrow already settled` | escrow::topup_escrow / admin settlement | The escrow had already been released or refunded. | User error; no further mutation is allowed. |
| `not the depositor` | escrow::topup_escrow | The caller was not the original depositor. | User error; only the depositor may top up. |
| `already settled` | escrow::admin_settle_escrow | The escrow was already settled. | User error; the state is finalized. |

## Dispute

| Panic string | Module / function(s) | Cause | Caller handling |
| --- | --- | --- | --- |
| `NotExpired: expiry ledger has not been reached` | dispute::expire_dispute | The current ledger was before the dispute expiry ledger. | Retry once the expiry ledger is reached. |

## Splitter

| Panic string | Module / function(s) | Cause | Caller handling |
| --- | --- | --- | --- |
| `recipients cannot be empty` | splitter::create_split | An empty recipient list was passed. | User error; provide recipients. |
| `unauthorized` | splitter | The caller was not authorized for the requested mutation. | User error; use the correct caller. |
| `already distributed` | splitter | The split was already distributed. | User error; do not repeat the action. |
| `split cancelled` | splitter | The split was cancelled. | User error; the split is no longer active. |
| `already cancelled` | splitter | The split was already cancelled. | User error; no-op. |

## Recurring

| Panic string | Module / function(s) | Cause | Caller handling |
| --- | --- | --- | --- |
| `batch exceeds maximum of 20` | recurring batch helpers | A batch request exceeded the limit. | User error; split the batch. |

## Admin

| Panic string | Module / function(s) | Cause | Caller handling |
| --- | --- | --- | --- |
| `not authorized: caller is not the admin` | admin | The caller was not the admin. | User error; use an admin account. |

## Permit

| Panic string | Module / function(s) | Cause | Caller handling |
| --- | --- | --- | --- |
| `invalid nonce` | permit | The permit nonce failed validation. | User error; request a new permit. |

## Snapshot

| Panic string | Module / function(s) | Cause | Caller handling |
| --- | --- | --- | --- |
| `snapshot not found` | snapshot::get_snapshot_balance / get_snapshot_ledger | A requested snapshot ID did not exist. | User error; use a valid snapshot ID. |

## Dividend

| Panic string | Module / function(s) | Cause | Caller handling |
| --- | --- | --- | --- |
| `no holders to distribute to` | dividend | The dividend distribution had no holders. | User error or configuration issue. |
| `insufficient dividend pool` | dividend | The dividend pool did not have sufficient funds. | User error or funding issue. |
