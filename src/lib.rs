pub mod helpers;
pub mod instruction;
pub mod processor;
pub mod state;

#[cfg(not(feature = "no-entrypoint"))]
pub mod entrypoint;

#[cfg(test)]
mod tests;

const CONFIG_SEED: &str = "config";
const TARGET_TOKEN_MINT_SEED: &str = "target_token_mint";
solana_program::declare_id!("9onZvMzqAFzSHJrLNVWfqLRFFQ5ZCGzNXB4PBxmp6z5Y");
