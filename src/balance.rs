use soroban_sdk::Env;
use crate::storage_types::DataKey;

pub fn read_supply(e: &Env) -> i128 {
    e.storage().persistent().get(&DataKey::TotalSupply).unwrap_or(0)
}

pub fn increase_supply(e: &Env, amount: i128) {
    let supply = read_supply(e);
    let new_supply = supply.checked_add(amount).expect("supply overflow");
    e.storage().persistent().set(&DataKey::TotalSupply, &new_supply);
}

pub fn decrease_supply(e: &Env, amount: i128) {
    let supply = read_supply(e);
    let new_supply = supply.checked_sub(amount).expect("supply underflow");
    e.storage().persistent().set(&DataKey::TotalSupply, &new_supply);
}

use soroban_sdk::Address;

pub fn balance_of(e: &Env, account: &Address) -> i128 {
    e.storage().persistent().get(&DataKey::BalanceOf(account.clone())).unwrap_or(0)
}

pub fn spendable_balance(e: &Env, account: &Address) -> i128 {
    if is_frozen(e, account) {
        return 0;
    }
    balance_of(e, account)
}

pub fn is_frozen(e: &Env, account: &Address) -> bool {
    e.storage().persistent().get(&DataKey::Frozen(account.clone())).unwrap_or(false)
}

pub fn set_authorized(e: &Env, admin: &Address, account: &Address, authorized: bool) {
    crate::admin::check_admin(e, admin);
    if authorized {
        e.storage().persistent().remove(&DataKey::Frozen(account.clone()));
    } else {
        e.storage().persistent().set(&DataKey::Frozen(account.clone()), &true);
    }
}

pub fn burn_from(e: &Env, spender: &Address, from: &Address, amount: i128) {
    spender.require_auth();
    assert!(amount > 0, "amount must be positive");
    crate::allowance::spend_allowance(e, from, spender, amount);
    let balance = balance_of(e, from);
    assert!(balance >= amount, "insufficient balance");
    let new_balance = balance - amount;
    if new_balance == 0 {
        e.storage().persistent().remove(&DataKey::BalanceOf(from.clone()));
    } else {
        e.storage().persistent().set(&DataKey::BalanceOf(from.clone()), &new_balance);
    }
    decrease_supply(e, amount);
}

pub fn add_balance(e: &Env, account: &Address, amount: i128) {
    let bal = balance_of(e, account);
    e.storage().persistent().set(&DataKey::BalanceOf(account.clone()), &(bal + amount));
}
