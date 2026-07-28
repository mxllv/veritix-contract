use soroban_sdk::{contracttype, Env, String};

use crate::storage_types::{bump_instance, DataKey};
use crate::validation::{require_decimal_within_max, require_nonempty_string};

pub const MAX_DECIMALS: u32 = 18;

#[derive(Clone)]
#[contracttype]
pub struct TokenMetadata {
    pub name: String,
    pub symbol: String,
    pub decimal: u32,
}

pub fn read_metadata(e: &Env) -> TokenMetadata {
    bump_instance(e);
    e.storage().instance().get(&DataKey::Metadata).unwrap()
}

pub fn write_metadata(e: &Env, metadata: TokenMetadata) {
    bump_instance(e);
    e.storage().instance().set(&DataKey::Metadata, &metadata);
}

pub fn validate_metadata(metadata: &TokenMetadata) {
    require_nonempty_string(&metadata.name, "name cannot be empty");
    require_nonempty_string(&metadata.symbol, "symbol cannot be empty");
    require_decimal_within_max(metadata.decimal, MAX_DECIMALS);
}

fn admin(env: &Env) -> Address {
    Address::generate(env)
}

fn recipient(env: &Env) -> Address {
    Address::generate(env)
}

#[test]
fn test_init_and_has_level_badge() {
    let env = Env::default();
    env.mock_all_auths();
    let admin = admin(&env);
    let game = recipient(&env);

    let contract_id = env.register_contract(None, StellarHuntsNft);
    let client = StellarHuntsNftClient::new(&env, &contract_id);

    client.init(
        &admin,
        &game,
        &String::from_str(&env, "ipfs://placeholder/"),
        &String::from_str(&env, "StellarHuntsBadge"),
        &String::from_str(&env, "SHB"),
    );

    // Initially no badges.
    let r = recipient(&env);
    assert!(!client.has_level_badge(&r, &crate::Levels::Easy));
}

pub fn read_decimal(e: &Env) -> u32 {
    read_metadata(e).decimal
}

pub fn read_name(e: &Env) -> String {
    read_metadata(e).name
}

pub fn read_symbol(e: &Env) -> String {
    read_metadata(e).symbol
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

pub fn update_metadata_fields(e: &Env, name: Option<String>, symbol: Option<String>) {
    let mut metadata = read_metadata(e);
    if let Some(n) = name {
        require_nonempty_string(&n, "name cannot be empty");
        metadata.name = n;
    }
    if let Some(s) = symbol {
        require_nonempty_string(&s, "symbol cannot be empty");
        metadata.symbol = s;
    }
    write_metadata(e, metadata);
}
