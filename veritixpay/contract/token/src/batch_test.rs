#[cfg(test)]
mod batch_tests {
    use soroban_sdk::{testutils::Address as _, Address, Env, Vec};
    use crate::allowance::write_allowance;
    use crate::balance::{increase_supply, read_balance, read_total_supply, receive_balance};
    use crate::batch::{burn_from_batch, mint_batch, transfer_batch, BatchEntry};
    use crate::contract::VeritixToken;
    use crate::freeze::is_frozen;

    fn setup_env() -> Env {
        let e = Env::default();
        e.mock_all_auths();
        e
    }

    #[test]
    fn test_mint_batch_credits_all_recipients() {
        let e = setup_env();
        let cid = e.register_contract(None, VeritixToken);
        let admin = Address::generate(&e);
        let r1 = Address::generate(&e);
        let r2 = Address::generate(&e);
        e.as_contract(&cid, || {
            crate::admin::write_admin(&e, &admin);
            let mut recs: Vec<BatchEntry> = Vec::new(&e);
            recs.push_back(BatchEntry { address: r1.clone(), amount: 500 });
            recs.push_back(BatchEntry { address: r2.clone(), amount: 300 });
            mint_batch(&e, admin.clone(), recs);
            assert_eq!(read_balance(&e, r1.clone()), 500);
            assert_eq!(read_balance(&e, r2.clone()), 300);
        });
    }


    
pub fn read_allowance(e: &Env, from: Address, spender: Address) -> AllowanceValue {
    let key = DataKey::Allowance(AllowanceDataKey {
        from: from.clone(),
        spender: spender.clone(),
    });

    if let Some(allowance) = e
        .storage()
        .persistent()
        .get::<DataKey, AllowanceValue>(&key)
    {
        // Equal-to-current-ledger approvals are still valid for the current ledger.
        // They become expired only once the sequence advances past expiration_ledger.
        if allowance.expiration_ledger < e.ledger().sequence() {
            // Prune expired entry from storage
            e.storage().persistent().remove(&key);
            AllowanceValue {
                amount: 0,
                expiration_ledger: allowance.expiration_ledger,
            }
        } else {
            // Extend TTL on active allowance read
            e.storage().persistent().extend_ttl(
                &key,
                ALLOWANCE_LIFETIME_THRESHOLD,
                ALLOWANCE_BUMP_AMOUNT,
            );
            allowance
        }
    } else {
        AllowanceValue {
            amount: 0,
            expiration_ledger: 0,
        }
    }
}

fn write_owner_allowance_index(e: &Env, from: &Address, spender: &Address, add: bool) {
    let owner_key = DataKey::OwnerAllowances(from.clone());
    let mut spenders: Vec<Address> = e.storage().persistent().get(&owner_key).unwrap_or_else(|| Vec::new(e));
    if add {
        let mut exists = false;
        for i in 0..spenders.len() {
            if spenders.get(i).unwrap() == *spender {
                exists = true;
                break;
            }
        }
        if !exists {
            spenders.push_back(spender.clone());
        }
    } else {
        let mut updated = Vec::new(e);
        for i in 0..spenders.len() {
            let addr = spenders.get(i).unwrap();
            if addr != *spender {
                updated.push_back(addr);
            }
        }
        spenders = updated;
    }
    e.storage().persistent().set(&owner_key, &spenders);
    e.storage().persistent().extend_ttl(&owner_key, PERSISTENT_LIFETIME_THRESHOLD, PERSISTENT_BUMP_AMOUNT);
}

pub fn write_allowance(
    e: &Env,
    from: Address,
    spender: Address,
    amount: i128,
    expiration_ledger: u32,
) {
    require_non_negative_amount(amount);
    require_current_or_future_ledger(e.ledger().sequence(), expiration_ledger);

    let key = DataKey::Allowance(AllowanceDataKey {
        from: from.clone(),
        spender: spender.clone(),
    });

    let index_key = DataKey::SpenderAllowances(spender.clone());
    let mut spenders_from: Vec<Address> = e
        .storage()
        .persistent()
        .get(&index_key)
        .unwrap_or_else(|| Vec::new(e));

    if amount == 0 {
        e.storage().persistent().remove(&key);
        let mut updated = Vec::new(e);
        for i in 0..spenders_from.len() {
            let addr = spenders_from.get(i).unwrap();
            if addr != from {
                updated.push_back(addr);
            }
        }
        e.storage().persistent().set(&index_key, &updated);
        // Keep spender index alive for long-lived delegated payment lookups.
        e.storage().persistent().extend_ttl(
            &index_key,
            PERSISTENT_LIFETIME_THRESHOLD,
            PERSISTENT_BUMP_AMOUNT,
        );
        write_owner_allowance_index(e, &from, &spender, false);
    } else {
        let mut exists = false;
        for i in 0..spenders_from.len() {
            if spenders_from.get(i).unwrap() == from {
                exists = true;
                break;
            }
        }
        if !exists {
            spenders_from.push_back(from.clone());
            e.storage().persistent().set(&index_key, &spenders_from);
            // Keep spender index alive for long-lived delegated payment lookups.
            e.storage().persistent().extend_ttl(
                &index_key,
                PERSISTENT_LIFETIME_THRESHOLD,
                PERSISTENT_BUMP_AMOUNT,
            );
        }
        write_owner_allowance_index(e, &from, &spender, true);
        let allowance = AllowanceValue {
            amount,
            expiration_ledger,
        };
        e.storage().persistent().set(&key, &allowance);
        e.storage().persistent().extend_ttl(
            &key,
            ALLOWANCE_LIFETIME_THRESHOLD,
            ALLOWANCE_BUMP_AMOUNT,
        );
    }
}

    #[test]
    fn test_transfer_batch_distributes_correctly() {
        let e = setup_env();
        let cid = e.register_contract(None, VeritixToken);
        let from = Address::generate(&e);
        let r1 = Address::generate(&e);
        let r2 = Address::generate(&e);
        e.as_contract(&cid, || {
            crate::balance::receive_balance(&e, from.clone(), 1000);
            let mut recs: Vec<BatchEntry> = Vec::new(&e);
            recs.push_back(BatchEntry { address: r1.clone(), amount: 400 });
            recs.push_back(BatchEntry { address: r2.clone(), amount: 600 });
            transfer_batch(&e, from.clone(), recs);
            assert_eq!(read_balance(&e, r1.clone()), 400);
            assert_eq!(read_balance(&e, r2.clone()), 600);
            assert_eq!(read_balance(&e, from.clone()), 0);
        });
    }

    #[test]
    #[should_panic(expected = "BatchLimit")]
    fn test_mint_batch_rejects_over_50() {
        let e = setup_env();
        let cid = e.register_contract(None, VeritixToken);
        let admin = Address::generate(&e);
        e.as_contract(&cid, || {
            crate::admin::write_admin(&e, &admin);
            let mut recs: Vec<BatchEntry> = Vec::new(&e);
            for _ in 0..51 {
                recs.push_back(BatchEntry { address: Address::generate(&e), amount: 10 });
            }
            mint_batch(&e, admin.clone(), recs);
        });
    }

    #[test]
    fn test_burn_from_batch_reduces_balances_and_supply() {
        let e = setup_env();
        let cid = e.register_contract(None, VeritixToken);
        let admin = Address::generate(&e);
        let owner1 = Address::generate(&e);
        let owner2 = Address::generate(&e);
        let spender = Address::generate(&e);
        e.as_contract(&cid, || {
            crate::admin::write_admin(&e, &admin);
            receive_balance(&e, owner1.clone(), 1000);
            receive_balance(&e, owner2.clone(), 1000);
            increase_supply(&e, 2000);
            write_allowance(&e, owner1.clone(), spender.clone(), 500, e.ledger().sequence() + 1000);
            write_allowance(&e, owner2.clone(), spender.clone(), 500, e.ledger().sequence() + 1000);
            let mut targets: Vec<(Address, i128)> = Vec::new(&e);
            targets.push_back((owner1.clone(), 300));
            targets.push_back((owner2.clone(), 200));
            burn_from_batch(&e, spender.clone(), targets);
            assert_eq!(read_balance(&e, owner1.clone()), 700);
            assert_eq!(read_balance(&e, owner2.clone()), 800);
            assert_eq!(read_total_supply(&e), 1500);
        });
    }

    #[test]
    #[should_panic(expected = "batch too large")]
    fn test_burn_from_batch_rejects_over_50() {
        let e = setup_env();
        let cid = e.register_contract(None, VeritixToken);
        let admin = Address::generate(&e);
        let spender = Address::generate(&e);
        e.as_contract(&cid, || {
            crate::admin::write_admin(&e, &admin);
            let mut targets: Vec<(Address, i128)> = Vec::new(&e);
            for _ in 0..51 {
                targets.push_back((Address::generate(&e), 1));
            }
            burn_from_batch(&e, spender.clone(), targets);
        });
    }

    #[test]
    #[should_panic(expected = "BatchLimit")]
    fn test_transfer_batch_rejects_over_50() {
        let e = setup_env();
        let cid = e.register_contract(None, VeritixToken);
        let from = Address::generate(&e);
        e.as_contract(&cid, || {
            crate::balance::receive_balance(&e, from.clone(), 100_000);
            let mut recs: Vec<BatchEntry> = Vec::new(&e);
            for _ in 0..51 {
                recs.push_back(BatchEntry { address: Address::generate(&e), amount: 1 });
            }
            transfer_batch(&e, from.clone(), recs);
        });
    }

    // --- Issue #446: Batch atomicity tests ---

    // Verifies that mint_batch is atomic: if one entry has a zero amount (invalid),
    // the entire batch panics and NO balances are credited for any address.
    // Soroban's transaction model reverts all state changes on panic.
    #[test]
    fn test_mint_batch_partial_failure_reverts_all() {
        let e = setup_env();
        let cid = e.register_contract(None, VeritixToken);
        let admin = Address::generate(&e);
        let r1 = Address::generate(&e);
        let r2 = Address::generate(&e);
        let r3 = Address::generate(&e);

        // Capture pre-state before attempting the batch
        let (pre1, pre2, pre3) = e.as_contract(&cid, || {
            crate::admin::write_admin(&e, &admin);
            (
                read_balance(&e, r1.clone()),
                read_balance(&e, r2.clone()),
                read_balance(&e, r3.clone()),
            )
        });

        // The batch includes a zero-amount entry (r2) which must cause a panic,
        // reverting any credits applied to r1 before the panic.
        let panicked = std::panic::catch_unwind(|| {
            e.as_contract(&cid, || {
                let mut recs: Vec<BatchEntry> = Vec::new(&e);
                recs.push_back(BatchEntry { address: r1.clone(), amount: 100 });
                recs.push_back(BatchEntry { address: r2.clone(), amount: 0 }); // invalid
                recs.push_back(BatchEntry { address: r3.clone(), amount: 100 });
                mint_batch(&e, admin.clone(), recs);
            });
        });
        assert!(panicked.is_err(), "expected panic from zero-amount entry");

        // Post-state: no balances should have changed
        e.as_contract(&cid, || {
            assert_eq!(read_balance(&e, r1.clone()), pre1, "r1 balance must not change");
            assert_eq!(read_balance(&e, r2.clone()), pre2, "r2 balance must not change");
            assert_eq!(read_balance(&e, r3.clone()), pre3, "r3 balance must not change");
        });
    }

    // Verifies that clawback_batch is atomic: if one target has insufficient balance,
    // the batch panics and no clawbacks occur for any address.
    #[test]
    fn test_clawback_batch_insufficient_balance_reverts_all() {
        let e = setup_env();
        let cid = e.register_contract(None, VeritixToken);
        let admin = Address::generate(&e);
        let t1 = Address::generate(&e);
        let t2 = Address::generate(&e); // will have insufficient balance
        let t3 = Address::generate(&e);

        let (pre1, pre2, pre3) = e.as_contract(&cid, || {
            crate::admin::write_admin(&e, &admin);
            receive_balance(&e, t1.clone(), 200);
            increase_supply(&e, 200);
            // t2 has 0, t3 has 200
            receive_balance(&e, t3.clone(), 200);
            increase_supply(&e, 200);
            (
                read_balance(&e, t1.clone()),
                read_balance(&e, t2.clone()),
                read_balance(&e, t3.clone()),
            )
        });

        let panicked = std::panic::catch_unwind(|| {
            e.as_contract(&cid, || {
                let mut targets: Vec<(Address, i128)> = Vec::new(&e);
                targets.push_back((t1.clone(), 100));
                targets.push_back((t2.clone(), 100)); // insufficient — t2 has 0
                targets.push_back((t3.clone(), 100));
                crate::batch::clawback_batch(&e, admin.clone(), targets);
            });
        });
        assert!(panicked.is_err(), "expected panic from insufficient balance");

        e.as_contract(&cid, || {
            assert_eq!(read_balance(&e, t1.clone()), pre1, "t1 balance must not change");
            assert_eq!(read_balance(&e, t2.clone()), pre2, "t2 balance must not change");
            assert_eq!(read_balance(&e, t3.clone()), pre3, "t3 balance must not change");
        });
    }

    // Verifies that freeze_batch is atomic: if one address is already frozen,
    // the batch panics and no other addresses in the batch end up frozen.
    #[test]
    fn test_freeze_batch_already_frozen_reverts_all() {
        let e = setup_env();
        let cid = e.register_contract(None, VeritixToken);
        let admin = Address::generate(&e);
        let a1 = Address::generate(&e);
        let a2 = Address::generate(&e); // pre-frozen
        let a3 = Address::generate(&e);

        e.as_contract(&cid, || {
            crate::admin::write_admin(&e, &admin);
            // Pre-freeze a2 to trigger the AlreadyFrozen panic mid-batch
            crate::freeze::freeze_account(&e, admin.clone(), a2.clone());
        });

        let panicked = std::panic::catch_unwind(|| {
            e.as_contract(&cid, || {
                let mut targets: Vec<Address> = Vec::new(&e);
                targets.push_back(a1.clone());
                targets.push_back(a2.clone()); // already frozen — panics here
                targets.push_back(a3.clone());
                crate::batch::freeze_batch(&e, admin.clone(), targets);
            });
        });
        assert!(panicked.is_err(), "expected panic from already-frozen address");

        // a1 and a3 must NOT be frozen — batch was reverted
        e.as_contract(&cid, || {
            assert!(!is_frozen(&e, &a1), "a1 must not be frozen after revert");
            assert!(!is_frozen(&e, &a3), "a3 must not be frozen after revert");
        });
    }
}
