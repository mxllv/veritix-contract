use soroban_sdk::{contracttype, Address};

// ── TTL constants ────────────────────────────────────────────────────────────
// Balances: ~1 year (6_310_000 ledgers at ~5s/ledger)
pub const BALANCE_LIFETIME_THRESHOLD: u32 = 6_310_000;
// Escrow records: ~1 year
pub const ESCROW_LIFETIME_THRESHOLD: u32 = 7_884_000;
// Dispute records: ~6 months from opening
pub const DISPUTE_LIFETIME_THRESHOLD: u32 = 3_942_000;
// Recurring records: ~1 year from last execution
pub const RECURRING_LIFETIME_THRESHOLD: u32 = 6_310_000;
// Split records: ~90 days from creation
pub const SPLIT_LIFETIME_THRESHOLD: u32 = 1_555_200;

// ── Memo constants ───────────────────────────────────────────────────────────
pub const MAX_MEMO_BYTES: u32 = 64;

#[contracttype]
#[derive(Clone)]
pub enum DataKey {
    Admin,
    AdminActiveAfter,
    ProposedAdmin,
    EscrowCount,
    EscrowValueLocked,
    Escrow(u32),
    DepositorEscrows(Address),
    BeneficiaryEscrows(Address),
    MultiEscrowCount,
    MultiEscrow(u32),
    RecurringHistory(u32),
    ClaimantDisputes(Address),
    LastEscrowTime(Address),
    Allowance(Address, Address),
    Frozen(Address),
    EscrowDispute(u32),
    TotalSupply,
    RecurringCount,
    Recurring(u32),
    Arbiter,
    ResolverStats(Address),
    FeeBps,
    TreasuryAddress,
    TotalFeesCollected,
    SplitCount,
    Split(u32),
    AutoRelease(u32),
    DisputeCount(u32),
    MaxDisputes(u32),
    Vesting(u32),
    VestingCount,
    HolderVestings(Address),
    HolderSet,
    HolderCount,
    Nonce(Address),
    PayeeRecurrings(Address),
    MediationFeeBps,
    Holders,
    BalanceOf(Address),
    Authorized(Address),
    AllowanceSpenders(Address),
    WhitelistEnabled,
    Whitelisted(Address),
    Version,
    MaxSupply,
    Paused,
}

#[contracttype]
#[derive(Clone, Debug, PartialEq)]
pub struct RecurringPayment {
    pub recurring_id: u32,
    pub execution_ledger: u32,
    pub amount: i128,
}

// #571: Vesting record for locked token schedules
#[contracttype]
#[derive(Clone, Debug, PartialEq)]
pub struct VestingRecord {
    pub id: u32,
    pub holder: Address,
    pub token: Address,
    pub amount: i128,
    pub vesting_ledger: u32,
    pub claimed: bool,
}


#[contracttype]
#[derive(Clone, Debug, PartialEq)]
pub struct ResolverStats {
    pub resolver: Address,
    pub total_resolved: u32,
    pub for_beneficiary: u32,
    pub for_depositor: u32,
}