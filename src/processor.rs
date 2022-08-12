use crate::helpers::check_signed;
use crate::instruction::Instruction;
use crate::state::Config;
use crate::{id, CONFIG_SEED};
use borsh::{BorshDeserialize, BorshSerialize};
use solana_program::account_info::{next_account_info, AccountInfo};
use solana_program::entrypoint::ProgramResult;
use solana_program::program::invoke_signed;
use solana_program::program_error::ProgramError;
use solana_program::program_pack::Pack;
use solana_program::pubkey::Pubkey;
use solana_program::rent::Rent;
use solana_program::sysvar::Sysvar;
use solana_program::{msg, system_instruction};
use spl_token::check_program_account;
use spl_token::state::Account;

pub struct Processor;

impl Processor {
    pub fn process_instruction(
        program_id: &Pubkey,
        accounts: &[AccountInfo],
        input: &[u8],
    ) -> ProgramResult {
        match Instruction::try_from_slice(input)? {
            Instruction::UpdateConfig { params } => {
                Self::process_update_config(program_id, accounts, params)?
            }
            Instruction::Enter { amount } => Self::process_enter(program_id, accounts, amount)?,
        }

        Ok(())
    }

    pub fn process_update_config(
        program_id: &Pubkey,
        accounts: &[AccountInfo],
        params: Config,
    ) -> ProgramResult {
        let accounts = &mut accounts.iter();

        let admin_info = next_account_info(accounts)?;
        let config = next_account_info(accounts)?;
        let rent = next_account_info(accounts)?;

        check_signed(admin_info)?;
        check_program_account(&params.source_token)?;
        check_program_account(&params.target_token)?;

        if config.data_is_empty() {
            msg!("Creating config account");

            let rent = Rent::from_account_info(rent)?;
            let space = params.try_to_vec()?.len();
            let (config_pubkey, bump_seed) =
                Pubkey::find_program_address(&["config".as_bytes()], &program_id);
            if config.key != config_pubkey {
                return Err(ProgramError::InvalidArgument);
            }
            let seeds = &["config".as_bytes(), &[bump_seed]];

            invoke_signed(
                &system_instruction::create_account(
                    admin_info.key,
                    &config_pubkey,
                    rent.minimum_balance(space),
                    space,
                    &program_id,
                ),
                &[admin_info.clone(), config.clone()],
                &[seeds],
            )?;
        }

        msg!("Operation: process_update_config");
        Ok(())
    }

    pub fn process_enter(
        program_id: &Pubkey,
        accounts: &[AccountInfo],
        amount: u64,
    ) -> ProgramResult {
        let accounts = &mut accounts.iter();

        let user_account_info = next_account_info(accounts)?;
        let source_token_account_info = next_account_info(accounts)?;
        let target_token_account_info = next_account_info(accounts)?;
        let config_account_info = next_account_info(accounts)?;

        let config: Config = Config::try_from_slice(&config_account_info.data.borrow())?;

        let source_associated = spl_associated_token_account::get_associated_token_address(
            &user_account_info.key,
            &config.source_token,
        );
        let target_associated = spl_associated_token_account::get_associated_token_address(
            &user_account_info.key,
            &config.target_token,
        );

        if source_token_account_info != source_associated
            || target_token_account_info != target_associated
        {
            return Err(ProgramError::InvalidArgument);
        }

        let mut source_account = Account::unpack(&source_token_account_info.data.borrow())?;
        source_account.amount = source_account.amount.checked_sub(amount)?;

        spl_token::instruction::mint_to(
            &config.target_token,
            &target_associated,
            &[],
            program_id,
            &[],
            amount,
        )?;

        msg!("Operation: process_enter");
        Ok(())
    }
}
