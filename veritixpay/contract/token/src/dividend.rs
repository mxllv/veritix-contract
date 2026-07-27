use crate::balance::{get_all_holders, read_balance, receive_balance, spend_balance};
use crate::storage_types::{
    increment_counter, read_persistent_record, write_persistent_record, DataKey,
};
use crate::validation::require_positive_amount;
use soroban_sdk::{contracttype, symbol_short, Address, Env};

#[contracttype]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct DividendRecord {
    pub id: u32,
    pub distributor: Address,
    pub total_amount: i128,
    pub distributed: bool,
}

/// Creates a dividend distribution record and locks the funds in the contract.
/// Requires distributor to have sufficient balance.
pub fn create_dividend(e: &Env, distributor: Address, total_amount: i128) -> u32 {
    require_positive_amount(total_amount);
    distributor.require_auth();

    let dividend_id = increment_counter(e, &DataKey::DividendCount);

    // Lock funds in the contract
    spend_balance(e, distributor.clone(), total_amount);
    receive_balance(e, e.current_contract_address(), total_amount);

    let record = DividendRecord {
        id: dividend_id,
        distributor,
        total_amount,
        distributed: false,
    };

    write_persistent_record(e, &DataKey::Dividend(dividend_id), &record);

    e.events().publish(
        (symbol_short!("dividend_created"), dividend_id),
        total_amount,
    );

    dividend_id
}

/// Distributes a proportional dividend to all current token holders based on their balance.
pub fn distribute_dividend(e: &Env, caller: Address, dividend_id: u32) {
    caller.require_auth();

    let mut record: DividendRecord = read_persistent_record(
        e,
        &DataKey::Dividend(dividend_id),
        "dividend record not found",
    );

    if record.distributor != caller {
        panic!("unauthorized");
    }
    if record.distributed {
        panic!("already distributed");
    }

    let holders = get_all_holders(e);

    if holders.is_empty() {
        panic!("no holders to distribute to");
    }

    // Calculate total supply from all holder balances
    let mut total_supply: i128 = 0;
    for holder in holders.iter() {
        total_supply = total_supply
            .checked_add(read_balance(e, holder.clone()))
            .expect("supply overflow");
    }

    if total_supply == 0 {
        panic!("total supply is zero");
    }

    let mut remaining_amount = record.total_amount;
    let len = holders.len();

    // Distribute proportionally
    for (i, holder) in holders.iter().enumerate() {
        let holder_balance = read_balance(e, holder.clone());

        let amount_to_send = if i == (len as usize - 1) {
            // Last holder gets everything left to avoid rounding dust
            remaining_amount
        } else {
            record
                .total_amount
                .checked_mul(holder_balance)
                .expect("dividend calculation overflow")
                / total_supply
        };

        // Transfer from contract to holder
        spend_balance(e, e.current_contract_address(), amount_to_send);
        receive_balance(e, holder.clone(), amount_to_send);

        remaining_amount = remaining_amount
            .checked_sub(amount_to_send)
            .expect("dividend remaining underflow");
    }

    // Mark distributed
    record.distributed = true;
    write_persistent_record(e, &DataKey::Dividend(dividend_id), &record);

    e.events().publish(
        (
            symbol_short!("dividend_distributed"),
            dividend_id,
            record.distributor,
        ),
        record.total_amount,
    );
}

pub fn get_dividend(e: &Env, dividend_id: u32) -> DividendRecord {
    read_persistent_record(
        e,
        &DataKey::Dividend(dividend_id),
        "dividend record not found",
    )
}
