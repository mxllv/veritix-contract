use super::*;
use crate::balance::receive_balance;
use crate::balance::spend_balance;
use soroban_sdk::{testutils::Address as _, Address, Env};

#[test]
fn test_freeze_stores_true_in_persistent_storage() {
    let env = Env::default();
    let admin = Address::generate(&env);
    let target = Address::generate(&env);

    freeze_account(&env, admin, target.clone());

    assert_eq!(is_frozen(&env, &target), true);
}

#[test]
fn test_is_frozen_returns_false_for_unfrozen_address() {
    let env = Env::default();
    let target = Address::generate(&env);

    assert_eq!(is_frozen(&env, &target), false);
}

#[test]
fn test_unfreeze_removes_storage_entry() {
    let env = Env::default();
    let admin = Address::generate(&env);
    let target = Address::generate(&env);

    freeze_account(&env, admin.clone(), target.clone());
    assert_eq!(is_frozen(&env, &target), true);

    unfreeze_account(&env, admin, target.clone());
    assert_eq!(is_frozen(&env, &target), false);
}

#[test]
#[should_panic(expected = "AlreadyFrozen")]
fn test_freeze_already_frozen_panics() {
    let env = Env::default();
    let admin = Address::generate(&env);
    let target = Address::generate(&env);

    freeze_account(&env, admin.clone(), target.clone());
    freeze_account(&env, admin, target);
}

#[test]
#[should_panic(expected = "NotFrozen")]
fn test_unfreeze_not_frozen_panics() {
    let env = Env::default();
    let admin = Address::generate(&env);
    let target = Address::generate(&env);

    unfreeze_account(&env, admin, target);
}

#[test]
#[should_panic(expected = "InvalidFreeze")]
fn test_freeze_admin_address_panics() {
    let env = Env::default();
    let admin = Address::generate(&env);

    // Store admin in persistent storage so the guard can read it
    env.storage().persistent().set(&crate::storage_types::DataKey::Admin, &admin);
    freeze_account(&env, admin.clone(), admin);
}

#[test]
#[should_panic]
fn test_frozen_account_cannot_spend_balance() {
    let env = Env::default();
    let target = Address::generate(&env);
    let admin = Address::generate(&env);

    freeze_account(&env, admin, target.clone());
    spend_balance(&env, target, 100);
}

#[test]
fn test_frozen_account_can_receive_balance() {
    let env = Env::default();
    let target = Address::generate(&env);
    let admin = Address::generate(&env);

    freeze_account(&env, admin, target.clone());
    receive_balance(&env, target, 100);
}

#[test]
#[should_panic]
fn test_freeze_requires_admin_auth() {
    let env = Env::default();
    let target = Address::generate(&env);

    freeze_account(&env, target.clone(), target);
}


#[test]
#[should_panic(expected = "NotFrozen")]
fn test_unfreeze_not_frozen_panics() {
    let env = Env::default();
    let admin = Address::generate(&env);
    let target = Address::generate(&env);

    unfreeze_account(&env, admin, target);
}

#[test]
#[should_panic(expected = "InvalidFreeze")]
fn test_freeze_admin_address_panics() {
    let env = Env::default();
    let admin = Address::generate(&env);

    // Store admin in persistent storage so the guard can read it
    env.storage().persistent().set(&crate::storage_types::DataKey::Admin, &admin);
    freeze_account(&env, admin.clone(), admin);
}

#[test]
#[should_panic]
fn test_frozen_account_cannot_spend_balance() {
    let env = Env::default();
    let target = Address::generate(&env);
    let admin = Address::generate(&env);

    freeze_account(&env, admin, target.clone());
    spend_balance(&env, target, 100);
}
use crate::balance::{
        decrease_supply, increase_supply, read_balance, read_total_supply, receive_balance,
        spend_balance,
    };
    use crate::contract::VeritixToken;

    fn setup_env() -> (Env, Address) {
        let e = Env::default();
        e.mock_all_auths();
        let contract_id = e.register_contract(None, VeritixToken);
        (e, contract_id)
    }

    #[test]
    fn test_read_balance_returns_zero_for_unknown_address() {
        let (e, contract_id) = setup_env();
        let addr = Address::generate(&e);
        e.as_contract(&contract_id, || {
            assert_eq!(read_balance(&e, addr), 0);
        });
    }

    #[test]
    fn test_receive_balance_sets_and_reads_correctly() {
        let (e, contract_id) = setup_env();
        let addr = Address::generate(&e);
        e.as_contract(&contract_id, || {
            receive_balance(&e, addr.clone(), 500);
            assert_eq!(read_balance(&e, addr), 500);
        });
    }

    #[test]
    fn test_spend_balance_decrements_correctly() {
        let (e, contract_id) = setup_env();
        let addr = Address::generate(&e);
        e.as_contract(&contract_id, || {
            receive_balance(&e, addr.clone(), 1_000);
            spend_balance(&e, addr.clone(), 400);
            assert_eq!(read_balance(&e, addr), 600);
        });
    }