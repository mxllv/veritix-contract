use soroban_sdk::{testutils::Address as _, testutils::Ledger as _, Address, BytesN, Env, token, Vec};
use soroban_sdk::xdr::ToXdr;
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

// ═══════════════════════════════════════════════════════════════════════════════
// #571: Vesting schedule tests
// ═══════════════════════════════════════════════════════════════════════════════

#[test]
fn test_create_and_claim_vesting() {
    let e = Env::default();
    e.mock_all_auths();

    let contract_id = e.register_contract(None, VeriTixPay);
    let client = VeriTixPayClient::new(&e, &contract_id);

    let admin = Address::generate(&e);
    client.initialize(&admin);

    let holder = Address::generate(&e);
    let token = create_token_contract(&e, &admin);
    let token_admin_client = token::StellarAssetClient::new(&e, &token);
    let token_client = token::Client::new(&e, &token);

    // Mint tokens to admin
    token_admin_client.mint(&admin, &1000);

    let vesting_ledger = e.ledger().sequence() + 100;
    let amount: i128 = 500;

    // Create vesting
    let vesting_id = client.create_vesting(&admin, &holder, &token, &amount, &vesting_ledger);
    assert_eq!(vesting_id, 0);

    // Tokens moved from admin to contract
    assert_eq!(token_client.balance(&admin), 500);
    assert_eq!(token_client.balance(&contract_id), 500);

    // Get vestings by holder
    let vestings = client.get_vesting_by_holder(&holder);
    assert_eq!(vestings.len(), 1);
    assert_eq!(vestings.get(0).unwrap(), 0u32);

    // Advance to vesting ledger
    e.ledger().with_mut(|l| l.sequence_number = vesting_ledger);

    // Claim vesting
    client.claim_vesting(&holder, &vesting_id);

    // Tokens transferred to holder
    assert_eq!(token_client.balance(&holder), 500);
    assert_eq!(token_client.balance(&contract_id), 0);
}

#[test]
#[should_panic(expected = "vesting period not yet reached")]
fn test_claim_vesting_before_vesting_panics() {
    let e = Env::default();
    e.mock_all_auths();

    let contract_id = e.register_contract(None, VeriTixPay);
    let client = VeriTixPayClient::new(&e, &contract_id);

    let admin = Address::generate(&e);
    client.initialize(&admin);

    let holder = Address::generate(&e);
    let token = create_token_contract(&e, &admin);
    let token_admin_client = token::StellarAssetClient::new(&e, &token);
    token_admin_client.mint(&admin, &1000);

    let vesting_ledger = e.ledger().sequence() + 100;
    let vesting_id = client.create_vesting(&admin, &holder, &token, &500, &vesting_ledger);

    // Try claiming before vesting ledger — should panic
    client.claim_vesting(&holder, &vesting_id);
}

#[test]
#[should_panic(expected = "vesting already claimed")]
fn test_double_claim_vesting_panics() {
    let e = Env::default();
    e.mock_all_auths();

    let contract_id = e.register_contract(None, VeriTixPay);
    let client = VeriTixPayClient::new(&e, &contract_id);

    let admin = Address::generate(&e);
    client.initialize(&admin);

    let holder = Address::generate(&e);
    let token = create_token_contract(&e, &admin);
    let token_admin_client = token::StellarAssetClient::new(&e, &token);
    token_admin_client.mint(&admin, &1000);

    let vesting_ledger = e.ledger().sequence() + 100;
    let vesting_id = client.create_vesting(&admin, &holder, &token, &500, &vesting_ledger);

    e.ledger().with_mut(|l| l.sequence_number = vesting_ledger);

    client.claim_vesting(&holder, &vesting_id);
    // Second claim should panic
    client.claim_vesting(&holder, &vesting_id);
}

#[test]
#[should_panic(expected = "vesting_ledger must be in the future")]
fn test_create_vesting_past_ledger_panics() {
    let e = Env::default();
    e.mock_all_auths();

    let contract_id = e.register_contract(None, VeriTixPay);
    let client = VeriTixPayClient::new(&e, &contract_id);

    let admin = Address::generate(&e);
    client.initialize(&admin);

    let holder = Address::generate(&e);
    let token = create_token_contract(&e, &admin);
    let token_admin_client = token::StellarAssetClient::new(&e, &token);
    token_admin_client.mint(&admin, &1000);

    // Create vesting with past ledger
    let past_ledger = e.ledger().sequence();
    client.create_vesting(&admin, &holder, &token, &500, &past_ledger);
}

// ═══════════════════════════════════════════════════════════════════════════════
// #572: Split-to-escrow tests
// ═══════════════════════════════════════════════════════════════════════════════

#[test]
fn test_split_to_escrow_creates_per_recipient_escrows() {
    let e = Env::default();
    e.mock_all_auths();

    let contract_id = e.register_contract(None, VeriTixPay);
    let client = VeriTixPayClient::new(&e, &contract_id);

    let admin = Address::generate(&e);
    client.initialize(&admin);

    let sender = Address::generate(&e);
    let recipient1 = Address::generate(&e);
    let recipient2 = Address::generate(&e);
    let token = create_token_contract(&e, &admin);
    let token_admin_client = token::StellarAssetClient::new(&e, &token);
    let token_client = token::Client::new(&e, &token);

    token_admin_client.mint(&sender, &10000);

    let expiry = e.ledger().sequence() + 1000;
    let receivers = Vec::from_array(&e, [(recipient1.clone(), 6000u32), (recipient2.clone(), 4000u32)]);

    let escrow_ids = client.split_to_escrow(&sender, &receivers, &token, &10000i128, &expiry);
    assert_eq!(escrow_ids.len(), 2);

    // Verify tokens were pulled from sender
    assert_eq!(token_client.balance(&sender), 0);
    assert_eq!(token_client.balance(&contract_id), 10000);

    // Verify first recipient's escrow (60% = 6000)
    let rec1 = client.get_escrow(&escrow_ids.get(0).unwrap());
    assert_eq!(rec1.beneficiary, recipient1);
    assert_eq!(rec1.amount, 6000);

    // Verify second recipient's escrow (40% = 4000)
    let rec2 = client.get_escrow(&escrow_ids.get(1).unwrap());
    assert_eq!(rec2.beneficiary, recipient2);
    assert_eq!(rec2.amount, 4000);
}

#[test]
#[should_panic(expected = "total basis points must equal 10000")]
fn test_split_to_escrow_invalid_bps_panics() {
    let e = Env::default();
    e.mock_all_auths();

    let contract_id = e.register_contract(None, VeriTixPay);
    let client = VeriTixPayClient::new(&e, &contract_id);

    let admin = Address::generate(&e);
    client.initialize(&admin);

    let sender = Address::generate(&e);
    let token = create_token_contract(&e, &admin);
    let token_admin_client = token::StellarAssetClient::new(&e, &token);
    token_admin_client.mint(&sender, &10000);

    let expiry = e.ledger().sequence() + 1000;
    // Only 5000 + 3000 = 8000 bps, not 10000
    let recipients = Vec::from_array(&e, [
        (Address::generate(&e), 5000u32),
        (Address::generate(&e), 3000u32),
    ]);

    client.split_to_escrow(&sender, &recipients, &token, &10000i128, &expiry);
}

#[test]
#[should_panic(expected = "must have at least one recipient")]
fn test_split_to_escrow_empty_recipients_panics() {
    let e = Env::default();
    e.mock_all_auths();

    let contract_id = e.register_contract(None, VeriTixPay);
    let client = VeriTixPayClient::new(&e, &contract_id);

    let admin = Address::generate(&e);
    client.initialize(&admin);

    let sender = Address::generate(&e);
    let token = create_token_contract(&e, &admin);
    let token_admin_client = token::StellarAssetClient::new(&e, &token);
    token_admin_client.mint(&sender, &10000);

    let expiry = e.ledger().sequence() + 1000;
    let empty: soroban_sdk::Vec<(Address, u32)> = Vec::new(&e);

    client.split_to_escrow(&sender, &empty, &token, &10000i128, &expiry);
}

#[test]
#[should_panic(expected = "duplicate recipient address")]
fn test_split_to_escrow_duplicate_recipient_panics() {
    let e = Env::default();
    e.mock_all_auths();

    let contract_id = e.register_contract(None, VeriTixPay);
    let client = VeriTixPayClient::new(&e, &contract_id);

    let admin = Address::generate(&e);
    client.initialize(&admin);

    let sender = Address::generate(&e);
    let dup = Address::generate(&e);
    let token = create_token_contract(&e, &admin);
    let token_admin_client = token::StellarAssetClient::new(&e, &token);
    token_admin_client.mint(&sender, &10000);

    let expiry = e.ledger().sequence() + 1000;
    // Same address twice — should panic
    let recipients = Vec::from_array(&e, [
        (dup.clone(), 6000u32),
        (dup.clone(), 4000u32),
    ]);

    client.split_to_escrow(&sender, &recipients, &token, &10000i128, &expiry);
}

// ═══════════════════════════════════════════════════════════════════════════════
// #573: Airdrop tests
// ═══════════════════════════════════════════════════════════════════════════════

#[test]
fn test_airdrop_distributes_proportionally() {
    let e = Env::default();
    e.mock_all_auths();

    let contract_id = e.register_contract(None, VeriTixPay);
    let client = VeriTixPayClient::new(&e, &contract_id);

    let admin = Address::generate(&e);
    client.initialize(&admin);

    let holder1 = Address::generate(&e);
    let holder2 = Address::generate(&e);
    let token = create_token_contract(&e, &admin);
    let token_admin_client = token::StellarAssetClient::new(&e, &token);
    let token_client = token::Client::new(&e, &token);

    // Set up holders via escrow releases (which tracks them in HolderSet)
    let depositor = Address::generate(&e);
    token_admin_client.mint(&depositor, &10000);
    token_admin_client.mint(&admin, &2000);

    let expiry = e.ledger().sequence() + 1000;

    // Create and release escrow for holder1 (70% = 7000)
    let escrow_id1 = client.create_escrow(
        &depositor,
        &holder1,
        &token,
        &7000,
        &expiry,
        &soroban_sdk::Bytes::new(&e),
    );
    // Advance timestamp past rate-limit cooldown (300s)
    e.ledger().with_mut(|l| {
        l.sequence_number += 1;
        l.timestamp += 301;
    });

    // Create and release escrow for holder2 (30% = 3000)
    let escrow_id2 = client.create_escrow(
        &depositor,
        &holder2,
        &token,
        &3000,
        &expiry,
        &soroban_sdk::Bytes::new(&e),
    );

    // Advance ledger for release operations
    e.ledger().with_mut(|l| {
        l.sequence_number += 1;
        l.timestamp += 301;
    });

    // Release both escrows — this adds beneficiaries to HolderSet
    client.release_escrow(&depositor, &escrow_id1);
    client.release_escrow(&depositor, &escrow_id2);

    // Now holders have balances: h1=7000, h2=3000
    assert_eq!(token_client.balance(&holder1), 7000);
    assert_eq!(token_client.balance(&holder2), 3000);

    // Airdrop 1000 tokens: h1 gets 700, h2 gets 300 (proportional)
    e.ledger().with_mut(|l| l.sequence_number = e.ledger().sequence() + 1);
    client.airdrop(&admin, &token, &1000i128);

    // Verify proportional distribution
    // h1: 7000/10000 * 1000 = 700
    // h2: 3000/10000 * 1000 = 300
    assert_eq!(token_client.balance(&holder1), 7700);
    assert_eq!(token_client.balance(&holder2), 3300);
}

#[test]
#[should_panic(expected = "no holders to airdrop to")]
fn test_airdrop_no_holders_panics() {
    let e = Env::default();
    e.mock_all_auths();

    let contract_id = e.register_contract(None, VeriTixPay);
    let client = VeriTixPayClient::new(&e, &contract_id);

    let admin = Address::generate(&e);
    client.initialize(&admin);

    let token = create_token_contract(&e, &admin);
    let token_admin_client = token::StellarAssetClient::new(&e, &token);
    token_admin_client.mint(&admin, &1000);

    // No holders registered — should panic
    client.airdrop(&admin, &token, &500i128);
}

// ═══════════════════════════════════════════════════════════════════════════════
// #574: Permit batch tests
// ═══════════════════════════════════════════════════════════════════════════════

#[test]
#[should_panic(expected = "approvals cannot be empty")]
fn test_permit_batch_empty_approvals_panics() {
    let e = Env::default();
    e.mock_all_auths();

    let contract_id = e.register_contract(None, VeriTixPay);
    let client = VeriTixPayClient::new(&e, &contract_id);

    let admin = Address::generate(&e);
    client.initialize(&admin);

    let owner = Address::generate(&e);
    let empty: soroban_sdk::Vec<(Address, i128, u32)> = Vec::new(&e);
    let pk = BytesN::<32>::from_array(&e, &[0u8; 32]);
    let sig = BytesN::<64>::from_array(&e, &[0u8; 64]);
    client.permit_batch(&owner, &empty, &0u64, &pk, &sig);
}

#[test]
fn test_permit_batch_with_valid_signature() {
    let e = Env::default();
    e.mock_all_auths();

    let contract_id = e.register_contract(None, VeriTixPay);
    let client = VeriTixPayClient::new(&e, &contract_id);

    let admin = Address::generate(&e);
    client.initialize(&admin);

    // Generate a keypair for the owner
    let owner_key = e.crypto().ed25519_generate();
    let owner_pk = owner_key.public_key();
    let owner = Address::from_public_key(&owner_pk);

    let spender1 = Address::generate(&e);
    let spender2 = Address::generate(&e);
    let expiry = e.ledger().sequence() + 1000;

    // Build approvals
    let approvals = Vec::from_array(
        &e,
        [
            (spender1.clone(), 500i128, expiry),
            (spender2.clone(), 300i128, expiry),
        ],
    );

    // Compute the hash the same way permit_batch does (manually for the test)
    let nonce: u64 = client.permit_nonces(&owner);

    let mut msg = soroban_sdk::Bytes::new(&e);
    msg.append(&soroban_sdk::symbol_short!("permit_bt").to_xdr(&e));
    msg.append(&owner.to_xdr(&e));
    for i in 0..approvals.len() {
        let (sp, amt, exp) = approvals.get(i).unwrap();
        msg.append(&sp.to_xdr(&e));
        msg.append(&amt.to_xdr(&e));
        msg.append(&exp.to_xdr(&e));
    }
    msg.append(&nonce.to_xdr(&e));
    let hash = e.crypto().sha256(&msg);
    let hash_bytes: soroban_sdk::Bytes = hash.into();
    let sig = owner_key.sign(&hash_bytes);

    // Execute permit_batch with the valid signature
    client.permit_batch(&owner, &approvals, &nonce, &owner_pk, &sig);

    // Nonce should have incremented
    assert_eq!(client.permit_nonces(&owner), 1u64);
}

#[test]
#[should_panic(expected = "invalid nonce")]
fn test_permit_batch_wrong_nonce_panics() {
    let e = Env::default();
    e.mock_all_auths();

    let contract_id = e.register_contract(None, VeriTixPay);
    let client = VeriTixPayClient::new(&e, &contract_id);

    let admin = Address::generate(&e);
    client.initialize(&admin);

    let owner_key = e.crypto().ed25519_generate();
    let owner_pk = owner_key.public_key();
    let owner = Address::from_public_key(&owner_pk);

    let spender = Address::generate(&e);
    let expiry = e.ledger().sequence() + 1000;
    let approvals = Vec::from_array(&e, [(spender, 500i128, expiry)]);

    let pk = BytesN::<32>::from_array(&e, &[0u8; 32]);
    let sig = BytesN::<64>::from_array(&e, &[0u8; 64]);

    // Use wrong nonce (1 instead of 0) — should panic
    client.permit_batch(&owner, &approvals, &1u64, &pk, &sig);
}

#[test]
#[should_panic(expected = "invalid nonce")]
fn test_permit_batch_replay_attack_panics() {
    let e = Env::default();
    e.mock_all_auths();

    let contract_id = e.register_contract(None, VeriTixPay);
    let client = VeriTixPayClient::new(&e, &contract_id);

    let admin = Address::generate(&e);
    client.initialize(&admin);

    let owner_key = e.crypto().ed25519_generate();
    let owner_pk = owner_key.public_key();
    let owner = Address::from_public_key(&owner_pk);

    let spender = Address::generate(&e);
    let expiry = e.ledger().sequence() + 1000;
    let approvals = Vec::from_array(&e, [(spender.clone(), 500i128, expiry)]);

    let nonce: u64 = client.permit_nonces(&owner);

    // Build and sign message
    let mut msg = soroban_sdk::Bytes::new(&e);
    msg.append(&soroban_sdk::symbol_short!("permit_bt").to_xdr(&e));
    msg.append(&owner.to_xdr(&e));
    for i in 0..approvals.len() {
        let (sp, amt, exp) = approvals.get(i).unwrap();
        msg.append(&sp.to_xdr(&e));
        msg.append(&amt.to_xdr(&e));
        msg.append(&exp.to_xdr(&e));
    }
    msg.append(&nonce.to_xdr(&e));
    let hash = e.crypto().sha256(&msg);
    let hash_bytes: soroban_sdk::Bytes = hash.into();
    let sig = owner_key.sign(&hash_bytes);

    // First call succeeds
    client.permit_batch(&owner, &approvals, &nonce, &owner_pk, &sig);

    // Replay the same message — should panic with invalid nonce
    client.permit_batch(&owner, &approvals, &nonce, &owner_pk, &sig);
}