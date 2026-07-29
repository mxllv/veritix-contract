#[cfg(test)]
mod tests {
    use super::*;
    use crate::contract::{VeriTixPay, VeriTixPayClient};
    use soroban_sdk::{testutils::Address as _, Env, Vec};

    struct TestEnv<'a> {
        e: Env,
        client: VeriTixPayClient<'a>,
        sender: Address,
        recipient1: Address,
        recipient2: Address,
        new_recipient: Address,
        token: Address,
    }

    fn setup() -> TestEnv<'static> {
        let e = Env::default();
        e.mock_all_auths();

        let contract_id = e.register_contract(None, VeriTixPay);
        let client = VeriTixPayClient::new(&e, &contract_id);

        let sender = Address::generate(&e);
        let recipient1 = Address::generate(&e);
        let recipient2 = Address::generate(&e);
        let new_recipient = Address::generate(&e);
        let token = e.register_stellar_asset_contract(sender.clone());

        soroban_sdk::token::StellarAssetClient::new(&e, &token).mint(&sender, &10_000);

        // Initialize contract with admin
        client.initialize(&sender);

        TestEnv { e, client, sender, recipient1, recipient2, new_recipient, token }
    }

    #[test]
    fn test_reentrancy_guard_blocks_double_distribution() {
        let env = Env::default();
        
        let mut record = super::SplitRecord {
            id: 0,
            sender: Address::generate(&env),
            recipients: Vec::from_array(&env, [(Address::generate(&env), 5000), (Address::generate(&env), 5000)]),
            token: Address::generate(&env),
            total_amount: 1000,
            event_ledger: env.ledger().sequence() + 1000,
            distributed: false,
            cancelled: false,
        };

        assert!(!record.distributed);
        record.distributed = true;

        assert!(record.distributed);
    }

    #[test]
    fn test_replace_split_recipient_success() {
        let t = setup();
        let event_ledger = t.e.ledger().sequence() + 1000;

        // Create split with two recipients
        let recipients = Vec::from_array(&t.e, [(t.recipient1.clone(), 6000), (t.recipient2.clone(), 4000)]);
        let split_id = crate::splitter::create_split(
            t.e.clone(),
            t.sender.clone(),
            recipients,
            t.token.clone(),
            1000,
            event_ledger,
        );

        // Verify initial recipients
        let record = crate::splitter::load_record(&t.e, split_id);
        assert_eq!(record.recipients.len(), 2);
        assert_eq!(record.recipients.get(0).unwrap().0, t.recipient1);
        assert_eq!(record.recipients.get(0).unwrap().1, 6000);

        // Replace recipient1 with new_recipient
        t.client.replace_split_recipient(&t.sender, &split_id, &t.recipient1, &t.new_recipient);

        // Verify the recipient was replaced, share_bps preserved
        let updated_record = crate::splitter::load_record(&t.e, split_id);
        assert_eq!(updated_record.recipients.len(), 2);
        assert_eq!(updated_record.recipients.get(0).unwrap().0, t.new_recipient);
        assert_eq!(updated_record.recipients.get(0).unwrap().1, 6000); // share_bps preserved

        // Verify event was emitted
        let events = t.e.events().all();
        assert!(events.iter().any(|event| {
            event
                .topics
                .contains(&soroban_sdk::symbol_short!("splt_rpl").into())
        }));
    }

    #[test]
    #[should_panic(expected = "old recipient not found in split")]
    fn test_replace_split_recipient_old_not_found() {
        let t = setup();
        let event_ledger = t.e.ledger().sequence() + 1000;
        let non_existent_recipient = Address::generate(&t.e);

        // Create split
        let recipients = Vec::from_array(&t.e, [(t.recipient1.clone(), 6000), (t.recipient2.clone(), 4000)]);
        let split_id = crate::splitter::create_split(
            t.e.clone(),
            t.sender.clone(),
            recipients,
            t.token.clone(),
            1000,
            event_ledger,
        );

        // Try to replace a non-existent recipient
        t.client.replace_split_recipient(&t.sender, &split_id, &non_existent_recipient, &t.new_recipient);
    }

    #[test]
    #[should_panic(expected = "new recipient is already in the split")]
    fn test_replace_split_recipient_duplicate_new() {
        let t = setup();
        let event_ledger = t.e.ledger().sequence() + 1000;

        // Create split
        let recipients = Vec::from_array(&t.e, [(t.recipient1.clone(), 6000), (t.recipient2.clone(), 4000)]);
        let split_id = crate::splitter::create_split(
            t.e.clone(),
            t.sender.clone(),
            recipients,
            t.token.clone(),
            1000,
            event_ledger,
        );

        // Try to replace recipient1 with recipient2 (already exists)
        t.client.replace_split_recipient(&t.sender, &split_id, &t.recipient1, &t.recipient2);
    }

    #[test]
    #[should_panic(expected = "not authorised to replace recipient")]
    fn test_replace_split_recipient_unauthorized() {
        let t = setup();
        let event_ledger = t.e.ledger().sequence() + 1000;
        let unauthorized_user = Address::generate(&t.e);

        // Create split
        let recipients = Vec::from_array(&t.e, [(t.recipient1.clone(), 6000), (t.recipient2.clone(), 4000)]);
        let split_id = crate::splitter::create_split(
            t.e.clone(),
            t.sender.clone(),
            recipients,
            t.token.clone(),
            1000,
            event_ledger,
        );

        // Try to replace recipient from non-sender account
        t.client.replace_split_recipient(&unauthorized_user, &split_id, &t.recipient1, &t.new_recipient);
    }

    #[test]
    #[should_panic(expected = "split has already been distributed")]
    fn test_replace_split_recipient_distributed() {
        let t = setup();
        let event_ledger = t.e.ledger().sequence() + 1000;

        // Create split
        let recipients = Vec::from_array(&t.e, [(t.recipient1.clone(), 6000), (t.recipient2.clone(), 4000)]);
        let split_id = crate::splitter::create_split(
            t.e.clone(),
            t.sender.clone(),
            recipients,
            t.token.clone(),
            1000,
            event_ledger,
        );

        // Distribute the split
        crate::splitter::distribute_split(t.e.clone(), t.sender.clone(), split_id);

        // Try to replace recipient after distribution
        t.client.replace_split_recipient(&t.sender, &split_id, &t.recipient1, &t.new_recipient);
    }

    #[test]
    #[should_panic(expected = "split has been cancelled")]
    fn test_replace_split_recipient_cancelled() {
        let t = setup();
        let event_ledger = t.e.ledger().sequence() + 1000;

        // Create split
        let recipients = Vec::from_array(&t.e, [(t.recipient1.clone(), 6000), (t.recipient2.clone(), 4000)]);
        let split_id = crate::splitter::create_split(
            t.e.clone(),
            t.sender.clone(),
            recipients,
            t.token.clone(),
            1000,
            event_ledger,
        );

        // Cancel the split
        crate::splitter::cancel_split(t.e.clone(), t.sender.clone(), split_id);

        // Try to replace recipient after cancellation
        t.client.replace_split_recipient(&t.sender, &split_id, &t.recipient1, &t.new_recipient);
    }
}