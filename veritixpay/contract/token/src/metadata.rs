use soroban_sdk::{contracttype, Env, String};

use crate::storage_types::DataKey;

pub const MAX_DECIMALS: u32 = 18;

#[derive(Clone)]
#[contracttype]
pub struct TokenMetadata {
    pub name: String,
    pub symbol: String,
    pub decimal: u32,
}

pub fn read_metadata(e: &Env) -> TokenMetadata {
    e.storage().instance().get(&DataKey::Metadata).unwrap()
}

pub fn write_metadata(e: &Env, metadata: TokenMetadata) {
    e.storage().instance().set(&DataKey::Metadata, &metadata);
}

pub fn validate_metadata(metadata: &TokenMetadata) {
    if metadata.name.len() == 0 {
        panic!("name cannot be empty");
    }

    if metadata.symbol.len() == 0 {
        panic!("symbol cannot be empty");
    }

    if metadata.decimal > MAX_DECIMALS {
        panic!("decimal exceeds maximum");
    }
}

pub fn read_decimal(e: &Env) -> u32 {
    read_metadata(e).decimal
}

pub fn read_name(e: &Env) -> String {
    read_metadata(e).name
}

pub fn read_symbol(e: &Env) -> String {
    read_metadata(e).symbol
}
