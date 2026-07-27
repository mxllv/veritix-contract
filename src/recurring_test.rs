#![cfg(test)]

use soroban_sdk::{Env, testutils::Address as _, testutils::Ledger as _};
use crate::recurring::{record_recurring_execution, get_recurring_history};

#[test]
fn test_recurring_history_grows() {
    let e = Env::default();
    e.mock_all_auths();

    let caller = soroban_sdk::Address::generate(&e);
    let recurring_id = 1;
    let amount = 500;
    
    record_recurring_execution(e.clone(), caller.clone(), recurring_id, amount);
    
    let history = get_recurring_history(e.clone(), recurring_id);
    assert_eq!(history.len(), 1);
    assert_eq!(history.get(0).unwrap().amount, amount);
    assert_eq!(history.get(0).unwrap().execution_ledger, e.ledger().sequence());
    
    // Simulate next execution
    e.ledger().with_mut(|l| l.sequence_number = e.ledger().sequence() + 10);
    record_recurring_execution(e.clone(), caller.clone(), recurring_id, amount);
    
    let history = get_recurring_history(e.clone(), recurring_id);
    assert_eq!(history.len(), 2);
    assert_eq!(history.get(1).unwrap().amount, amount);
    assert_eq!(history.get(1).unwrap().execution_ledger, e.ledger().sequence());
}
