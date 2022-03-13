#![allow(unused)]

use solana_program_test::*;
use solana_sdk::{
    commitment_config::CommitmentLevel,
    instruction::InstructionError,
    program_pack::Pack,
    pubkey::Pubkey,
    signature::{Keypair, Signer},
    system_instruction, sysvar,
    transaction::{Transaction, TransactionError},
};
use std::time;

pub fn get_error_code(error: BanksClientError) -> Option<u32> {
    if let BanksClientError::TransactionError(transaction_error) = error {
        let error_code = match transaction_error {
            TransactionError::InstructionError(_, instruction_error) => match instruction_error {
                InstructionError::Custom(code) => code,
                _ => return None,
            },
            _ => return None,
        };

        return Some(error_code);
    }

    None
}

pub async fn wait(context: &mut ProgramTestContext, duration: time::Duration) {
    let begin_time = context
        .banks_client
        .get_sysvar::<sysvar::clock::Clock>()
        .await
        .unwrap()
        .unix_timestamp;

    let end_time = (begin_time as u128 + duration.as_millis()) as i64;
    loop {
        let clock = context
            .banks_client
            .get_sysvar::<sysvar::clock::Clock>()
            .await
            .unwrap();

        if clock.unix_timestamp >= end_time {
            break;
        }

        let current_slot = context.banks_client.get_root_slot().await.unwrap();
        context.warp_to_slot(current_slot + 500).unwrap();
    }
}

pub async fn setup_test_context() -> ProgramTestContext {
    let mut program_test = ProgramTest::default();
    program_test.add_program("p2swap", p2swap::id(), None);
    let context = program_test.start_with_context().await;

    context
}

pub async fn airdrop(context: &mut ProgramTestContext, receiver: &Pubkey, amount: u64) {
    let tx = Transaction::new_signed_with_payer(
        &[system_instruction::transfer(
            &context.payer.pubkey(),
            receiver,
            amount,
        )],
        Some(&context.payer.pubkey()),
        &[&context.payer],
        context.last_blockhash,
    );

    context.banks_client.process_transaction(tx).await.unwrap();
}

pub async fn mint_to(
    context: &mut ProgramTestContext,
    mint: &Pubkey,
    to: &Pubkey,
    owner: &Keypair,
    amount: u64,
) {
    let tx = Transaction::new_signed_with_payer(
        &[spl_token::instruction::mint_to(
            &spl_token::id(),
            mint,
            to,
            &owner.pubkey(),
            &[],
            amount,
        )
        .unwrap()],
        Some(&context.payer.pubkey()),
        &[&context.payer, owner],
        context.last_blockhash,
    );

    context.banks_client.process_transaction(tx).await.unwrap();
}

pub async fn create_token_account(
    context: &mut ProgramTestContext,
    account: &Keypair,
    mint: &Pubkey,
    manager: &Pubkey,
) {
    let rent = context.banks_client.get_rent().await.unwrap();

    let tx = Transaction::new_signed_with_payer(
        &[
            system_instruction::create_account(
                &context.payer.pubkey(),
                &account.pubkey(),
                rent.minimum_balance(spl_token::state::Account::LEN),
                spl_token::state::Account::LEN as u64,
                &spl_token::id(),
            ),
            spl_token::instruction::initialize_account(
                &spl_token::id(),
                &account.pubkey(),
                mint,
                manager,
            )
            .unwrap(),
        ],
        Some(&context.payer.pubkey()),
        &[&context.payer, &account],
        context.last_blockhash,
    );

    context.banks_client.process_transaction(tx).await.unwrap();
}

pub async fn create_mint(
    context: &mut ProgramTestContext,
    mint: &Keypair,
    authority: &Pubkey,
    decimals: u8,
) {
    let rent = context.banks_client.get_rent().await.unwrap();

    let tx = Transaction::new_signed_with_payer(
        &[
            system_instruction::create_account(
                &context.payer.pubkey(),
                &mint.pubkey(),
                rent.minimum_balance(spl_token::state::Mint::LEN),
                spl_token::state::Mint::LEN as u64,
                &spl_token::id(),
            ),
            spl_token::instruction::initialize_mint(
                &spl_token::id(),
                &mint.pubkey(),
                authority,
                None,
                decimals,
            )
            .unwrap(),
        ],
        Some(&context.payer.pubkey()),
        &[&context.payer, mint],
        context.last_blockhash,
    );

    context.banks_client.process_transaction(tx).await.unwrap();
}
