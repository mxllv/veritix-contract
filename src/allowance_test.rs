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
