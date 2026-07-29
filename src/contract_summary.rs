#[cfg(test)]
mod contract_summary_tests {
    use soroban_sdk::{Env, Address};
    use crate::contract::VeriTixPayClient;
    use crate::contract::ContractSummary;

    fn setup() -> (Env, VeriTixPayClient<'static>, Address) {
        let e = Env::default();
        e.mock_all_auths();
        let admin = Address::generate(&e);
        let contract_id = e.register_contract(None, crate::contract::VeriTixPay);
        let client = VeriTixPayClient::new(&e, &contract_id);
        client.initialize(&admin);
        (e, client, admin)
    }

    #[test]
    fn test_contract_summary() {
        let (e, client, admin) = setup();
        let summary = client.contract_summary();
        assert_eq!(summary.admin, admin);
        assert_eq!(summary.total_supply, 0);
        assert_eq!(summary.escrow_count, 0);
        assert_eq!(summary.total_value_locked, 0);
    }
}
