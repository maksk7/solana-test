use crate::{id, TARGET_TOKEN_MINT_SEED};
use solana_program::account_info::AccountInfo;
use solana_program::entrypoint::ProgramResult;
use solana_program::program_error::ProgramError;
use solana_program::pubkey::Pubkey;

pub fn check_signed(account: &AccountInfo) -> ProgramResult {
    if account.is_signer {
        return Ok(());
    }
    Err(ProgramError::MissingRequiredSignature)
}

pub fn get_target_token_mint_pubkey_and_bump() -> (Pubkey, u8) {
    Pubkey::find_program_address(&[TARGET_TOKEN_MINT_SEED.as_bytes()], &id())
}

pub fn get_program_source_token_pubkey_and_bump(
    source_token_mint_info_key: &Pubkey,
) -> (Pubkey, u8) {
    Pubkey::find_program_address(
        &[&source_token_mint_info_key.to_bytes(), &id().to_bytes()],
        &id(),
    )
}

pub fn get_token_associated_account_pubkey_and_bump(
    account_pubkey: &Pubkey,
    mint_token_pubkey: &Pubkey,
) -> (Pubkey, u8) {
    Pubkey::find_program_address(
        &[
            &account_pubkey.to_bytes(),
            &spl_token::id().to_bytes(),
            &mint_token_pubkey.to_bytes(),
        ],
        &id(),
    )
}
