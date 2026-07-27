#[cfg(test)]
mod tests {
    use crate::test::create_token_contract;
    use soroban_sdk::{testutils::Address as _, Address, Env};
    use crate::contract::{VeriTixPay, VeriTixPayClient};

    #[test]
    fn test_batch_basic_operations() {
        let e = Env::default();
        e.mock_all_auths();

        let contract_id = e.register_contract(None, VeriTixPay);
        let client = VeriTixPayClient::new(&e, &contract_id);

        let admin = Address::generate(&e);
        client.initialize(&admin);

        let token = create_token_contract(&e, &admin);
        let token_client = soroban_sdk::token::StellarAssetClient::new(&e, &token);

        let addr1 = Address::generate(&e);
        let addr2 = Address::generate(&e);

        token_client.mint(&addr1, &1000);
        token_client.mint(&addr2, &2000);

        let tc = soroban_sdk::token::Client::new(&e, &token);
        assert_eq!(tc.balance(&addr1), 1000);
        assert_eq!(tc.balance(&addr2), 2000);
    }
}
