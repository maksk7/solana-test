use borsh::{BorshDeserialize, BorshSerialize};
use solana_program::pubkey::Pubkey;

#[derive(BorshSerialize, BorshDeserialize, Debug, Clone, Copy, PartialEq)]
pub struct Config {
    pub source_token: Pubkey,
    pub target_token: Pubkey,
}
