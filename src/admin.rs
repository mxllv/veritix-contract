use soroban_sdk::{Address, Env};
use crate::storage_types::DataKey;

pub fn check_admin(e: &Env, caller: &Address) {
    let admin: Address = e.storage().persistent().get(&DataKey::Admin).expect("admin not set");
    if admin != *caller {
        panic!("Unauthorized: {} is not the contract admin", caller.to_string());
    }
    caller.require_auth();
}

pub fn is_initialized(e: &Env) -> bool {
    e.storage().persistent().has(&DataKey::Admin)
}

pub fn require_initialized(e: &Env) {
    if !is_initialized(e) {
        panic!("NotInitialized: call initialize first");
    }
}
