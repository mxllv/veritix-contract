use crate::storage_types::DataKey;
use soroban_sdk::{Address, Env};

pub fn is_frozen(e: &Env, addr: &Address) -> bool {
    e.storage()
        .persistent()
        .get(&DataKey::Freeze(addr.clone()))
        .unwrap_or(false)
}

pub fn freeze_account(e: &Env, _admin: Address, target: Address) {
    e.storage()
        .persistent()
        .set(&DataKey::Freeze(target), &true);
}

pub fn unfreeze_account(e: &Env, _admin: Address, target: Address) {
    e.storage()
        .persistent()
        .remove(&DataKey::Freeze(target));
}
