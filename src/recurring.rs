use soroban_sdk::{contracttype, token, Address, Env, Vec};
use crate::storage_types::{DataKey, RecurringPayment};

#[contracttype]
#[derive(Clone)]
pub struct RecurringRecord {
    pub payer: Address,
    pub payee: Address,
    pub token: Address,
    pub amount: i128,
    pub interval: u32,
    pub last_charged_ledger: u32,
    pub active: bool,
    pub max_executions: u32,
    pub execution_count: u32,
}

fn track_payee_recurring(e: &Env, payee: &Address, recurring_id: u32) {
    let mut list: soroban_sdk::Vec<u32> = e.storage().persistent()
        .get(&DataKey::PayeeRecurrings(payee.clone()))
        .unwrap_or(soroban_sdk::Vec::new(e));
    list.push_back(recurring_id);
    e.storage().persistent().set(&DataKey::PayeeRecurrings(payee.clone()), &list);
}

pub fn setup_recurring(
    e: &Env,
    payer: Address,
    payee: Address,
    token_addr: Address,
    amount: i128,
    interval: u32,
    max_executions: u32,
) -> u32 {
    // #426: amount must be positive — first check
    assert!(amount > 0, "amount must be positive");
    payer.require_auth();

    let id: u32 = e
        .storage()
        .persistent()
        .get(&DataKey::RecurringCount)
        .unwrap_or(0);
    let record = RecurringRecord {
        payer,
        payee,
        token: token_addr,
        amount,
        interval,
        last_charged_ledger: e.ledger().sequence(),
        active: true,
        max_executions,
        execution_count: 0,
    };
    e.storage()
        .persistent()
        .set(&DataKey::Recurring(id), &record);
    track_payee_recurring(e, &record.payee, id);
    e.storage()
        .persistent()
        .set(&DataKey::RecurringCount, &(id + 1));
    id
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

pub fn execute_recurring(e: &Env, recurring_id: u32) {
    let mut record: RecurringRecord = e
        .storage()
        .persistent()
        .get(&DataKey::Recurring(recurring_id))
        .expect("recurring not found");

    // Check if the record is active before proceeding
    assert!(record.active, "recurring is not active");

    // #435: due-date check is FIRST — cheapest possible early exit for griefing protection
    let next_due = record
        .last_charged_ledger
        .checked_add(record.interval)
        .expect("overflow");
    assert!(e.ledger().sequence() >= next_due, "not yet due");

    let token_client = token::Client::new(e, &record.token);
    token_client.transfer(&record.payer, &record.payee, &record.amount);

    // Anchor schedule to original baseline (not current ledger — prevents drift)
    record.last_charged_ledger = next_due;
    // Increment execution count after successful transfer
    record.execution_count += 1;
    // If we've reached max executions, deactivate the record
    if record.max_executions > 0 && record.execution_count >= record.max_executions {
        record.active = false;
    }
    e.storage()
        .persistent()
        .set(&DataKey::Recurring(recurring_id), &record);
}

pub fn record_recurring_execution(e: Env, caller: Address, recurring_id: u32, amount: i128) {
    caller.require_auth();
    let mut history = get_recurring_history(e.clone(), recurring_id);
    history.push_back(RecurringPayment {
        recurring_id,
        execution_ledger: e.ledger().sequence(),
        amount,
    });
    e.storage()
        .persistent()
        .set(&DataKey::RecurringHistory(recurring_id), &history);
}

pub fn get_recurring_history(e: Env, recurring_id: u32) -> Vec<RecurringPayment> {
    e.storage()
        .persistent()
        .get(&DataKey::RecurringHistory(recurring_id))
        .unwrap_or(Vec::new(&e))
}

   let token_client = soroban_sdk::token::Client::new(&e, &record.token);
    token_client.transfer(
        &e.current_contract_address(),
        &record.depositor,
        &record.total_amount,
    );

pub fn amend_recurring(e: &Env, caller: &Address, recurring_id: u32, new_amount: i128, new_interval: u32) {
    caller.require_auth();
    assert!(new_amount > 0, "amount must be positive");
    assert!(new_interval > 0, "interval must be positive");
    let mut record: RecurringRecord = e.storage().persistent()
        .get(&DataKey::Recurring(recurring_id))
        .expect("recurring not found");
    assert!(record.payer == *caller, "not the payer");
    assert!(record.active, "recurring is not active");
    record.amount = new_amount;
    record.interval = new_interval;
    e.storage().persistent().set(&DataKey::Recurring(recurring_id), &record);
}

pub fn recurring_count_for_payee(e: Env, payee: Address) -> u32 {
    let list: soroban_sdk::Vec<u32> = e.storage().persistent()
        .get(&DataKey::PayeeRecurrings(payee))
        .unwrap_or(soroban_sdk::Vec::new(&e));
    list.len()
}

pub fn recurring_ids_for_payee(e: Env, payee: Address) -> soroban_sdk::Vec<u32> {
    e.storage().persistent()
        .get(&DataKey::PayeeRecurrings(payee))
        .unwrap_or(soroban_sdk::Vec::new(&e))
}
