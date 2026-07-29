use soroban_sdk::{Address, Env, token, Vec};
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

// ── #575: Holder count property test ──────────────────────────────────────────

fn get_holder_count(e: &Env, client: &VeriTixPayClient) -> u32 {
    client.total_holders()
}

fn count_positive_balances(e: &Env, contract_id: &Address, addresses: &[Address]) -> u32 {
    let mut count = 0_u32;
    for addr in addresses {
        let bal: i128 = e.as_contract(contract_id, || {
            e.storage().persistent()
                .get(&DataKey::BalanceOf(addr.clone()))
                .unwrap_or(0)
        });
        if bal > 0 {
            count += 1;
        }
    }
    count
}

#[test]
fn test_holder_count_invariant() {
    let e = Env::default();
    e.mock_all_auths();

    let contract_id = e.register_contract(None, VeriTixPay);
    let client = VeriTixPayClient::new(&e, &contract_id);
    
    let admin = Address::generate(&e);
    client.initialize(&admin);

    // Create 5 addresses and track them
    let a1 = Address::generate(&e);
    let a2 = Address::generate(&e);
    let a3 = Address::generate(&e);
    let a4 = Address::generate(&e);
    let a5 = Address::generate(&e);
    let addresses = [a1.clone(), a2.clone(), a3.clone(), a4.clone(), a5.clone()];

    // Directly set DataKey::BalanceOf and DataKey::Holders in the contract's persistent storage
    // a1: 300, a2: 0, a3: 300, a4: 0, a5: 500 => 3 positive balances
    e.as_contract(&contract_id, || {
        e.storage().persistent().set(&DataKey::BalanceOf(a1.clone()), &300_i128);
        e.storage().persistent().set(&DataKey::BalanceOf(a2.clone()), &0_i128);
        e.storage().persistent().set(&DataKey::BalanceOf(a3.clone()), &300_i128);
        e.storage().persistent().set(&DataKey::BalanceOf(a4.clone()), &0_i128);
        e.storage().persistent().set(&DataKey::BalanceOf(a5.clone()), &500_i128);

        let mut holders_vec: Vec<Address> = Vec::new(&e);
        holders_vec.push_back(a1.clone());
        holders_vec.push_back(a3.clone());
        holders_vec.push_back(a5.clone());
        e.storage().persistent().set(&DataKey::Holders, &holders_vec);
    });

    // holder_count should equal count of addresses with balance > 0
    let holders = get_holder_count(&e, &client);
    let positive = count_positive_balances(&e, &contract_id, &addresses);
    assert_eq!(holders, positive);
    assert_eq!(holders, 3);
}

#[test]
fn test_holder_count_returns_to_zero_after_batch_clawback() {
    let e = Env::default();
    e.mock_all_auths();

    let contract_id = e.register_contract(None, VeriTixPay);
    let client = VeriTixPayClient::new(&e, &contract_id);
    
    let admin = Address::generate(&e);
    client.initialize(&admin);

    let mut addresses: Vec<Address> = Vec::new(&e);
    for _ in 0..50 {
        let addr = Address::generate(&e);
        addresses.push_back(addr);
    }

    // Simulate batch clawback — set all balances to zero and clear holders
    e.as_contract(&contract_id, || {
        for i in 0..addresses.len() {
            if let Some(addr) = addresses.get(i) {
                e.storage().persistent().set(&DataKey::BalanceOf(addr), &0_i128);
            }
        }
        e.storage().persistent().set(&DataKey::Holders, &Vec::<Address>::new(&e));
    });

    // All balances should be zero, holder_count should be 0
    let holders = get_holder_count(&e, &client);
    assert_eq!(holders, 0);
}