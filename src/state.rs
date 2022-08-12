use crate::{id, CONFIG_SEED};
use borsh::{BorshDeserialize, BorshSerialize};
use solana_program::pubkey::Pubkey;

#[derive(BorshSerialize, BorshDeserialize, Debug, Clone, Copy, PartialEq)]
pub struct Config {
    pub source_token_mint: Pubkey,
    pub target_token_mint: Pubkey,
}

impl Config {
    pub fn get_pubkey_with_bump() -> (Pubkey, u8) {
        Pubkey::find_program_address(&[CONFIG_SEED.as_bytes()], &id())
    }
}
