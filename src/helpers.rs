use solana_program::account_info::AccountInfo;
use solana_program::entrypoint::ProgramResult;
use solana_program::program_error::ProgramError;

pub fn check_signed(account: &AccountInfo) -> ProgramResult {
    if account.is_signer {
        return Ok(());
    }
    Err(ProgramError::MissingRequiredSignature)
}
