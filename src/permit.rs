use soroban_sdk::{Address, Env};
use crate::storage_types::DataKey;

pub fn read_nonce(e: &Env, user: &Address) -> u32 {
    e.storage()
        .persistent()
        .get::<_, u32>(&DataKey::Nonce(user.clone()))
        .unwrap_or(0)
}

pub fn check_and_increment_nonce(e: &Env, user: &Address, expected_nonce: u32) {
    let current = read_nonce(e, user);
    if expected_nonce != current {
        panic!("InvalidNonce: expected {} but got {}", current, expected_nonce);
    }
    e.storage()
        .persistent()
        .set(&DataKey::Nonce(user.clone()), &(current + 1));
}
