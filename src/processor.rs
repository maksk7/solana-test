use crate::helpers::{
    check_signed, get_program_source_token_pubkey_and_bump, get_target_token_mint_pubkey_and_bump,
    get_token_associated_account_pubkey_and_bump,
};
use crate::instruction::ProgramInstruction;
use crate::state::Config;
use crate::{id, CONFIG_SEED, TARGET_TOKEN_MINT_SEED};
use borsh::{BorshDeserialize, BorshSerialize};
use solana_program::account_info::{next_account_info, AccountInfo};
use solana_program::entrypoint::ProgramResult;
use solana_program::program::{invoke, invoke_signed};
use solana_program::program_error::ProgramError;
use solana_program::program_pack::Pack;
use solana_program::pubkey::Pubkey;
use solana_program::rent::Rent;
use solana_program::sysvar::Sysvar;
use solana_program::{msg, system_instruction};

pub struct Processor;

impl Processor {
    pub fn process_instruction(
        program_id: &Pubkey,
        accounts: &[AccountInfo],
        input: &[u8],
    ) -> ProgramResult {
        match ProgramInstruction::try_from_slice(input)? {
            ProgramInstruction::Init => Self::process_init(program_id, accounts)?,
            ProgramInstruction::Enter { amount } => {
                Self::process_enter(program_id, accounts, amount)?
            }
        }

        Ok(())
    }

    pub fn process_init(program_id: &Pubkey, accounts: &[AccountInfo]) -> ProgramResult {
        msg!("Instruction: Init");
        let accounts = &mut accounts.iter();

        let from_info = next_account_info(accounts)?;
        let config_info = next_account_info(accounts)?;
        let source_token_mint_info = next_account_info(accounts)?;
        let source_token_mint_authority = next_account_info(accounts)?;
        let target_token_mint_authority = next_account_info(accounts)?;
        let target_token_mint_info = next_account_info(accounts)?;
        let program_source_token_info = next_account_info(accounts)?;
        let rent_info = next_account_info(accounts)?;
        let rent = Rent::from_account_info(rent_info)?;
        let _system_program_info = next_account_info(accounts)?;
        let _token_program_info = next_account_info(accounts)?;

        check_signed(from_info)?;
        if !target_token_mint_info.data_is_empty()
            || !config_info.data_is_empty()
            || !program_source_token_info.data_is_empty()
        {
            return Err(ProgramError::InvalidAccountData);
        }

        let (target_token_mint_pubkey, target_token_mint_bump) =
            get_target_token_mint_pubkey_and_bump();
        if target_token_mint_pubkey != *target_token_mint_info.key {
            return Err(ProgramError::InvalidArgument);
        }
        let (config_pubkey, config_bump) = Config::get_pubkey_with_bump();
        if config_info.key != &config_pubkey {
            return Err(ProgramError::InvalidArgument);
        }

        msg!("Creating target token mint account");
        invoke_signed(
            &system_instruction::create_account(
                from_info.key,
                target_token_mint_info.key,
                rent.minimum_balance(spl_token::state::Mint::LEN),
                spl_token::state::Mint::LEN as u64,
                &spl_token::id(),
            ),
            &[from_info.clone(), target_token_mint_info.clone()],
            &[&[TARGET_TOKEN_MINT_SEED.as_bytes(), &[target_token_mint_bump]]],
        )?;
        invoke(
            &spl_token::instruction::initialize_mint(
                &spl_token::id(),
                target_token_mint_info.key,
                target_token_mint_authority.key,
                None,
                0,
            )?,
            &[target_token_mint_info.clone(), rent_info.clone()],
        )?;

        msg!("Creating config account");
        let config = Config {
            source_token_mint: *source_token_mint_info.key,
            target_token_mint: *target_token_mint_info.key,
        };
        let space = config.try_to_vec()?.len();
        invoke_signed(
            &system_instruction::create_account(
                from_info.key,
                &config_pubkey,
                rent.minimum_balance(space),
                space as u64,
                program_id,
            ),
            &[from_info.clone(), config_info.clone()],
            &[&[CONFIG_SEED.as_bytes(), &[config_bump]]],
        )?;

        let mut config = Config::try_from_slice(&config_info.data.borrow_mut())?;
        config.source_token_mint = *source_token_mint_info.key;
        config.target_token_mint = *target_token_mint_info.key;
        let _ = config.serialize(&mut &mut config_info.data.borrow_mut()[..]);

        msg!("Creating program source token PDA account");
        let (_, program_source_token_bump) =
            get_program_source_token_pubkey_and_bump(source_token_mint_info.key);
        let seeds = &[
            &source_token_mint_info.key.to_bytes()[0..32],
            &id().to_bytes()[0..32],
            &[program_source_token_bump],
        ];
        invoke_signed(
            &system_instruction::create_account(
                from_info.key,
                program_source_token_info.key,
                rent.minimum_balance(spl_token::state::Account::LEN),
                spl_token::state::Account::LEN as u64,
                &spl_token::id(),
            ),
            &[from_info.clone(), program_source_token_info.clone()],
            &[seeds],
        )?;
        invoke(
            &spl_token::instruction::initialize_account(
                &spl_token::id(),
                program_source_token_info.key,
                source_token_mint_info.key,
                source_token_mint_authority.key,
            )?,
            &[
                program_source_token_info.clone(),
                source_token_mint_info.clone(),
                source_token_mint_authority.clone(),
                rent_info.clone(),
            ],
        )?;

        msg!("Operation process_init has been completed.");

        Ok(())
    }

    pub fn process_enter(
        _program_id: &Pubkey,
        accounts: &[AccountInfo],
        amount: u64,
    ) -> ProgramResult {
        let accounts = &mut accounts.iter();

        let from_account_info = next_account_info(accounts)?;
        let source_token_mint_info = next_account_info(accounts)?;
        let target_token_mint_info = next_account_info(accounts)?;
        let target_token_mint_authority_info = next_account_info(accounts)?;
        let source_token_account_info = next_account_info(accounts)?;
        let target_token_account_info = next_account_info(accounts)?;
        let program_source_token_info = next_account_info(accounts)?;
        let config_account_info = next_account_info(accounts)?;
        let rent_info = next_account_info(accounts)?;
        let rent = Rent::from_account_info(rent_info)?;
        let _system_program_info = next_account_info(accounts)?;
        let _token_program_info = next_account_info(accounts)?;

        if config_account_info.data_is_empty() {
            return Err(ProgramError::UninitializedAccount);
        }

        let config: Config = Config::try_from_slice(&config_account_info.data.borrow_mut())?;
        if target_token_mint_info.key != &config.target_token_mint
            || source_token_mint_info.key != &config.source_token_mint
        {
            return Err(ProgramError::InvalidArgument);
        }

        if target_token_account_info.data_is_empty() {
            msg!("Creating target token account");

            let (_, bump) = get_token_associated_account_pubkey_and_bump(
                from_account_info.key,
                target_token_mint_info.key,
            );
            let seeds = &[
                &from_account_info.key.to_bytes()[0..32],
                &spl_token::id().to_bytes()[0..32],
                &target_token_mint_info.key.to_bytes()[0..32],
                &[bump],
            ];

            invoke_signed(
                &system_instruction::create_account(
                    from_account_info.key,
                    target_token_account_info.key,
                    rent.minimum_balance(spl_token::state::Account::LEN),
                    spl_token::state::Account::LEN as u64,
                    &spl_token::id(),
                ),
                &[from_account_info.clone(), target_token_account_info.clone()],
                &[seeds],
            )?;
            invoke(
                &spl_token::instruction::initialize_account(
                    &spl_token::id(),
                    target_token_account_info.key,
                    target_token_mint_info.key,
                    target_token_mint_authority_info.key,
                )?,
                &[
                    target_token_account_info.clone(),
                    target_token_mint_info.clone(),
                    target_token_mint_authority_info.clone(),
                    rent_info.clone(),
                ],
            )?;
        }

        invoke(
            &spl_token::instruction::transfer(
                &spl_token::id(),
                source_token_account_info.key,
                program_source_token_info.key,
                from_account_info.key,
                &[from_account_info.key],
                amount,
            )?,
            &[
                source_token_account_info.clone(),
                program_source_token_info.clone(),
                from_account_info.clone(),
                _token_program_info.clone(),
            ],
        )?;
        invoke_signed(
            &spl_token::instruction::mint_to(
                &spl_token::id(),
                target_token_mint_info.key,
                target_token_account_info.key,
                target_token_mint_authority_info.key,
                &[],
                amount,
            )?,
            &[
                target_token_mint_info.clone(),
                target_token_account_info.clone(),
                target_token_mint_authority_info.clone(),
            ],
            &[],
        )?;

        msg!("Operation process_enter has been done.");
        Ok(())
    }
}
