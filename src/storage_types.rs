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
}

#[contracttype]
#[derive(Clone, Debug, PartialEq)]
pub struct RecurringPayment {
    pub recurring_id: u32,
    pub execution_ledger: u32,
    pub amount: i128,
}


#[test]
fn test_recurring_history_grows() {
    let e = Env::default();
    e.mock_all_auths();

    let caller = soroban_sdk::Address::generate(&e);
    let recurring_id = 1;
    let amount = 500;
    
    record_recurring_execution(e.clone(), caller.clone(), recurring_id, amount);
    
    let history = get_recurring_history(e.clone(), recurring_id);
    assert_eq!(history.len(), 1);
    assert_eq!(history.get(0).unwrap().amount, amount);
    assert_eq!(history.get(0).unwrap().execution_ledger, e.ledger().sequence());
    
    // Simulate next execution
    e.ledger().with_mut(|l| l.sequence_number = e.ledger().sequence() + 10);
    record_recurring_execution(e.clone(), caller.clone(), recurring_id, amount);
    
    let history = get_recurring_history(e.clone(), recurring_id);
    assert_eq!(history.len(), 2);
    assert_eq!(history.get(1).unwrap().amount, amount);
    assert_eq!(history.get(1).unwrap().execution_ledger, e.ledger().sequence());
}

#[test]
#[should_panic(expected = "recurring is not active")]
fn test_max_executions_deactivates() {
    use soroban_sdk::{Address, token};
    use crate::recurring::{setup_recurring, execute_recurring};
    use crate::storage_types::DataKey;

    let e = Env::default();
    e.mock_all_auths();

    let payer = Address::generate(&e);
    let payee = Address::generate(&e);
    // Create a test token
    let token = e.register_stellar_asset_contract(Address::generate(&e));
    let token_client = token::Client::new(&e, &token);
    
    // Mint some tokens to the payer so transfers work
    token_client.mint(&payer, &1000);
    
    let amount = 100;
    let interval = 100; // 100 ledgers between executions
    let max_executions = 3;

    // Setup recurring payment with max 3 executions
    let recurring_id = setup_recurring(
        &e,
        payer.clone(),
        payee.clone(),
        token.clone(),
        amount,
        interval,
        max_executions,
    );

#[contracttype]
#[derive(Clone, Debug, PartialEq)]
pub struct ResolverStats {
    pub resolver: Address,
    pub total_resolved: u32,
    pub for_beneficiary: u32,
    pub for_depositor: u32,
}