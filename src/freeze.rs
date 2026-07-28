use soroban_sdk::{Address, Env};
use crate::storage_types::DataKey;
use crate::admin;

pub fn freeze_account(env: &Env, admin: &Address, account_id: &Address) {
    admin::check_admin(env, admin);
    let stored_admin: Address = env.storage().persistent().get(&DataKey::Admin).expect("admin not set");
    if account_id == &stored_admin {
        panic!("InvalidFreeze: cannot freeze the admin address");
    }
    let is_frozen: bool = env.storage().persistent().get(&DataKey::Frozen(account_id.clone())).unwrap_or(false);
    if is_frozen {
        panic!("AlreadyFrozen: account is already frozen");
    }
    env.storage().persistent().set(&DataKey::Frozen(account_id.clone()), &true);
}

pub fn unfreeze_account(env: &Env, _admin: &Address, account_id: &Address) {
    let is_frozen: bool = env.storage().persistent().get(&DataKey::Frozen(account_id.clone())).unwrap_or(false);
    if !is_frozen {
        panic!("NotFrozen: account is not frozen");
    }
    env.storage().persistent().remove(&DataKey::Frozen(account_id.clone()));
}

pub fn is_frozen(env: &Env, account_id: &Address) -> bool {
    env.storage().persistent().get(&DataKey::Frozen(account_id.clone())).unwrap_or(false)
}


fn setup() -> (Env, VeriTixPayClient<'static>, Address, Address, Address, Address) {
    let e = Env::default();
    e.mock_all_auths();
    let contract_id = e.register_contract(None, VeriTixPay);
    let client = VeriTixPayClient::new(&e, &contract_id);

    let depositor = Address::generate(&e);
    let organiser = Address::generate(&e);
    let venue = Address::generate(&e);
    let token = e.register_stellar_asset_contract(depositor.clone());

    let token_admin = soroban_sdk::token::StellarAssetClient::new(&e, &token);
    token_admin.mint(&depositor, &100_000);

    (e, client, depositor, organiser, venue, token)
}

#[test]
fn test_create_multi_escrow_transfers_total() {
    let (e, client, depositor, organiser, venue, token) = setup();
    let expiry = e.ledger().sequence() + 1000;

    let recipients = vec![
        &e,
        (organiser.clone(), 700_i128),
        (venue.clone(), 300_i128),
    ];

    let id = client.create_multi_escrow(&depositor, &recipients, &token, &expiry);
    assert_eq!(id, 0);

    let token_client = soroban_sdk::token::Client::new(&e, &token);
    assert_eq!(token_client.balance(&depositor), 100_000 - 1000);
}


#[test]
fn test_recurring_history_grows() {
    let e = Env::default();
    e.mock_all_auths();

    let caller = soroban_sdk::Address::generate(&e);
    let recurring_id = 1;
    let amount = 500;
    
    record_recurring_execution(e.clone(), caller.clone(), recurring_id, amount);
    
    let history = get_recurring_history(e.clone(), recurring_id);
    assert_eq!(history.len(), 1);
    assert_eq!(history.get(0).unwrap().amount, amount);
    assert_eq!(history.get(0).unwrap().execution_ledger, e.ledger().sequence());
    
    // Simulate next execution
    e.ledger().with_mut(|l| l.sequence_number = e.ledger().sequence() + 10);
    record_recurring_execution(e.clone(), caller.clone(), recurring_id, amount);
    
    let history = get_recurring_history(e.clone(), recurring_id);
    assert_eq!(history.len(), 2);
    assert_eq!(history.get(1).unwrap().amount, amount);
    assert_eq!(history.get(1).unwrap().execution_ledger, e.ledger().sequence());
}

#[test]
#[should_panic(expected = "recurring is not active")]
fn test_max_executions_deactivates() {
    use soroban_sdk::{Address, token};
    use crate::recurring::{setup_recurring, execute_recurring};
    use crate::storage_types::DataKey;

    let e = Env::default();
    e.mock_all_auths();

    let payer = Address::generate(&e);
    let payee = Address::generate(&e);
    // Create a test token
    let token = e.register_stellar_asset_contract(Address::generate(&e));
    let token_client = token::Client::new(&e, &token);
    
    // Mint some tokens to the payer so transfers work
    token_client.mint(&payer, &1000);
    
    let amount = 100;
    let interval = 100; // 100 ledgers between executions
    let max_executions = 3;

    // Setup recurring payment with max 3 executions
    let recurring_id = setup_recurring(
        &e,
        payer.clone(),
        payee.clone(),
        token.clone(),
        amount,
        interval,
        max_executions,
    );
