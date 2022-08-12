#![cfg(feature = "test-bpf")]

use crate::entrypoint::process_instruction;
use crate::helpers::{
    get_program_source_token_pubkey_and_bump, get_target_token_mint_pubkey_and_bump,
    get_token_associated_account_pubkey_and_bump,
};
use crate::id;
use crate::instruction::ProgramInstruction;
use crate::state::Config;
use borsh::{BorshDeserialize, BorshSerialize};
use solana_program::program_pack::Pack;
use solana_program::system_instruction;
use solana_program_test::{processor, tokio, ProgramTest, ProgramTestContext};
use solana_sdk::signature::{Keypair, Signer};
use solana_sdk::transaction::Transaction;

struct Env {
    test_context: ProgramTestContext,
    admin: Keypair,
    user: Keypair,
    source_token_mint_info: Keypair,
}

impl Env {
    async fn new() -> Self {
        let program_test = ProgramTest::new("solata_test", id(), processor!(process_instruction));
        let mut test_context = program_test.start_with_context().await;

        let admin = Keypair::new();
        let user = Keypair::new();
        let source_token_mint_info = Keypair::new();
        let rent = test_context.banks_client.get_rent().await.unwrap();

        // Credit admin and user accounts
        test_context
            .banks_client
            .process_transaction(Transaction::new_signed_with_payer(
                &[
                    system_instruction::transfer(
                        &test_context.payer.pubkey(),
                        &admin.pubkey(),
                        1_000_000_000,
                    ),
                    system_instruction::transfer(
                        &test_context.payer.pubkey(),
                        &user.pubkey(),
                        1_000_000_000,
                    ),
                ],
                Some(&test_context.payer.pubkey()),
                &[&test_context.payer],
                test_context.last_blockhash,
            ))
            .await
            .unwrap();

        // Create mint account
        test_context
            .banks_client
            .process_transaction(Transaction::new_signed_with_payer(
                &[
                    system_instruction::create_account(
                        &test_context.payer.pubkey(),
                        &source_token_mint_info.pubkey(),
                        rent.minimum_balance(spl_token::state::Mint::LEN),
                        spl_token::state::Mint::LEN as u64,
                        &spl_token::id(),
                    ),
                    spl_token::instruction::initialize_mint(
                        &spl_token::id(),
                        &source_token_mint_info.pubkey(),
                        &admin.pubkey(),
                        None,
                        0,
                    )
                    .unwrap(),
                ],
                Some(&test_context.payer.pubkey()),
                &[&test_context.payer, &source_token_mint_info],
                test_context.last_blockhash,
            ))
            .await
            .unwrap();

        // Init program account
        test_context
            .banks_client
            .process_transaction(Transaction::new_signed_with_payer(
                &[ProgramInstruction::init(
                    &admin.pubkey(),
                    &source_token_mint_info.pubkey(),
                    &admin.pubkey(),
                    &admin.pubkey(),
                )],
                Some(&admin.pubkey()),
                &[&admin],
                test_context.last_blockhash,
            ))
            .await
            .unwrap();

        let (config_pubkey, _) = Config::get_pubkey_with_bump();
        let acc = test_context
            .banks_client
            .get_account(config_pubkey)
            .await
            .unwrap()
            .unwrap();
        let config: Config = Config::try_from_slice(acc.data.as_slice()).unwrap();
        assert_eq!(config.source_token_mint, source_token_mint_info.pubkey());
        assert_eq!(
            config.target_token_mint,
            get_target_token_mint_pubkey_and_bump().0
        );

        Env {
            test_context,
            admin,
            user,
            source_token_mint_info,
        }
    }
}

#[tokio::test]
async fn test_init() {
    let _env = Env::new().await;
}

#[tokio::test]
async fn test_enter() {
    let mut env = Env::new().await;

    let source_token_account = Keypair::new();
    let rent = env.test_context.banks_client.get_rent().await.unwrap();

    let (program_source_token_pubkey, _) =
        get_program_source_token_pubkey_and_bump(&env.source_token_mint_info.pubkey());

    env.test_context
        .banks_client
        .process_transaction(Transaction::new_signed_with_payer(
            &[
                system_instruction::create_account(
                    &env.admin.pubkey(),
                    &source_token_account.pubkey(),
                    rent.minimum_balance(spl_token::state::Account::LEN),
                    spl_token::state::Account::LEN as u64,
                    &spl_token::id(),
                ),
                spl_token::instruction::initialize_account(
                    &spl_token::id(),
                    &source_token_account.pubkey(),
                    &env.source_token_mint_info.pubkey(),
                    &env.user.pubkey(),
                )
                .unwrap(),
                spl_token::instruction::mint_to(
                    &spl_token::id(),
                    &env.source_token_mint_info.pubkey(),
                    &source_token_account.pubkey(),
                    &env.admin.pubkey(),
                    &[],
                    1000,
                )
                .unwrap(),
            ],
            Some(&env.admin.pubkey()),
            &[&env.admin, &source_token_account],
            env.test_context.last_blockhash,
        ))
        .await
        .unwrap();

    env.test_context
        .banks_client
        .process_transaction(Transaction::new_signed_with_payer(
            &[ProgramInstruction::enter(
                &env.user.pubkey(),
                &env.source_token_mint_info.pubkey(),
                &env.admin.pubkey(),
                &source_token_account.pubkey(),
                1_000,
            )],
            Some(&env.user.pubkey()),
            &[&env.user, &env.admin],
            env.test_context.last_blockhash,
        ))
        .await
        .unwrap();

    let acc = env
        .test_context
        .banks_client
        .get_account(program_source_token_pubkey)
        .await
        .unwrap()
        .unwrap();

    let program_source_token_account: spl_token::state::Account =
        spl_token::state::Account::unpack(acc.data.as_slice()).unwrap();
    assert_eq!(program_source_token_account.amount, 1000u64);

    let (target_token_mint_info_pubkey, _) = get_target_token_mint_pubkey_and_bump();
    let (user_target_token_account, _) = get_token_associated_account_pubkey_and_bump(
        &env.user.pubkey(),
        &target_token_mint_info_pubkey,
    );
    let acc = env
        .test_context
        .banks_client
        .get_account(user_target_token_account)
        .await
        .unwrap()
        .unwrap();

    let user_target_token_account: spl_token::state::Account =
        spl_token::state::Account::unpack(acc.data.as_slice()).unwrap();
    assert_eq!(user_target_token_account.amount, 1000u64);
}
