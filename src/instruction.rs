use crate::helpers::{
    get_program_source_token_pubkey_and_bump, get_target_token_mint_pubkey_and_bump,
    get_token_associated_account_pubkey_and_bump,
};
use crate::id;
use crate::state::Config;
use borsh::{BorshDeserialize, BorshSerialize};
use solana_program::instruction::{AccountMeta, Instruction};
use solana_program::pubkey::Pubkey;
use solana_program::{system_program, sysvar};

#[derive(Debug, BorshDeserialize, BorshSerialize, PartialEq)]
pub enum ProgramInstruction {
    /// Init program configuration
    ///
    /// Accounts:
    /// [RS] User account
    /// [W] Config account PDA
    /// [R] Source token mint account
    /// [R] Source token mint authority account
    /// [R] Target token mint authority account
    /// [W] Target token mint account
    /// [W] Program source token account
    /// [R] Rent sysvar
    /// [R] System program id
    /// [R] Token program id
    Init,
    /// Enter
    ///
    /// Accounts:
    /// [RS] User
    /// [R] Source token mint account
    /// [W] Target token mint account
    /// [RS] Target token mint authority account
    /// [W] Source user token account
    /// [W] Target user token account
    /// [W] Program source token account
    /// [R] Config account PDA
    /// [R] Rent sysvar
    /// [R] System program id
    /// [R] Token program id
    Enter { amount: u64 },
}

impl ProgramInstruction {
    pub fn init(
        from_account_pubkey: &Pubkey,
        source_token_mint_pubkey: &Pubkey,
        source_token_mint_authority_pubkey: &Pubkey,
        target_token_mint_authority: &Pubkey,
    ) -> Instruction {
        let (config_pubkey, _) = Config::get_pubkey_with_bump();
        let (target_token_mint_pubkey, _) = get_target_token_mint_pubkey_and_bump();
        let (program_source_token_pubkey, _) =
            get_program_source_token_pubkey_and_bump(source_token_mint_pubkey);

        Instruction::new_with_borsh(
            id(),
            &ProgramInstruction::Init,
            vec![
                AccountMeta::new(*from_account_pubkey, true),
                AccountMeta::new(config_pubkey, false),
                AccountMeta::new_readonly(*source_token_mint_pubkey, false),
                AccountMeta::new_readonly(*source_token_mint_authority_pubkey, false),
                AccountMeta::new_readonly(*target_token_mint_authority, false),
                AccountMeta::new(target_token_mint_pubkey, false),
                AccountMeta::new(program_source_token_pubkey, false),
                AccountMeta::new_readonly(sysvar::rent::id(), false),
                AccountMeta::new_readonly(system_program::id(), false),
                AccountMeta::new_readonly(spl_token::id(), false),
            ],
        )
    }

    pub fn enter(
        from_account_pubkey: &Pubkey,
        source_token_mint_pubkey: &Pubkey,
        target_token_mint_authority_pubkey: &Pubkey,
        source_token_account_pubkey: &Pubkey,
        amount: u64,
    ) -> Instruction {
        let (config_pubkey, _) = Config::get_pubkey_with_bump();
        let (target_token_mint_pubkey, _) = get_target_token_mint_pubkey_and_bump();
        let (program_source_token_pubkey, _) =
            get_program_source_token_pubkey_and_bump(source_token_mint_pubkey);
        let (target_token_account_pubkey, _) = get_token_associated_account_pubkey_and_bump(
            from_account_pubkey,
            &target_token_mint_pubkey,
        );

        Instruction::new_with_borsh(
            id(),
            &ProgramInstruction::Enter { amount },
            vec![
                AccountMeta::new(*from_account_pubkey, true),
                AccountMeta::new_readonly(*source_token_mint_pubkey, false),
                AccountMeta::new(target_token_mint_pubkey, false),
                AccountMeta::new_readonly(*target_token_mint_authority_pubkey, true),
                AccountMeta::new(*source_token_account_pubkey, false),
                AccountMeta::new(target_token_account_pubkey, false),
                AccountMeta::new(program_source_token_pubkey, false),
                AccountMeta::new_readonly(config_pubkey, false),
                AccountMeta::new_readonly(sysvar::rent::id(), false),
                AccountMeta::new_readonly(system_program::id(), false),
                AccountMeta::new_readonly(spl_token::id(), false),
            ],
        )
    }
}
