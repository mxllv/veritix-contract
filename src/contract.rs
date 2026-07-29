use soroban_sdk::{contract, contractimpl, Address, Bytes, Env, Vec};
use crate::{escrow, multi_escrow, allowance, admin, dispute, recurring};
use crate::storage_types::{DataKey, RecurringPayment, ResolverStats};
use crate::validation::require_positive_amount;

pub trait VeriTixPayTrait {
    fn initialize(e: Env, admin: Address);

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

    // ── #453: Resolver stats ─────────────────────────────────────────────────
    fn resolver_stats(e: Env, resolver: Address) -> ResolverStats;

    // ── #454: Protocol fee stats ─────────────────────────────────────────────
    fn protocol_fee_stats(e: Env) -> (u32, Address, i128);
    fn emergency_withdraw(e: Env, admin: Address, recipient: Address, token: Address, amount: i128);

    fn trigger_auto_release(e: Env, escrow_id: u32);
    fn escrow_between(e: Env, addr1: Address, addr2: Address) -> u32;
    fn cancel_recurring_batch(e: Env, caller: Address, recurring_ids: Vec<u32>);
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
        escrow::release_escrow(e, caller, escrow_id)
    }

    fn release_partial_escrow(e: Env, caller: Address, escrow_id: u32, amount: i128) {
        require_positive_amount(amount);
        escrow::release_partial_escrow(e, caller, escrow_id, amount)
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
        match e.storage().persistent().get::<Option<escrow::EscrowRecord>>(&DataKey::Escrow(escrow_id))
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

    fn raise_dispute(e: Env, caller: Address, escrow_id: u32) {
        dispute::raise_dispute(&e, &caller, escrow_id)
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
        match e.storage().persistent().get::<Option<recurring::RecurringRecord>>(&DataKey::Recurring(recurring_id))
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
            (soroban_sdk::symbol_short!("emer_wdraw"), admin, recipient),
            amount,
        );
    }

    fn trigger_auto_release(e: Env, escrow_id: u32) {
        escrow::trigger_auto_release(e, escrow_id)
    }

    fn escrow_between(e: Env, addr1: Address, addr2: Address) -> u32 {
        escrow::escrow_between(e, addr1, addr2)
    }

    fn cancel_recurring_batch(e: Env, caller: Address, recurring_ids: Vec<u32>) {
        recurring::cancel_recurring_batch(&e, &caller, recurring_ids)
    }
}