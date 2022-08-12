use crate::state::Config;
use borsh::{BorshDeserialize, BorshSerialize};
use solana_program::pubkey::Pubkey;

#[derive(Debug, BorshDeserialize, BorshSerialize, PartialEq)]
pub enum Instruction {
    /// Updates config
    ///
    /// Accounts:
    /// [R] Admin
    /// [W] Config account PDA
    /// [R] Rent sysvar
    /// [R] System program
    UpdateConfig { params: Config },
    /// Enter
    ///
    /// Accounts:
    /// [R] User
    /// [W] Config account
    /// [R] Rent sysvar
    /// [R] System program
    Enter { amount: u64 },
}
