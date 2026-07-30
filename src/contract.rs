use soroban_sdk::{contract, contractimpl, contracttype, Address, Bytes, Env, String, Vec};
use crate::{escrow, multi_escrow, allowance, admin, dispute, recurring, balance, freeze, whitelist};
use crate::storage_types::{DataKey, RecurringPayment, ResolverStats};
use crate::validation::require_positive_amount;

// #573: Airdrop holder set tracking helper
fn track_holder_for_airdrop(e: &Env, addr: &Address) {
    let mut holders: Vec<Address> = e
        .storage()
        .persistent()
        .get(&DataKey::HolderSet)
        .unwrap_or_else(|| Vec::new(e));

    // Check if already present
    for i in 0..holders.len() {
        if holders.get(i).unwrap() == *addr {
            return;
        }
    }

    holders.push_back(addr.clone());
    e.storage()
        .persistent()
        .set(&DataKey::HolderSet, &holders);
    let count: u32 = e
        .storage()
        .persistent()
        .get(&DataKey::HolderCount)
        .unwrap_or(0);
    e.storage()
        .persistent()
        .set(&DataKey::HolderCount, &(count + 1));
}

pub trait VeriTixPayTrait {
    fn initialize(e: Env, admin: Address);
    fn initialize_with_max_supply(e: Env, admin: Address, max_supply: i128);

    // ── Escrow ────────────────────────────────────────────────────────────────
    fn create_escrow(
        e: Env,
        depositor: Address,
        beneficiary: Address,
        token: Address,
        amount: i128,
        expiry_ledger: u32,
        memo: Bytes,            // #175
    ) -> u32;

    fn release_escrow(e: Env, caller: Address, escrow_id: u32);
    fn release_partial_escrow(e: Env, caller: Address, escrow_id: u32, amount: i128); // #174
    fn refund_escrow(e: Env, caller: Address, escrow_id: u32);
    fn get_escrows_by_depositor(e: Env, depositor: Address) -> Vec<u32>;
    fn get_escrows_by_beneficiary(e: Env, beneficiary: Address) -> Vec<u32>;
    fn escrowed_total(e: Env) -> i128;
    fn escrow_stats(e: Env) -> escrow::EscrowStats;
    fn place_lien(e: Env, creditor: Address, escrow_id: u32, lien_amount: i128);
    fn clear_lien(e: Env, caller: Address, escrow_id: u32);
    fn get_escrow(e: Env, escrow_id: u32) -> escrow::EscrowRecord;
    fn is_escrow_settled(e: Env, escrow_id: u32) -> bool;

    // ── Disputes ──────────────────────────────────────────────────────────────
    fn get_disputes_by_claimant(e: Env, claimant: Address) -> Vec<u32>;
    fn set_arbiter(e: Env, arbiter: Address);
    fn raise_dispute(e: Env, caller: Address, escrow_id: u32);
    fn resolve_dispute(e: Env, resolver: Address, escrow_id: u32, winner: Address);

    // ── Recurring Payments ────────────────────────────────────────────────────
    fn setup_recurring(
        e: Env,
        payer: Address,
        payee: Address,
        token: Address,
        amount: i128,
        interval: u32,
        max_executions: u32,
    ) -> u32;
    fn execute_recurring(e: Env, recurring_id: u32);
    fn get_recurring_history(e: Env, recurring_id: u32) -> Vec<RecurringPayment>;
    fn is_recurring_active(e: Env, recurring_id: u32) -> bool;
    fn get_escrows_batch(e: Env, escrow_ids: Vec<u32>) -> Vec<Option<escrow::EscrowRecord>>;
    fn get_escrow_age(e: Env, escrow_id: u32) -> u32;

    // ── Multi-escrow ──────────────────────────────────────────────────────────
    fn create_multi_escrow(
        e: Env,
        depositor: Address,
        recipients: Vec<(Address, i128)>,
        token: Address,
        expiry_ledger: u32,
    ) -> u32;
    fn release_multi_escrow(e: Env, caller: Address, multi_escrow_id: u32);
    fn refund_multi_escrow(e: Env, caller: Address, multi_escrow_id: u32);
    fn ticket_escrow(
        e: Env,
        buyer: Address,
        organizer: Address,
        token: Address,
        ticket_price: i128,
        event_ledger: u32,
        ticket_ref: Bytes,
    ) -> u32;
    #[allow(clippy::too_many_arguments)]
    fn revenue_split(
        e: Env,
        sender: Address,
        organizer: Address,
        organizer_bps: u32,
        artist: Address,
        artist_bps: u32,
        platform: Address,
        token: Address,
        total_amount: i128,
        event_ledger: u32,
    ) -> u32;

    // ── Allowance ─────────────────────────────────────────────────────────────
    fn approve(e: Env, from: Address, spender: Address, amount: i128, expiration_ledger: u32);
    fn transfer_from(e: Env, spender: Address, from: Address, to: Address, amount: i128);

    // ── #451: Admin ownership ────────────────────────────────────────────────
    fn transfer_ownership(e: Env, new_admin: Address);
    fn accept_admin(e: Env, new_admin: Address);
    fn admin_active_after_ledger(e: Env) -> u32;

    // ── #452: Depositor escrowed value ───────────────────────────────────────
    fn escrowed_value_for_depositor(e: Env, depositor: Address) -> i128;

    // ── Pause ─────────────────────────────────────────────────────────────────
    fn set_paused(e: Env, admin: Address, paused: bool);
    fn is_paused(e: Env) -> bool;

    // ── #453: Resolver stats ─────────────────────────────────────────────────
    fn resolver_stats(e: Env, resolver: Address) -> ResolverStats;

    // ── #454: Protocol fee stats ─────────────────────────────────────────────
    fn protocol_fee_stats(e: Env) -> (u32, Address, i128);
    fn emergency_withdraw(e: Env, admin: Address, recipient: Address, token: Address, amount: i128);

    fn amend_recurring(e: Env, caller: Address, recurring_id: u32, new_amount: i128, new_interval: u32);
    fn recurring_count_for_payee(e: Env, payee: Address) -> u32;
    fn recurring_ids_for_payee(e: Env, payee: Address) -> Vec<u32>;
    fn cancel_split(e: Env, caller: Address, split_id: u32);
    fn transfer_escrow_beneficiary(e: Env, depositor: Address, escrow_id: u32, new_beneficiary: Address);
    fn total_holders(e: Env) -> u32;
    fn get_holders(e: Env) -> Vec<Address>;
    fn set_mediation_fee(e: Env, admin: Address, fee_bps: u32);
    fn version(e: Env) -> soroban_sdk::String;
    fn contract_summary(e: Env) -> ContractSummary;
    fn spendable_balance(e: Env, account: Address) -> i128;
    fn set_authorized(e: Env, admin: Address, account: Address, authorized: bool);
    fn increase_allowance(e: Env, from: Address, spender: Address, amount: i128);
    fn decrease_allowance(e: Env, from: Address, spender: Address, amount: i128);
    fn burn_from(e: Env, spender: Address, from: Address, amount: i128);
    fn transfer_with_memo(e: Env, from: Address, to: Address, amount: i128, memo: Bytes);
    fn revoke_all_allowances(e: Env, from: Address);
    fn enable_whitelist(e: Env, admin: Address);
    fn disable_whitelist(e: Env, admin: Address);
    fn add_to_whitelist(e: Env, admin: Address, account: Address);
    fn remove_from_whitelist(e: Env, admin: Address, account: Address);
    fn is_whitelisted(e: Env, account: Address) -> bool;
    fn set_protocol_fee(e: Env, admin: Address, fee_bps: u32, treasury: Address);
    fn trigger_auto_release(e: Env, escrow_id: u32);
    fn escrow_between(e: Env, addr1: Address, addr2: Address) -> u32;
    fn cancel_recurring_batch(e: Env, caller: Address, recurring_ids: Vec<u32>);
}

#[contracttype]
#[derive(Clone)]
pub struct ContractSummary {
    pub admin: Address,
    pub total_supply: i128,
    pub escrow_count: u32,
    pub total_value_locked: i128,
}

#[contract]
pub struct VeriTixPay;

#[contractimpl]
impl VeriTixPayTrait for VeriTixPay {
    fn initialize(env: Env, admin: Address) {
        admin::validate_admin_address(&env, &admin);

        if env.storage().persistent().has(&DataKey::Admin) {
            panic!("AlreadyInitialized: contract state is locked");
        }

        env.storage().persistent().set(&DataKey::Admin, &admin);
    }

    fn initialize_with_max_supply(env: Env, admin: Address, max_supply: i128) {
        admin::validate_admin_address(&env, &admin);

        if env.storage().persistent().has(&DataKey::Admin) {
            panic!("AlreadyInitialized: contract state is locked");
        }

        env.storage().persistent().set(&DataKey::Admin, &admin);
        env.storage().persistent().set(&DataKey::MaxSupply, &max_supply);
    }

    fn create_escrow(
        e: Env,
        depositor: Address,
        beneficiary: Address,
        token: Address,
        amount: i128,
        expiry_ledger: u32,
        memo: Bytes,
    ) -> u32 {
        require_positive_amount(amount);
        escrow::create_escrow(e, depositor, beneficiary, token, amount, expiry_ledger, memo)
    }

    fn release_escrow(e: Env, caller: Address, escrow_id: u32) {
        // Track the beneficiary in the holder set for airdrop (#573)
        let record = escrow::load_record(&e, escrow_id);
        let beneficiary = record.beneficiary.clone();
        escrow::release_escrow(e.clone(), caller, escrow_id);
        track_holder_for_airdrop(&e, &beneficiary);
    }

    fn release_partial_escrow(e: Env, caller: Address, escrow_id: u32, amount: i128) {
        require_positive_amount(amount);
        // Track the beneficiary in the holder set for airdrop (#573)
        let record = escrow::load_record(&e, escrow_id);
        let beneficiary = record.beneficiary.clone();
        escrow::release_partial_escrow(e.clone(), caller, escrow_id, amount);
        track_holder_for_airdrop(&e, &beneficiary);
    }

    fn refund_escrow(e: Env, caller: Address, escrow_id: u32) {
        escrow::refund_escrow(e, caller, escrow_id)
    }

    fn get_escrows_by_depositor(e: Env, depositor: Address) -> Vec<u32> {
        escrow::get_escrows_by_depositor(e, depositor)
    }

    fn get_escrows_by_beneficiary(e: Env, beneficiary: Address) -> Vec<u32> {
        escrow::get_escrows_by_beneficiary(e, beneficiary)
    }

    fn escrowed_total(e: Env) -> i128 {
        escrow::get_escrowed_total(&e)
    }

    fn escrow_stats(e: Env) -> escrow::EscrowStats {
        escrow::get_escrow_stats(&e)
    }

    fn place_lien(e: Env, creditor: Address, escrow_id: u32, lien_amount: i128) {
        escrow::place_lien(e, creditor, escrow_id, lien_amount)
    }

    fn clear_lien(e: Env, caller: Address, escrow_id: u32) {
        escrow::clear_lien(e, caller, escrow_id)
    }

    fn get_escrow(e: Env, escrow_id: u32) -> escrow::EscrowRecord {
        escrow::load_record(&e, escrow_id)
    }

    fn is_escrow_settled(e: Env, escrow_id: u32) -> bool {
        match e.storage().persistent().get::<DataKey, escrow::EscrowRecord>(&DataKey::Escrow(escrow_id))
        {
            Some(escrow) => escrow.released || escrow.refunded,
            None => true,
        }
    }

    fn get_disputes_by_claimant(e: Env, claimant: Address) -> Vec<u32> {
        dispute::get_disputes_by_claimant(e, claimant)
    }

    fn set_arbiter(e: Env, arbiter: Address) {
        dispute::set_arbiter(&e, &arbiter)
    }

    fn resolve_dispute(e: Env, resolver: Address, escrow_id: u32, winner: Address) {
        dispute::resolve_dispute(&e, &resolver, escrow_id, &winner)
    }

    fn setup_recurring(
        e: Env,
        payer: Address,
        payee: Address,
        token: Address,
        amount: i128,
        interval: u32,
        max_executions: u32,
    ) -> u32 {
        recurring::setup_recurring(&e, payer, payee, token, amount, interval, max_executions)
    }

    fn execute_recurring(e: Env, recurring_id: u32) {
        recurring::execute_recurring(&e, recurring_id)
    }

    fn get_recurring_history(e: Env, recurring_id: u32) -> Vec<RecurringPayment> {
        recurring::get_recurring_history(e, recurring_id)
    }

    fn is_recurring_active(e: Env, recurring_id: u32) -> bool {
        match e.storage().persistent().get::<DataKey, recurring::RecurringRecord>(&DataKey::Recurring(recurring_id))
        {
            Some(record) => record.active,
            None => false,
        }
    }

    fn get_escrows_batch(e: Env, escrow_ids: Vec<u32>) -> Vec<Option<escrow::EscrowRecord>> {
        escrow::get_escrows_batch(e, escrow_ids)
    }

    fn get_escrow_age(e: Env, escrow_id: u32) -> u32 {
        escrow::get_escrow_age(e, escrow_id)
    }

    fn create_multi_escrow(
        e: Env,
        depositor: Address,
        recipients: Vec<(Address, i128)>,
        token: Address,
        expiry_ledger: u32,
    ) -> u32 {
        // Enforce that total distributed amount values are checked within sub-module contexts
        multi_escrow::create_multi_escrow(e, depositor, recipients, token, expiry_ledger)
    }

    fn release_multi_escrow(e: Env, caller: Address, multi_escrow_id: u32) {
        multi_escrow::release_multi_escrow(e, caller, multi_escrow_id)
    }

    fn refund_multi_escrow(e: Env, caller: Address, multi_escrow_id: u32) {
        multi_escrow::refund_multi_escrow(e, caller, multi_escrow_id)
    }

    fn ticket_escrow(
        e: Env,
        buyer: Address,
        organizer: Address,
        token: Address,
        ticket_price: i128,
        event_ledger: u32,
        ticket_ref: Bytes,
    ) -> u32 {
        buyer.require_auth();
        require_positive_amount(ticket_price);
        
        escrow::create_escrow(
            e,
            buyer,
            organizer,
            token,
            ticket_price,
            event_ledger + 100,
            ticket_ref,
        )
    }

    fn revenue_split(
        e: Env,
        sender: Address,
        organizer: Address,
        organizer_bps: u32,
        artist: Address,
        artist_bps: u32,
        platform: Address,
        token: Address,
        total_amount: i128,
        event_ledger: u32,
    ) -> u32 {
        sender.require_auth();
        require_positive_amount(total_amount);
        
        assert!(organizer_bps + artist_bps < 10_000, "invalid basis points");
        let _platform_bps = 10_000 - organizer_bps - artist_bps;
        let organizer_amt = total_amount * organizer_bps as i128 / 10_000;
        let artist_amt = total_amount * artist_bps as i128 / 10_000;
        let platform_amt = total_amount - organizer_amt - artist_amt;

        let recipients = Vec::from_array(
            &e,
            [
                (organizer, organizer_amt),
                (artist, artist_amt),
                (platform, platform_amt),
            ],
        );
        let split_id = multi_escrow::create_multi_escrow(
            e.clone(),
            sender.clone(),
            recipients,
            token,
            event_ledger + 100,
        );
        multi_escrow::release_multi_escrow(e, sender, split_id);
        split_id
    }

    fn approve(e: Env, from: Address, spender: Address, amount: i128, expiration_ledger: u32) {
        from.require_auth();
        require_positive_amount(amount);
        allowance::create_allowance(&e, &from, &spender, amount, expiration_ledger);
    }

    fn transfer_from(e: Env, spender: Address, from: Address, _to: Address, amount: i128) {
        spender.require_auth();
        require_positive_amount(amount);
        allowance::spend_allowance(&e, &from, &spender, amount);
        // Implement the actual token transfer logic here
    }

    // ── #451: Admin ownership ────────────────────────────────────────────────

    fn transfer_ownership(e: Env, new_admin: Address) {
        admin::transfer_ownership(&e, &new_admin)
    }

    fn accept_admin(e: Env, new_admin: Address) {
        admin::accept_admin(&e, &new_admin)
    }

    fn admin_active_after_ledger(e: Env) -> u32 {
        admin::admin_active_after_ledger(&e)
    }

    // ── #452: Depositor escrowed value ───────────────────────────────────────

    fn escrowed_value_for_depositor(e: Env, depositor: Address) -> i128 {
        escrow::escrowed_value_for_depositor(&e, &depositor)
    }

    // ── Pause ─────────────────────────────────────────────────────────────────

    fn set_paused(e: Env, admin: Address, paused: bool) {
        crate::pause::set_paused(&e, &admin, paused);
    }

    fn is_paused(e: Env) -> bool {
        e.storage().persistent().get::<_, bool>(&DataKey::Paused).unwrap_or(false)
    }

    // ── #453: Resolver stats ─────────────────────────────────────────────────

    fn resolver_stats(e: Env, resolver: Address) -> ResolverStats {
        dispute::get_resolver_stats(&e, &resolver)
    }

    // ── #454: Protocol fee stats ─────────────────────────────────────────────

    fn protocol_fee_stats(e: Env) -> (u32, Address, i128) {
        let fee_bps: u32 = e.storage().persistent().get(&DataKey::FeeBps).unwrap_or(0);
        let treasury: Address = e
            .storage()
            .persistent()
            .get(&DataKey::TreasuryAddress)
            .unwrap_or_else(|| {
                e.storage()
                    .persistent()
                    .get(&DataKey::Admin)
                    .expect("admin not set")
            });
        let total_collected: i128 = e
            .storage()
            .persistent()
            .get(&DataKey::TotalFeesCollected)
            .unwrap_or(0);
        (fee_bps, treasury, total_collected)
    }

    fn emergency_withdraw(e: Env, admin: Address, recipient: Address, token: Address, amount: i128) {
        // Verify caller is admin and authenticated
        admin::check_admin(&e, &admin);

        // Get contract's current balance of the token
        let token_client = soroban_sdk::token::Client::new(&e, &token);
        let contract_balance = token_client.balance(&e.current_contract_address());

        // Get total escrowed value (locked funds that cannot be touched)
        let total_escrowed = escrow::get_escrowed_total(&e);

        // Verify we're not withdrawing more than the stranded funds (contract balance - escrowed funds)
        assert!(amount <= contract_balance - total_escrowed, "Insufficient non-escrowed funds");
        assert!(amount > 0, "Amount must be positive");

        // Transfer the amount from the contract to the recipient
        token_client.transfer(&e.current_contract_address(), &recipient, &amount);

        // Emit the emergency withdrawal event
        e.events().publish(
            (soroban_sdk::symbol_short!("em_wdraw"), admin, recipient),
            amount,
        );
    }

    fn amend_recurring(e: Env, caller: Address, recurring_id: u32, new_amount: i128, new_interval: u32) {
        recurring::amend_recurring(&e, &caller, recurring_id, new_amount, new_interval)
    }

    fn escrow_between(e: Env, addr1: Address, addr2: Address) -> u32 {
        escrow::escrow_between(e, addr1, addr2)
    }

    fn cancel_recurring_batch(e: Env, caller: Address, recurring_ids: Vec<u32>) {
        recurring::cancel_recurring_batch(&e, &caller, recurring_ids)
    }

    fn cancel_split(e: Env, caller: Address, split_id: u32) {
        crate::splitter::cancel_split(e, caller, split_id)
    }

    fn topup_escrow(e: Env, depositor: Address, escrow_id: u32, amount: i128) {
        escrow::topup_escrow(e, depositor, escrow_id, amount)
    }

        e.events().publish(
            (soroban_sdk::symbol_short!("vst_crt"), admin, holder),
            (id, amount, vesting_ledger),
        );

        id
    }

    fn claim_vesting(e: Env, holder: Address, vesting_id: u32) {
        holder.require_auth();

        let mut record: VestingRecord = e
            .storage()
            .persistent()
            .get(&DataKey::Vesting(vesting_id))
            .expect("vesting record not found");

        assert!(record.holder == holder, "not the vesting holder");
        assert!(!record.claimed, "vesting already claimed");
        assert!(
            e.ledger().sequence() >= record.vesting_ledger,
            "vesting period not yet reached"
        );

    fn raise_dispute(e: Env, caller: Address, escrow_id: u32) {
        let max_disputes: u32 = e.storage().persistent().get(&DataKey::MaxDisputes).unwrap_or(3);
        let current_count: u32 = e.storage().persistent().get(&DataKey::DisputeCount(escrow_id)).unwrap_or(0);
        assert!(current_count < max_disputes, "maximum dispute count exceeded");
        e.storage().persistent().set(&DataKey::DisputeCount(escrow_id), &(current_count + 1));
        dispute::raise_dispute(&e, &caller, escrow_id)
    }

    fn version(e: Env) -> soroban_sdk::String {
        e.storage().persistent().get(&DataKey::Version).unwrap_or(String::from_str(&e, "1.0.0"))
    }

    fn contract_summary(e: Env) -> ContractSummary {
        let admin: Address = e.storage().persistent().get(&DataKey::Admin).expect("admin not set");
        let total_supply: i128 = e.storage().persistent().get(&DataKey::TotalSupply).unwrap_or(0);
        let escrow_count: u32 = e.storage().persistent().get(&DataKey::EscrowCount).unwrap_or(0);
        let total_value_locked: i128 = e.storage().persistent().get(&DataKey::EscrowValueLocked).unwrap_or(0);
        ContractSummary { admin, total_supply, escrow_count, total_value_locked }
    }

    fn spendable_balance(e: Env, account: Address) -> i128 {
        balance::spendable_balance(&e, &account)
    }

    fn get_vesting_by_holder(e: Env, holder: Address) -> Vec<u32> {
        e.storage()
            .persistent()
            .get(&DataKey::HolderVestings(holder))
            .unwrap_or_else(|| Vec::new(&e))
    }

    // ── #572: Split-to-escrow ────────────────────────────────────────────────

    fn split_to_escrow(
        e: Env,
        sender: Address,
        recipients: Vec<(Address, u32)>,
        token: Address,
        total_amount: i128,
        expiry_ledger: u32,
    ) -> Vec<u32> {
        sender.require_auth();
        require_positive_amount(total_amount);

    fn burn_from(e: Env, spender: Address, from: Address, amount: i128) {
        balance::burn_from(&e, &spender, &from, amount)
    }

    fn transfer_with_memo(e: Env, from: Address, to: Address, amount: i128, memo: Bytes) {
        from.require_auth();
        assert!(amount > 0, "amount must be positive");
        assert!(memo.len() <= 64, "memo cannot exceed 64 bytes");
        whitelist::check(&e, &from, &to);
        let token_client = soroban_sdk::token::Client::new(&e, &e.current_contract_address());
        token_client.transfer(&from, &to, &amount);
        e.events().publish((soroban_sdk::symbol_short!("transfer"), from, to), (amount, memo));
    }

        // Validate BPS sum equals 10000 and check for duplicates
        let mut total_bps: u32 = 0;
        for i in 0..recipients.len() {
            let (addr, bps) = recipients.get(i).unwrap();
            assert!(bps > 0, "recipient share_bps cannot be zero");
            total_bps += bps;
            // Check for duplicate addresses
            for j in (i + 1)..recipients.len() {
                let (other_addr, _) = recipients.get(j).unwrap();
                assert!(addr != other_addr, "duplicate recipient address");
            }
        }
        assert!(total_bps == 10000, "total basis points must equal 10000");

        // Pull all tokens from sender into the contract once
        let token_client = token::Client::new(&e, &token);
        token_client.transfer(&sender, &e.current_contract_address(), &total_amount);

        let mut escrow_ids = Vec::new(&e);
        let mut remaining = total_amount;
        let len = recipients.len();
        let empty_memo = Bytes::new(&e);

        for i in 0..len {
            let (address, bps) = recipients.get(i).unwrap();
            let share = if i == len - 1 {
                remaining // last recipient gets all remaining dust
            } else {
                total_amount * bps as i128 / 10000
            };
            remaining = remaining.checked_sub(share).expect("split remaining underflow");

            // Use internal batch helper to skip auth + rate-limit
            let escrow_id = escrow::create_escrow_batch(
                &e,
                &sender,
                &address,
                &token,
                share,
                expiry_ledger,
                &empty_memo,
            );
            escrow_ids.push_back(escrow_id);
        }

        e.events().publish(
            (soroban_sdk::symbol_short!("split_esc"), sender),
            total_amount,
        );

        escrow_ids
    }

    // ── #573: Airdrop ────────────────────────────────────────────────────────

    fn airdrop(e: Env, admin: Address, token: Address, total_amount: i128) {
        admin::check_admin(&e, &admin);
        require_positive_amount(total_amount);

        let holder_count: u32 = e
            .storage()
            .persistent()
            .get(&DataKey::HolderCount)
            .unwrap_or(0);
        assert!(holder_count > 0, "no holders to airdrop to");

        let token_client = token::Client::new(&e, &token);
        let admin_balance = token_client.balance(&admin);
        assert!(admin_balance >= total_amount, "insufficient admin balance");

        // Transfer total airdrop amount from admin to contract
        token_client.transfer(&admin, &e.current_contract_address(), &total_amount);

        let holders: Vec<Address> = e
            .storage()
            .persistent()
            .get(&DataKey::HolderSet)
            .unwrap_or_else(|| Vec::new(&e));

        // Calculate total held by eligible (non-frozen, positive balance) holders
        let mut total_held: i128 = 0;
        for i in 0..holders.len() {
            let holder = holders.get(i).unwrap();
            let frozen: bool = e
                .storage()
                .persistent()
                .get(&DataKey::Frozen(holder.clone()))
                .unwrap_or(false);
            if !frozen {
                let balance = token_client.balance(&holder);
                if balance > 0 {
                    total_held += balance;
                }
            }
        }

        assert!(total_held > 0, "no eligible holders with positive balance");

        let mut distributed: i128 = 0;
        for i in 0..holders.len() {
            let holder = holders.get(i).unwrap();
            let frozen: bool = e
                .storage()
                .persistent()
                .get(&DataKey::Frozen(holder.clone()))
                .unwrap_or(false);
            if !frozen {
                let balance = token_client.balance(&holder);
                if balance > 0 {
                    let share = balance * total_amount / total_held;
                    if share > 0 {
                        token_client.transfer(
                            &e.current_contract_address(),
                            &holder,
                            &share,
                        );
                        distributed = distributed.checked_add(share)
                            .expect("airdrop distributed overflow");
                    }
                }
            }
        }

        // Return any remainder to admin
        let remainder = total_amount.checked_sub(distributed)
            .expect("airdrop remainder underflow");
        if remainder > 0 {
            token_client.transfer(
                &e.current_contract_address(),
                &admin,
                &remainder,
            );
        }

        e.events().publish(
            (soroban_sdk::symbol_short!("airdrop"), admin),
            total_amount,
        );
    }

    // ── #574: Permit batch ───────────────────────────────────────────────────

    fn permit_batch(
        e: Env,
        owner: Address,
        approvals: Vec<(Address, i128, u32)>,
        nonce: u64,
        public_key: BytesN<32>,
        signature: BytesN<64>,
    ) {
        permit::permit_batch(&e, owner, approvals, nonce, public_key, signature)
    }

    fn cancel_recurring_batch(e: Env, caller: Address, recurring_ids: Vec<u32>) {
        recurring::cancel_recurring_batch(&e, &caller, recurring_ids)
    }
}