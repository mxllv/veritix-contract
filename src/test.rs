use soroban_sdk::{Address, Env, token};
use crate::contract::{VeriTixPay, VeriTixPayClient};

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

#[test]
fn test_full_event_lifecycle() {
    let e = Env::default();
    e.mock_all_auths();

    // Setup contract and admin
    let contract_id = e.register_contract(None, VeriTixPay);
    let client = VeriTixPayClient::new(&e, &contract_id);
    
    let admin = Address::generate(&e);
    client.initialize(&admin);

    // Create token and mint 1000 VTX to buyer
    let buyer = Address::generate(&e);
    let organizer = Address::generate(&e);
    let artist = Address::generate(&e);
    let platform = Address::generate(&e);
    
    let token = create_token_contract(&e, &admin);
    let token_admin_client = token::StellarAssetClient::new(&e, &token);
    let token_client = token::Client::new(&e, &token);
    
    token_admin_client.mint(&buyer, &1000);
    
    // Verify initial buyer balance
    assert_eq!(token_client.balance(&buyer), 1000);
    assert_eq!(token_client.balance(&contract_id), 0);
    
    // Record initial total supply
    let initial_total_supply = 1000; // Minted 1000 tokens
    
    // Buyer calls create_escrow(buyer, organizer, 100, event_ledger)
    let event_ledger = e.ledger().sequence() + 100;
    let ticket_ref = soroban_sdk::Bytes::from_slice(&e, b"ticket_ref");
    let escrow_id = client.ticket_escrow(&buyer, &organizer, &token, &100, &event_ledger, &ticket_ref);
    
    // Confirm buyer balance reduced by 100, contract holds 100
    assert_eq!(token_client.balance(&buyer), 900);
    assert_eq!(token_client.balance(&contract_id), 100);
    assert_eq!(client.escrowed_total(), 100);
    
    // Advance ledger past event_ledger
    e.ledger().with_mut(|l| l.sequence_number = event_ledger + 10);
    
    // Organizer calls release_escrow
    client.release_escrow(&organizer, &escrow_id);
    
    // Confirm organizer received 100
    assert_eq!(token_client.balance(&organizer), 100);
    assert_eq!(token_client.balance(&contract_id), 0);
    assert_eq!(client.escrowed_total(), 0);
    
    // Organizer calls create_split([organizer 8000bps, artist 1500bps, platform 500bps], 100)
    let split_event_ledger = e.ledger().sequence() + 100;
    let recipients = soroban_sdk::Vec::from_array(
        &e,
        [
            (organizer.clone(), 8000),
            (artist.clone(), 1500),
            (platform.clone(), 500),
        ],
    );
    let split_id = crate::splitter::create_split(
        e.clone(),
        organizer.clone(),
        recipients,
        token.clone(),
        100,
        split_event_ledger,
    );
    
    // Organizer calls distribute(split_id)
    crate::splitter::distribute_split(e.clone(), organizer.clone(), split_id);
    
    // Confirm: organizer 80, artist 15, platform 5
    assert_eq!(token_client.balance(&organizer), 80);
    assert_eq!(token_client.balance(&artist), 15);
    assert_eq!(token_client.balance(&platform), 5);
    
    // Assert read_total_supply unchanged throughout
    // Total supply should still be 1000 (buyer 900 + organizer 80 + artist 15 + platform 5 = 1000)
    assert_eq!(token_client.balance(&buyer) + token_client.balance(&organizer) + token_client.balance(&artist) + token_client.balance(&platform), initial_total_supply);
    
    // Assert contract balance is 0 after all operations
    assert_eq!(token_client.balance(&contract_id), 0);
}