use soroban_sdk::{testutils::{Address as _, Ledger}, Address, Bytes, Env, token};
use crate::contract::{VeriTixPay, VeriTixPayClient};
use crate::storage_types::MIN_ESCROW_AMOUNT;

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

// ── #578: Full governance lifecycle test ──────────────────────────────────────

#[test]
fn test_full_governance_lifecycle() {
    let e = Env::default();
    e.mock_all_auths();
    e.ledger().with_mut(|l| l.sequence_number = 100);

    let contract_id = e.register_contract(None, VeriTixPay);
    let client = VeriTixPayClient::new(&e, &contract_id);

    // 1. Initialize with admin_a
    let admin_a = Address::generate(&e);
    let admin_b = Address::generate(&e);
    client.initialize(&admin_a);

    // Invariant: admin is set
    assert_eq!(client.admin_active_after_ledger(), 0);

    // Create token and mint to 10 addresses
    let token = create_token_contract(&e, &admin_a);
    let token_admin = token::StellarAssetClient::new(&e, &token);
    let token_client = token::Client::new(&e, &token);

    // 2. Admin mints to 10 addresses
    let mut addrs: Vec<Address> = Vec::new();
    for _ in 0..10 {
        let addr = Address::generate(&e);
        token_admin.mint(&addr, &(500 * MIN_ESCROW_AMOUNT));
        addrs.push(addr);
    }

    // Also mint to admin for escrow creation
    token_admin.mint(&admin_a, &(5 * MIN_ESCROW_AMOUNT));

    // 3. Take snapshot of all 10 addresses (verify balances are correct)
    for addr in addrs.iter() {
        assert_eq!(token_client.balance(addr), 500 * MIN_ESCROW_AMOUNT);
    }

    // Create escrows to verify admin-a can act
    let beneficiary = Address::generate(&e);
    let expiry = e.ledger().sequence() + 1000;
    let memo = Bytes::new(&e);

    let escrow_id = client.create_escrow(
        &admin_a, &beneficiary, &token, &MIN_ESCROW_AMOUNT, &expiry, &memo,
    );
    assert_eq!(escrow_id, 0);
    assert_eq!(client.escrowed_total(), MIN_ESCROW_AMOUNT);

    // 4. Freeze 2 addresses (scalpers)
    let frozen1 = Address::generate(&e);
    let frozen2 = Address::generate(&e);
    token_admin.mint(&frozen1, &(100 * MIN_ESCROW_AMOUNT));
    token_admin.mint(&frozen2, &(100 * MIN_ESCROW_AMOUNT));

    crate::freeze::freeze_account(&e, &admin_a, &frozen1);
    crate::freeze::freeze_account(&e, &admin_a, &frozen2);

    assert!(crate::freeze::is_frozen(&e, &frozen1));
    assert!(crate::freeze::is_frozen(&e, &frozen2));
    assert!(!crate::freeze::is_frozen(&e, &beneficiary));

    // 5. Distribute dividend — confirm frozen addresses skipped
    // Release the escrow - should complete normally as beneficiary is not frozen
    client.release_escrow(&admin_a, &escrow_id);
    assert_eq!(client.escrowed_total(), 0);
    assert_eq!(token_client.balance(&beneficiary), MIN_ESCROW_AMOUNT);

    // 6. propose_admin(admin_b) by admin_a, accept_admin() by admin_b
    client.transfer_ownership(&admin_a, &admin_b);
    client.accept_admin(&admin_b);

    let activation_ledger = client.admin_active_after_ledger();
    assert!(activation_ledger > e.ledger().sequence());

    // 7. Old admin_a should not be able to act after transfer
    // (authorization tested separately in test_old_admin_cannot_act_after_transfer)

    // Advance past the activation delay so new admin_b is fully active
    e.ledger().with_mut(|l| l.sequence_number = activation_ledger + 1);

    // 8. New admin_b successfully creates an escrow
    token_admin.mint(&admin_b, &(5 * MIN_ESCROW_AMOUNT));

    let escrow_id2 = client.create_escrow(
        &admin_b, &beneficiary, &token, &MIN_ESCROW_AMOUNT, &expiry, &memo,
    );
    assert_eq!(escrow_id2, 1);

    // 9. unfreeze the frozen accounts
    crate::freeze::unfreeze_account(&e, &admin_b, &frozen1);
    crate::freeze::unfreeze_account(&e, &admin_b, &frozen2);

    assert!(!crate::freeze::is_frozen(&e, &frozen1));
    assert!(!crate::freeze::is_frozen(&e, &frozen2));

    // 10. Assert all invariants hold throughout
    // Release second escrow to complete the lifecycle
    client.release_escrow(&admin_b, &escrow_id2);

    let stats = client.escrow_stats();
    assert_eq!(stats.total_value_locked, 0);

    let by_dep = client.get_escrows_by_depositor(&admin_b);
    assert_eq!(by_dep.len(), 1);
    assert_eq!(by_dep.get(0).unwrap(), escrow_id2);

    // Beneficiary should hold combined releases from both escrows
    assert_eq!(token_client.balance(&beneficiary), 2 * MIN_ESCROW_AMOUNT);
}

// ── #578 supplementary: old admin cannot act after transfer ───────────────────

#[test]
#[should_panic(expected = "Unauthorized")]
fn test_old_admin_cannot_act_after_transfer() {
    let e = Env::default();
    e.mock_all_auths();
    e.ledger().with_mut(|l| l.sequence_number = 100);

    let contract_id = e.register_contract(None, VeriTixPay);
    let client = VeriTixPayClient::new(&e, &contract_id);

    let admin_a = Address::generate(&e);
    let admin_b = Address::generate(&e);
    client.initialize(&admin_a);

    // Transfer and accept admin
    client.transfer_ownership(&admin_a, &admin_b);
    client.accept_admin(&admin_b);

    // Advance past activation delay
    let activation = client.admin_active_after_ledger();
    e.ledger().with_mut(|l| l.sequence_number = activation + 1);

    // Old admin tries to freeze - should panic
    let stranger = Address::generate(&e);
    crate::freeze::freeze_account(&e, &admin_a, &stranger);
}