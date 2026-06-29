//! Recurring payment module.
//! Supports schedule setup plus permissionless "crank" execution when intervals elapse.
//!
//! # Authorization Model
//! - `setup_recurring` requires authorization from both the payer and payee.
//! - `execute_recurring` is permissionless (anyone can trigger) but pulls funds from
//!   the payer's balance at execution time, not at setup time. This is a "pull" model.
//! - No funds are locked in the contract during setup; the payer's balance remains
//!   in their account until each execution. Canceling a recurring payment therefore
//!   does not require a refund since no funds were ever transferred to the contract.

use crate::balance::{receive_balance, spend_balance};
use crate::storage_types::{increment_counter, DataKey, PERSISTENT_BUMP_AMOUNT, PERSISTENT_LIFETIME_THRESHOLD};
use crate::validation::require_positive_amount;
use soroban_sdk::{contracttype, symbol_short, vec, Address, Env, Vec};

#[contracttype]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct RecurringRecord {
    pub id: u32,
    pub payer: Address,
    pub payee: Address,
    pub amount: i128,
    pub interval: u32,
    pub last_charged_ledger: u32,
    pub active: bool,
    pub paused: bool,
}

fn append_payer_index(e: &Env, payer: &Address, id: u32) {
    let key = DataKey::PayerRecurrings(payer.clone());
    let mut ids: Vec<u32> = e.storage().persistent().get(&key).unwrap_or_else(|| vec![e]);
    ids.push_back(id);
    e.storage().persistent().set(&key, &ids);
    e.storage().persistent().extend_ttl(&key, PERSISTENT_LIFETIME_THRESHOLD, PERSISTENT_BUMP_AMOUNT);
}

pub fn setup_recurring(e: &Env, payer: Address, payee: Address, amount: i128, interval: u32) -> u32 {
    require_positive_amount(amount);
    if interval == 0 { panic!("InvalidInterval: interval must be at least 1"); }
    if payer == payee { panic!("InvalidRecurring: payer and payee cannot be the same address"); }
    payer.require_auth();
    payee.require_auth();
    let count = increment_counter(e, &DataKey::RecurringCount);
    let record = RecurringRecord {
        id: count, payer: payer.clone(), payee: payee.clone(),
        amount, interval, last_charged_ledger: e.ledger().sequence(),
        active: true, paused: false,
    };
    let key = DataKey::Recurring(count);
    e.storage().persistent().set(&key, &record);
    e.storage().persistent().extend_ttl(&key, PERSISTENT_LIFETIME_THRESHOLD, PERSISTENT_BUMP_AMOUNT);
    append_payer_index(e, &payer, count);
    e.events().publish((symbol_short!("recur_stp"), payer.clone()), (payee, amount));
    count
}

pub fn execute_recurring(e: &Env, recurring_id: u32) {
    let key = DataKey::Recurring(recurring_id);
    let mut record: RecurringRecord = e.storage().persistent().get(&key)
        .unwrap_or_else(|| panic!("recurring record not found"));
    e.storage().persistent().extend_ttl(&key, PERSISTENT_LIFETIME_THRESHOLD, PERSISTENT_BUMP_AMOUNT);
    if !record.active { panic!("recurring payment is not active"); }
    if record.paused { panic!("recurring payment is paused"); }
    let current_ledger = e.ledger().sequence();
    if current_ledger < record.last_charged_ledger + record.interval { panic!("interval has not elapsed"); }
    if crate::balance::read_balance(e, record.payer.clone()) < record.amount {
        panic!("InsufficientBalance: payer has insufficient balance");
    }
    spend_balance(e, record.payer.clone(), record.amount);
    receive_balance(e, record.payee.clone(), record.amount);
    record.last_charged_ledger = current_ledger;
    e.storage().persistent().set(&key, &record);
    e.storage().persistent().extend_ttl(&key, PERSISTENT_LIFETIME_THRESHOLD, PERSISTENT_BUMP_AMOUNT);
    e.events().publish((symbol_short!("recur_exe"), recurring_id), record.amount);
}

pub fn cancel_recurring(e: &Env, caller: Address, recurring_id: u32) {
    caller.require_auth();
    let key = DataKey::Recurring(recurring_id);
    let mut record: RecurringRecord = e.storage().persistent().get(&key)
        .unwrap_or_else(|| panic!("recurring record not found"));
    if record.payer != caller { panic!("unauthorized"); }
    record.active = false;
    e.storage().persistent().set(&key, &record);
    e.storage().persistent().extend_ttl(&key, PERSISTENT_LIFETIME_THRESHOLD, PERSISTENT_BUMP_AMOUNT);
    e.events().publish((symbol_short!("recur_cxl"), recurring_id, caller.clone()), ());
}

pub fn pause_recurring(e: &Env, caller: Address, recurring_id: u32) {
    caller.require_auth();
    let key = DataKey::Recurring(recurring_id);
    let mut record: RecurringRecord = e.storage().persistent().get(&key)
        .unwrap_or_else(|| panic!("recurring record not found"));
    if record.payer != caller { panic!("unauthorized"); }
    record.paused = true;
    e.storage().persistent().set(&key, &record);
    e.storage().persistent().extend_ttl(&key, PERSISTENT_LIFETIME_THRESHOLD, PERSISTENT_BUMP_AMOUNT);
    e.events().publish((symbol_short!("recur_psd"), recurring_id), ());
}

pub fn resume_recurring(e: &Env, caller: Address, recurring_id: u32) {
    caller.require_auth();
    let key = DataKey::Recurring(recurring_id);
    let mut record: RecurringRecord = e.storage().persistent().get(&key)
        .unwrap_or_else(|| panic!("recurring record not found"));
    if record.payer != caller { panic!("unauthorized"); }
    record.paused = false;
    e.storage().persistent().set(&key, &record);
    e.storage().persistent().extend_ttl(&key, PERSISTENT_LIFETIME_THRESHOLD, PERSISTENT_BUMP_AMOUNT);
    e.events().publish((symbol_short!("recur_rmd"), recurring_id), ());
}

pub fn amend_recurring(e: &Env, caller: Address, recurring_id: u32, new_amount: Option<i128>, new_interval: Option<u32>) {
    caller.require_auth();
    let key = DataKey::Recurring(recurring_id);
    let mut record: RecurringRecord = e.storage().persistent().get(&key)
        .unwrap_or_else(|| panic!("recurring record not found"));
    if record.payer != caller { panic!("unauthorized"); }
    if !record.active { panic!("recurring payment is not active"); }
    if let Some(amt) = new_amount { require_positive_amount(amt); record.amount = amt; }
    if let Some(ivl) = new_interval { if ivl == 0 { panic!("interval must be >= 1"); } record.interval = ivl; }
    e.storage().persistent().set(&key, &record);
    e.storage().persistent().extend_ttl(&key, PERSISTENT_LIFETIME_THRESHOLD, PERSISTENT_BUMP_AMOUNT);
    e.events().publish((symbol_short!("recur_amd"), recurring_id), (record.amount, record.interval));
}

pub fn get_recurring(e: &Env, recurring_id: u32) -> RecurringRecord {
    let key = DataKey::Recurring(recurring_id);
    let record = e.storage().persistent().get(&key)
        .unwrap_or_else(|| panic!("recurring record not found"));
    e.storage().persistent().extend_ttl(&key, PERSISTENT_LIFETIME_THRESHOLD, PERSISTENT_BUMP_AMOUNT);
    record
}

pub fn get_recurring_by_payer(e: &Env, payer: Address) -> Vec<u32> {
    let key = DataKey::PayerRecurrings(payer);
    e.storage().persistent().get(&key).unwrap_or_else(|| vec![e])
}

/// Batch-cancel up to 20 recurring payments belonging to payer.
/// Atomically fails (panics) if any ID is not owned by payer or exceeds the 20-limit.
pub fn cancel_recurring_batch(e: &Env, payer: Address, recurring_ids: Vec<u32>) {
    payer.require_auth();
    if recurring_ids.len() > 20 {
        panic!("batch exceeds maximum of 20");
    }
    // First pass: validate all ownership before mutating any state.
    for id in recurring_ids.iter() {
        let key = DataKey::Recurring(id);
        let record: RecurringRecord = e
            .storage()
            .persistent()
            .get(&key)
            .unwrap_or_else(|| panic!("recurring record not found"));
        if record.payer != payer {
            panic!("unauthorized");
        }
    }
    // Second pass: apply cancellations.
    let count = recurring_ids.len() as u32;
    for id in recurring_ids.iter() {
        let key = DataKey::Recurring(id);
        let mut record: RecurringRecord = e.storage().persistent().get(&key).unwrap();
        record.active = false;
        e.storage().persistent().set(&key, &record);
        e.storage()
            .persistent()
            .extend_ttl(&key, PERSISTENT_LIFETIME_THRESHOLD, PERSISTENT_BUMP_AMOUNT);
    }
    e.events()
        .publish((symbol_short!("rcur_batch_c"), payer), count);
}
