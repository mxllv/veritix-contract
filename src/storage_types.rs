use soroban_sdk::{contracttype, Address};

/// Minimum escrow amount to prevent spam (1 XLM equivalent in token stroops)
pub const MIN_ESCROW_AMOUNT: i128 = 10_000_000;

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
    PayeeRecurrings(Address),
    MediationFeeBps,
    Holders,
    Version,
    BalanceOf(Address),
    Authorized(Address),
    AllowanceSpenders(Address),
    WhitelistEnabled,
    Whitelisted(Address),
    AutoRelease(u32),
    DisputeCount(u32),
    MaxDisputes(u32),
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