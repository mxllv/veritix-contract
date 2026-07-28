//! Admin module.
//! Owns admin identity storage, authorization checks, and rotation events.
//! `check_admin` requires signer auth first, then identity match, to prevent spoofed caller paths.

use soroban_sdk::{symbol_short, Address, Env};

use crate::storage_types::{bump_instance, DataKey};

// --- Core admin storage helpers ---

pub fn read_admin(e: &Env) -> Address {
    bump_instance(e);
    e.storage().instance().get(&DataKey::Admin).unwrap()
}

pub fn write_admin(e: &Env, id: &Address) {
    bump_instance(e);
    e.storage().instance().set(&DataKey::Admin, id);
}

pub fn has_admin(e: &Env) -> bool {
    e.storage().instance().has(&DataKey::Admin)
}

/// Verifies that `admin` is the current admin and has authorized the call.
pub fn check_admin(e: &Env, admin: &Address) {
    admin.require_auth();
    let stored = read_admin(e);
    if admin != &stored {
        panic!("not authorized: caller is not the admin");
    }
}

pub fn read_clawback_cosigner(e: &Env) -> Option<Address> {
    e.storage().instance().get(&DataKey::ClawbackCoSigner)
}

pub fn write_clawback_cosigner(e: &Env, cosigner: &Address) {
    bump_instance(e);
    e.storage().instance().set(&DataKey::ClawbackCoSigner, cosigner);
}

pub fn read_pending_admin(e: &Env) -> Option<Address> {
    e.storage().instance().get(&DataKey::PendingAdmin)
}

pub fn propose_admin(e: &Env, new_admin: &Address) {
    let current_admin = read_admin(e);
    current_admin.require_auth();
    bump_instance(e);
    e.storage().instance().set(&DataKey::PendingAdmin, new_admin);
}

pub fn accept_admin(e: &Env) {
    let pending: Address = e.storage().instance().get(&DataKey::PendingAdmin).expect("no pending admin");
    pending.require_auth();
    let old = read_admin(e);
    write_admin(e, &pending);
    e.storage().instance().remove(&DataKey::PendingAdmin);
    e.events().publish(
        (symbol_short!("admin_set"), old),
        pending,
    );
}



    fn create_initialized_client(env: &Env) -> (Address, VeritixTokenClient<'_>) {
        let contract_id = env.register_contract(None, VeritixToken);
        let client = VeritixTokenClient::new(env, &contract_id);
        let admin = Address::generate(env);
        client.initialize(
            &admin,
            &String::from_str(env, "Veritix"),
            &String::from_str(env, "VTX"),
            &7u32,
        );
        (admin, client)
    }

    // --- test_initialize_sets_admin ---

    #[test]
    fn test_initialize_sets_admin() {
        let env = setup_env();
        let (admin, client) = create_initialized_client(&env);
        assert_eq!(client.admin(), admin);
    }

    // --- test_has_admin_false_before_initialize ---

    #[test]
    fn test_has_admin_false_before_initialize() {
        let env = Env::default();
        let contract_id = env.register_contract(None, VeritixToken);
        env.as_contract(&contract_id, || {
            assert!(!has_admin(&env));
        });
    }

    // --- test_transfer_admin_updates_stored_admin ---

    #[test]
    fn test_transfer_admin_updates_stored_admin() {
        let env = setup_env();
        let (admin, client) = create_initialized_client(&env);
        let new_admin = Address::generate(&env);

        client.set_admin(&new_admin);

        assert_eq!(client.admin(), new_admin);
        assert_ne!(client.admin(), admin);
    }

    // --- test_transfer_admin_unauthorized_panics ---

    #[test]
    #[should_panic]
    fn test_transfer_admin_unauthorized_panics() {
        let env = Env::default();
        let admin = Address::generate(&env);
        let new_admin = Address::generate(&env);

        write_admin(&env, &admin);

        // No mock auths — transfer_admin requires the current admin to authorize
        env.set_auths(&[]);
        transfer_admin(&env, new_admin);
    }

    // --- test_transfer_admin_emits_event ---

    #[test]
    fn test_transfer_admin_emits_event() {
        let env = setup_env();
        let (_admin, client) = create_initialized_client(&env);
        let new_admin = Address::generate(&env);

        // Clear any initialization events
        let _ = env.events().all();

        client.set_admin(&new_admin);

        let events = env.events().all();
        assert!(!events.is_empty(), "expected at least one event after set_admin");

        // The admin_set event topics: (symbol_short!("admin_set"), current_admin)
        // data: new_admin
        let event = events.first().unwrap();
        assert_eq!(event.1.len(), 2);
    }

    // --- test_check_admin_wrong_address_panics ---

    #[test]
    #[should_panic]
    fn test_check_admin_wrong_address_panics() {
        let env = setup_env();
        let contract_id = env.register_contract(None, VeritixToken);
        let admin = Address::generate(&env);
        let impostor = Address::generate(&env);

        env.as_contract(&contract_id, || {
            write_admin(&env, &admin);
            // check_admin with a non-admin address should panic
            crate::admin::check_admin(&env, &impostor);
        });
    }

    // --- test_transfer_admin (basic rotation) ---

    #[test]
    fn test_transfer_admin() {
        let env = setup_env();
        let (admin, client) = create_initialized_client(&env);
        let new_admin = Address::generate(&env);

        client.set_admin(&new_admin);

        assert_eq!(client.admin(), new_admin);
        assert_ne!(client.admin(), admin);
    }

    // --- test_transfer_admin_to_same_address ---

    #[test]
    fn test_transfer_admin_to_same_address() {
        let env = setup_env();
        let (admin, client) = create_initialized_client(&env);

        client.set_admin(&admin);
        assert_eq!(client.admin(), admin);
    }

    #[test]
    fn test_admin_info_tracks_admin_rotation() {
        let env = setup_env();
        let (_admin, client) = create_initialized_client(&env);
        let new_admin = Address::generate(&env);
        let before = client.admin_info();
        assert_eq!(before.paused, false);
        client.set_admin(&new_admin);
        let after = client.admin_info();
        assert_eq!(after.admin, new_admin);
        assert_eq!(after.paused, false);
    }


/// Rotates the stored admin to `new_admin`. Must be called by the current admin.
pub fn transfer_admin(e: &Env, new_admin: Address) {
    let current_admin = read_admin(e);
    current_admin.require_auth();
    write_admin(e, &new_admin);
    e.events().publish(
        (symbol_short!("admin_set"), current_admin),
        new_admin,
    );
}


    fn create_initialized_client(env: &Env) -> (Address, VeritixTokenClient<'_>) {
        let contract_id = env.register_contract(None, VeritixToken);
        let client = VeritixTokenClient::new(env, &contract_id);
        let admin = Address::generate(env);
        client.initialize(
            &admin,
            &String::from_str(env, "Veritix"),
            &String::from_str(env, "VTX"),
            &7u32,
        );
        (admin, client)
    }

    // --- test_initialize_sets_admin ---

    #[test]
    fn test_initialize_sets_admin() {
        let env = setup_env();
        let (admin, client) = create_initialized_client(&env);
        assert_eq!(client.admin(), admin);
    }

    // --- test_has_admin_false_before_initialize ---

    #[test]
    fn test_has_admin_false_before_initialize() {
        let env = Env::default();
        let contract_id = env.register_contract(None, VeritixToken);
        env.as_contract(&contract_id, || {
            assert!(!has_admin(&env));
        });
    }

    // --- test_transfer_admin_updates_stored_admin ---

    #[test]
    fn test_transfer_admin_updates_stored_admin() {
        let env = setup_env();
        let (admin, client) = create_initialized_client(&env);
        let new_admin = Address::generate(&env);

        client.set_admin(&new_admin);

        assert_eq!(client.admin(), new_admin);
        assert_ne!(client.admin(), admin);
    }

    // --- test_transfer_admin_unauthorized_panics ---

    #[test]
    #[should_panic]
    fn test_transfer_admin_unauthorized_panics() {
        let env = Env::default();
        let admin = Address::generate(&env);
        let new_admin = Address::generate(&env);

        write_admin(&env, &admin);

        // No mock auths — transfer_admin requires the current admin to authorize
        env.set_auths(&[]);
        transfer_admin(&env, new_admin);
    }

    // --- test_transfer_admin_emits_event ---

    #[test]
    fn test_transfer_admin_emits_event() {
        let env = setup_env();
        let (_admin, client) = create_initialized_client(&env);
        let new_admin = Address::generate(&env);

        // Clear any initialization events
        let _ = env.events().all();

        client.set_admin(&new_admin);

        let events = env.events().all();
        assert!(!events.is_empty(), "expected at least one event after set_admin");

        // The admin_set event topics: (symbol_short!("admin_set"), current_admin)
        // data: new_admin
        let event = events.first().unwrap();
        assert_eq!(event.1.len(), 2);
    }

    // --- test_check_admin_wrong_address_panics ---

    #[test]
    #[should_panic]
    fn test_check_admin_wrong_address_panics() {
        let env = setup_env();
        let contract_id = env.register_contract(None, VeritixToken);
        let admin = Address::generate(&env);
        let impostor = Address::generate(&env);

        env.as_contract(&contract_id, || {
            write_admin(&env, &admin);
            // check_admin with a non-admin address should panic
            crate::admin::check_admin(&env, &impostor);
        });
    }

    // --- test_transfer_admin (basic rotation) ---

    #[test]
    fn test_transfer_admin() {
        let env = setup_env();
        let (admin, client) = create_initialized_client(&env);
        let new_admin = Address::generate(&env);

        client.set_admin(&new_admin);

        assert_eq!(client.admin(), new_admin);
        assert_ne!(client.admin(), admin);
    }

    // --- test_transfer_admin_to_same_address ---

    #[test]
    fn test_transfer_admin_to_same_address() {
        let env = setup_env();
        let (admin, client) = create_initialized_client(&env);

        client.set_admin(&admin);
        assert_eq!(client.admin(), admin);
    }

    #[test]
    fn test_admin_info_tracks_admin_rotation() {
        let env = setup_env();
        let (_admin, client) = create_initialized_client(&env);
        let new_admin = Address::generate(&env);
        let before = client.admin_info();
        assert_eq!(before.paused, false);
        client.set_admin(&new_admin);
        let after = client.admin_info();
        assert_eq!(after.admin, new_admin);
        assert_eq!(after.paused, false);
    }
