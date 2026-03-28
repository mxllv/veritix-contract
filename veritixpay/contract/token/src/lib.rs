#![no_std]
#![allow(unexpected_cfgs)]

// Veritix Pay — Smart contract logic coming soon.
// Contributors: see CONTRIBUTING.md for how to get started.

pub mod admin;
pub mod allowance;
pub mod balance;
pub mod dispute;
pub mod escrow;
pub mod freeze;
pub mod metadata;
pub mod recurring;
pub mod splitter;
pub mod storage_types;

mod contract;

#[cfg(test)]
mod test;

#[cfg(test)]
mod admin_test;

#[cfg(test)]
mod escrow_test;

#[cfg(test)]
mod recurring_test;

#[cfg(test)]
mod splitter_test;

pub use crate::contract::VeritixToken;
