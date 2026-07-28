# Storage Layout Reference

`DataKey` is defined in `veritixpay/contract/token/src/storage_types.rs`.

## Instance Storage Keys
- `Admin` -> `Address`
  - Written in `admin.rs` (`write_admin`).
  - Owner module: `admin`.
  - TTL: instance TTL via `bump_instance`.
- `Metadata` -> `TokenMetadata`
  - Written in `metadata.rs`.
  - Owner module: `metadata`.
  - TTL: instance TTL via `bump_instance`.
- `TotalSupply` -> `i128`
  - Written in `balance.rs`.
  - Owner module: `balance`.
  - TTL: instance TTL via `bump_instance`.
- `EscrowCount` / `SplitCount` / `RecurringCount` / `DisputeCount` -> `u32`
  - Written through `increment_counter`.
  - Owner modules: escrow/splitter/recurring/dispute.
  - TTL: instance TTL bumped centrally in `increment_counter`.

## Persistent Storage Keys
- `Balance(Address)` -> `i128`
  - Owner module: `balance`.
  - TTL: `BALANCE_*` constants.
- `Allowance(AllowanceDataKey)` -> `AllowanceValue`
  - Owner module: `allowance`.
  - TTL: `ALLOWANCE_*` constants.
- `SpenderAllowances(Address)` -> `Vec<Address>`
  - Owner module: `allowance`.
  - TTL: persistent TTL constants.
- `Freeze(Address)` -> `bool`
  - Owner module: `freeze`.
  - TTL: persistent TTL constants.
- `Escrow(u32)` -> `EscrowRecord`
  - Owner module: `escrow`.
  - TTL: `ESCROW_*` constants for read bump; persistent TTL on write.
- `Split(u32)` -> `SplitRecord`
  - Owner module: `splitter`.
  - TTL: `SPLIT_*` constants.
- `Recurring(u32)` -> `RecurringRecord`
  - Owner module: `recurring`.
  - TTL: `RECURRING_*` constants.
- `Dispute(u32)` -> `DisputeRecord`
  - Owner module: `dispute`.
  - TTL: `DISPUTE_*` constants.
- `EscrowDispute(u32)` -> `u32` (active dispute id)
  - Owner module: `dispute`.
  - TTL: persistent TTL constants.

## Additional DataKey Variants (added since the table above, closes #561)

The enum has grown to 35 variants; the newer ones aren't yet covered above.

| Variant | Tier | Owner module |
| --- | --- | --- |
| `MaxSupply` | instance | `metadata` |
| `HolderSet` | persistent | `balance` (indexes all holders) |
| `SnapshotCount` | instance | `snapshot` |
| `Snapshot(u32)` | persistent | `snapshot` |
| `PayerRecurrings(Address)` | persistent | `recurring` (index over `Recurring`) |
| `SplitCount` | instance | `splitter` |
| `Split(u32)` | persistent | `splitter` |
| `DisputeCount` | instance | `dispute` |
| `EscrowDisputeHistory(u32)` | persistent | `dispute` |
| `ResolverDisputes(Address)` | persistent | `dispute` (index) |
| `OpenDisputes` | persistent | `dispute` (index of active ids) |
| `FrozenAccounts` | persistent | `freeze` (index over `Freeze`) |
| `OwnerAllowances(Address)` | persistent | `allowance` (index) |
| `Paused` | instance | `pause` |
| `ClawbackCoSigner` | instance | `admin` |
| `PendingAdmin` | instance | `admin` |
| `ExpiryWarned(u32)` | persistent | `escrow` |
| `Nonce(Address)` | persistent | `permit` |
| `DistributedCount` | instance | `batch`/`dividend` |
| `CancelledCount` | instance | `batch`/`dividend` |
| `TotalDistributedValue` | instance | `batch`/`dividend` |

## TTL Policy Summary
- Instance keys: bumped via `bump_instance` and counter mutation helper.
- Persistent keys: bumped on read/write using module-specific constants or shared persistent defaults.
- Long-horizon escrow records use year-scale constants to reduce expiry risk.
