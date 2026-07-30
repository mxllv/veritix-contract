#![no_std]
#![allow(dead_code)]
#![allow(deprecated)]

mod admin;
mod admin_test;
mod allowance;
mod allowance_test;
mod balance;
mod contract;
mod dispute;
mod dispute_test;
mod escrow;
mod escrow_test;
mod freeze;
mod multi_escrow;
mod multi_escrow_test;
mod permit;
mod recurring;
mod recurring_test;
mod pause;
mod splitter;
mod splitter_test;
mod storage_types;
#[cfg(test)]
mod test;
#[cfg(test)]
mod version_test;
#[cfg(test)]
mod contract_summary;
mod validation;
mod whitelist;
