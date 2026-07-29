use soroban_sdk::{Address, Env, token};
use crate::contract::{VeriTixPay, VeriTixPayClient};
use crate::storage_types::DataKey;

pub fn create_token_contract(e: &Env, admin: &Address) -> Address {
    e.register_stellar_asset_contract(admin.clone())
}

#[test]
fn test_emergency_withdraw() {
    let e = Env::default();
    e.mock_all_auths();

    // Setup contract and admin
    let contract_id = e.register_contract(None, VeriTixPay);
    let client = VeriTixPayClient::new(&e, &contract_id);
    
    let admin = Address::generate(&e);
    client.initialize(&admin);

    // Create token and mint some tokens
    let token = create_token_contract(&e, &admin);
    let token_admin_client = token::StellarAssetClient::new(&e, &token);
    let token_client = token::Client::new(&e, &token);
    
    // Mint 1000 tokens directly to the contract (stranded funds)
    token_admin_client.mint(&contract_id, &1000);
    
    // Create a recipient to receive the withdrawn funds
    let recipient = Address::generate(&e);
    
    // Verify contract has 1000 tokens, total escrowed is 0
    assert_eq!(token_client.balance(&contract_id), 1000);
    assert_eq!(client.escrowed_total(), 0);
    
    // Withdraw the stranded funds
    client.emergency_withdraw(&admin, &recipient, &token, &1000);
    
    // Verify recipient received the funds, contract has 0 left
    assert_eq!(token_client.balance(&recipient), 1000);
    assert_eq!(token_client.balance(&contract_id), 0);
}

#[test]
#[should_panic(expected = "Insufficient non-escrowed funds")]
fn test_emergency_withdraw_cannot_touch_escrow_funds() {
    let e = Env::default();
    e.mock_all_auths();

    // Setup contract and admin
    let contract_id = e.register_contract(None, VeriTixPay);
    let client = VeriTixPayClient::new(&e, &contract_id);
    
    let admin = Address::generate(&e);
    client.initialize(&admin);

    // Create token and mint some tokens to depositor
    let depositor = Address::generate(&e);
    let token = create_token_contract(&e, &depositor);
    let token_admin_client = token::StellarAssetClient::new(&e, &token);
    let token_client = token::Client::new(&e, &token);
    
    token_admin_client.mint(&depositor, &1000);
    
    // Create an escrow which locks 500 tokens in the contract
    let beneficiary = Address::generate(&e);
    let expiry = e.ledger().sequence() + 1000;
    let id = client.create_escrow(
        &depositor, &beneficiary, &token, &500, &expiry, &soroban_sdk::Bytes::new(&e)
    );
    
    // Verify contract has 500 tokens in escrow
    assert_eq!(token_client.balance(&contract_id), 500);
    assert_eq!(client.escrowed_total(), 500);
    
    // Try to withdraw 501 tokens - should panic because only 0 non-escrowed funds exist
    let recipient = Address::generate(&e);
    client.emergency_withdraw(&admin, &recipient, &token, &501);
}

// ── #583: Max supply ──────────────────────────────────────────────────────────

#[test]
fn test_set_max_supply_allows_minting_up_to_limit() {
    let e = Env::default();
    e.mock_all_auths();
    let contract_id = e.register_contract(None, VeriTixPay);
    let client = VeriTixPayClient::new(&e, &contract_id);
    let admin = Address::generate(&e);
    client.initialize(&admin);

    // Set max supply to 1000
    e.storage().persistent().set(&DataKey::MaxSupply, &1000_i128);

    // Mint 500 — should succeed (below max)
    crate::balance::increase_supply(&e, 500);
    assert_eq!(crate::balance::read_supply(&e), 500);
}

#[test]
#[should_panic(expected = "SupplyCap")]
fn test_mint_exceeding_max_supply_panics() {
    let e = Env::default();
    e.mock_all_auths();
    let contract_id = e.register_contract(None, VeriTixPay);
    let client = VeriTixPayClient::new(&e, &contract_id);
    let admin = Address::generate(&e);
    client.initialize(&admin);

    // Set max supply to 1000
    e.storage().persistent().set(&DataKey::MaxSupply, &1000_i128);

    // Mint 1000 — succeeds (exactly at max)
    crate::balance::increase_supply(&e, 1000);

    // Mint 1 more — should panic
    crate::balance::increase_supply(&e, 1);
}

#[test]
fn test_max_supply_zero_means_unlimited() {
    let e = Env::default();
    e.mock_all_auths();
    let contract_id = e.register_contract(None, VeriTixPay);
    let client = VeriTixPayClient::new(&e, &contract_id);
    let admin = Address::generate(&e);
    client.initialize(&admin);

    // Ensure max supply is 0 (default — unlimited)
    let max = crate::balance::read_max_supply(&e);
    assert_eq!(max, 0);

    // Mint a large amount — should succeed
    crate::balance::increase_supply(&e, 1_000_000);
    assert_eq!(crate::balance::read_supply(&e), 1_000_000);
}

// ── #583: initialize with max supply ──────────────────────────────────────────

#[test]
fn test_initialize_with_max_supply() {
    let e = Env::default();
    e.mock_all_auths();
    let contract_id = e.register_contract(None, VeriTixPay);
    let client = VeriTixPayClient::new(&e, &contract_id);
    let admin = Address::generate(&e);
    client.initialize(&admin);

    // Set max supply after initialization
    e.storage().persistent().set(&DataKey::MaxSupply, &5000_i128);

    assert_eq!(crate::balance::read_max_supply(&e), 5000);
    assert_eq!(crate::balance::read_supply(&e), 0);

    // Mint up to limit
    crate::balance::increase_supply(&e, 5000);
    assert_eq!(crate::balance::read_supply(&e), 5000);
}