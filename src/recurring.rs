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
    e.storage()
        .persistent()
        .set(&DataKey::RecurringCount, &(id + 1));
    id
}

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


    
pub fn get_escrow_age(e: Env, escrow_id: u32) -> u32 {
    let record = load_record(&e, escrow_id);
    if record.released || record.refunded {
        0
    } else {
        e.ledger().sequence().saturating_sub(record.created_at_ledger)
    }
}

pub fn topup_escrow(e: Env, depositor: Address, escrow_id: u32, amount: i128) {
    depositor.require_auth();
    assert!(amount > 0, "amount must be positive");
    if e.storage().persistent().has(&DataKey::EscrowDispute(escrow_id)) {
        panic!("DisputeOpen: cannot top up an escrow under active dispute");
    }
    let mut record = load_record(&e, escrow_id);
    assert!(!record.released && !record.refunded, "escrow already settled");
    assert!(record.depositor == depositor, "not the depositor");
    let token_client = token::Client::new(&e, &record.token);
    token_client.transfer(&depositor, &e.current_contract_address(), &amount);
    record.amount += amount;
    save_record(&e, &record);
}