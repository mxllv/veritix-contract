use soroban_sdk::{contracttype, Address, Env};
use crate::storage_types::DataKey;

#[contracttype]
#[derive(Clone)]
pub struct Allowance {
    pub amount: i128,
    pub expiration_ledger: u32,
}

pub fn write_allowance(e: &Env, from: &Address, spender: &Address, amount: i128, expiration_ledger: u32) {
    let key = DataKey::Allowance(from.clone(), spender.clone());
    if amount == 0 {
        e.storage().persistent().remove(&key);
    } else {
        let allowance = Allowance { amount, expiration_ledger };
        e.storage().persistent().set(&key, &allowance);
    }
}

pub fn create_allowance(e: &Env, from: &Address, spender: &Address, amount: i128, expiration_ledger: u32) {
    track_spender(e, from, spender);
    write_allowance(e, from, spender, amount, expiration_ledger);
}

pub fn read_allowance(e: &Env, from: &Address, spender: &Address) -> Allowance {
    let key = DataKey::Allowance(from.clone(), spender.clone());
    e.storage().persistent().get(&key).unwrap_or(Allowance {
        amount: 0,
        expiration_ledger: 0,
    })
}

pub fn spend_allowance(e: &Env, from: &Address, spender: &Address, amount: i128) {
    let allowance = read_allowance(e, from, spender);
    if allowance.expiration_ledger < e.ledger().sequence() {
        panic!("allowance expired");
    }
    if allowance.amount < amount {
        panic!("insufficient allowance");
    }
    let new_amount = allowance.amount - amount;
    write_allowance(e, from, spender, new_amount, allowance.expiration_ledger);
}

pub fn revoke_all_allowances(e: &Env, from: &Address) {
    let spenders: soroban_sdk::Vec<Address> = e.storage().persistent()
        .get(&DataKey::AllowanceSpenders(from.clone()))
        .unwrap_or(soroban_sdk::Vec::new(e));
    for i in 0..spenders.len() {
        if let Some(spender) = spenders.get(i) {
            e.storage().persistent().remove(&DataKey::Allowance(from.clone(), spender));
        }
    }
    e.storage().persistent().remove(&DataKey::AllowanceSpenders(from.clone()));
}

pub fn track_spender(e: &Env, from: &Address, spender: &Address) {
    let mut spenders: soroban_sdk::Vec<Address> = e.storage().persistent()
        .get(&DataKey::AllowanceSpenders(from.clone()))
        .unwrap_or(soroban_sdk::Vec::new(e));
    let mut found = false;
    for i in 0..spenders.len() {
        if let Some(s) = spenders.get(i) {
            if s == *spender {
                found = true;
                break;
            }
        }
    }
    if !found {
        spenders.push_back(spender.clone());
        e.storage().persistent().set(&DataKey::AllowanceSpenders(from.clone()), &spenders);
    }
}
