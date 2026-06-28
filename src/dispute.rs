use soroban_sdk::{contracttype, Address, Env};
use crate::storage_types::DataKey;
use crate::escrow::{EscrowRecord, load_record};

#[contracttype]
#[derive(Clone)]
pub struct DisputeRecord {
    pub escrow_id: u32,
    pub opener: Address,
    pub reason: soroban_sdk::Bytes,
}

pub fn open_dispute(e: &Env, caller: Address, escrow_id: u32, reason: soroban_sdk::Bytes) {
    caller.require_auth();
    let escrow = load_record(e, escrow_id);
    // #427: cannot open dispute on settled escrow
    if escrow.released || escrow.refunded {
        panic!("InvalidState: cannot open a dispute on a settled escrow");
    }
    assert!(
        caller == escrow.depositor || caller == escrow.beneficiary,
        "only escrow parties can open a dispute"
    );
    assert!(
        !e.storage().persistent().has(&DataKey::EscrowDispute(escrow_id)),
        "dispute already open"
    );
    let dispute = DisputeRecord { escrow_id, opener: caller, reason };
    e.storage().persistent().set(&DataKey::EscrowDispute(escrow_id), &dispute);
}

pub fn resolve_dispute(e: &Env, admin: Address, escrow_id: u32, release_to_beneficiary: bool) {
    check_admin_for_dispute(e, &admin);
    admin.require_auth();
    assert!(
        e.storage().persistent().has(&DataKey::EscrowDispute(escrow_id)),
        "no open dispute"
    );
    e.storage().persistent().remove(&DataKey::EscrowDispute(escrow_id));
    // Caller resolves the escrow externally via release_escrow or refund_escrow
    let _ = release_to_beneficiary;
}

fn check_admin_for_dispute(e: &Env, caller: &Address) {
    let admin: Address = e.storage().persistent().get(&DataKey::Admin).expect("admin not set");
    if admin != *caller {
        panic!("Unauthorized");
    }
}
