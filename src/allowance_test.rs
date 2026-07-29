#![cfg(test)]

use soroban_sdk::{testutils::{Address as _, Ledger as _}, Address, Env};
use crate::test::create_token_contract;

#[test]
fn test_allowance_valid_at_expiry_ledger() {
    let e = Env::default();
    e.mock_all_auths();

    let admin = Address::generate(&e);
    let token = create_token_contract(&e, &admin);
    let tc = soroban_sdk::token::Client::new(&e, &token);
    let from = Address::generate(&e);
    let spender = Address::generate(&e);
    let to = Address::generate(&e);

    let stellar = soroban_sdk::token::StellarAssetClient::new(&e, &token);
    stellar.mint(&from, &5000);

    let expiry = e.ledger().sequence() + 10;
    tc.approve(&from, &spender, &1000, &expiry);

    e.ledger().with_mut(|li| li.sequence_number = expiry);

    tc.transfer_from(&spender, &from, &to, &500);
    assert_eq!(tc.balance(&to), 500);
    assert_eq!(tc.balance(&from), 4500);
}

#[test]
fn test_allowance_still_valid_one_before_expiry() {
    let e = Env::default();
    e.mock_all_auths();

    let admin = Address::generate(&e);
    let token = create_token_contract(&e, &admin);
    let tc = soroban_sdk::token::Client::new(&e, &token);
    let from = Address::generate(&e);
    let spender = Address::generate(&e);
    let to = Address::generate(&e);

    let stellar = soroban_sdk::token::StellarAssetClient::new(&e, &token);
    stellar.mint(&from, &5000);

    let expiry = e.ledger().sequence() + 10;
    tc.approve(&from, &spender, &1000, &expiry);

    e.ledger().with_mut(|li| li.sequence_number = expiry - 1);

    tc.transfer_from(&spender, &from, &to, &500);
    assert_eq!(tc.balance(&to), 500);
}

pub fn check_admin(e: &Env, caller: &Address) {
    let admin: Address = e
        .storage()
        .persistent()
        .get(&DataKey::Admin)
        .expect("admin not set");

    if admin != *caller {
        panic!("Unauthorized: caller is not the contract admin");
    }

    let admin_active_after: u32 = e
        .storage()
        .persistent()
        .get(&DataKey::AdminActiveAfter)
        .unwrap_or(0);

    if e.ledger().sequence() < admin_active_after {
        panic!(
            "AdminNotActive yet — new admin becomes active after ledger {}",
            admin_active_after
        );
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

// ── #451: One-step admin change with time-delay ──────────────────────────────

pub fn transfer_ownership(e: &Env, new_admin: &Address) {
    let current_admin: Address = e
        .storage()
        .persistent()
        .get(&DataKey::Admin)
        .expect("admin not set");
    current_admin.require_auth();

    e.storage()
        .persistent()
        .set(&DataKey::ProposedAdmin, new_admin);

    e.events().publish(
        (soroban_sdk::symbol_short!("ownership"),),
        (current_admin, new_admin.clone()),
    );
}
