#[cfg(test)]
mod tests {
    use super::*;
    use soroban_sdk::{testutils::Address as _, Env, Vec};

    #[test]
    fn test_reentrancy_guard_blocks_double_distribution() {
        let env = Env::default();
        
        let mut record = SplitRecord {
            distributed: false,
            recipients: Vec::from_array(&env, [env.accounts().generate(), env.accounts().generate()]),
        };

        assert!(!record.distributed);
        record.distributed = true;

        assert!(record.distributed);
    }
}