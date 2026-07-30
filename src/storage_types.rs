use soroban_sdk::{contracttype, Address};

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
    // #571: Vesting
    Vesting(u32),
    VestingCount,
    HolderVestings(Address),
    // #573: Airdrop
    HolderSet,
    HolderCount,
    // #574: Permit nonces
    Nonce(Address),
    // Upstream additions
    PayeeRecurrings(Address),
    MediationFeeBps,
    Holders,
    BalanceOf(Address),
    Authorized(Address),
    AllowanceSpenders(Address),
    WhitelistEnabled,
    Whitelisted(Address),
    Version,
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