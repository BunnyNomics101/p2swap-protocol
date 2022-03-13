mod utils;

use anchor_lang::{prelude::*, Id, InstructionData, System, ToAccountMetas};
use anchor_spl::token;
use p2swap;
use solana_program_test::*;
use solana_sdk::{
    borsh::try_from_slice_unchecked, instruction::Instruction, signature::Keypair, signer::Signer,
    sysvar, transaction::Transaction,
};

#[tokio::test]
async fn success_native() {
    let mut context = utils::setup_test_context().await;

    let order = Keypair::new();
    let funder = Keypair::new();
    let recipient = Keypair::new();

    let base_amount = 10 * 10u64.pow(9);
    let quote_amount = 11 * 10u64.pow(9);
    let expire_date = 9999999999;

    utils::airdrop(&mut context, &funder.pubkey(), base_amount * 2).await;
    utils::airdrop(&mut context, &recipient.pubkey(), quote_amount * 2).await;

    let (escrow, escrow_bump) =
        p2swap::utils::find_order_escrow_address(&funder.pubkey(), &order.pubkey());

    let accounts = p2swap::accounts::CreateOrder {
        order: order.pubkey(),
        funder: funder.pubkey(),
        recipient: recipient.pubkey(),
        funder_token_account: funder.pubkey(),
        escrow,
        quote_token_account: funder.pubkey(),
        escrow_mint: System::id(),
        quote_mint: System::id(),
        rent_sysvar: sysvar::rent::id(),
        clock_sysvar: sysvar::clock::id(),
        token_program: spl_token::id(),
        system_program: System::id(),
    }
    .to_account_metas(None);

    let data = p2swap::instruction::CreateOrder {
        escrow_bump,
        base_amount,
        quote_amount,
        start_date: None,
        expire_date,
    }
    .data();

    let instruction = Instruction {
        program_id: p2swap::id(),
        data,
        accounts,
    };

    let tx = Transaction::new_signed_with_payer(
        &[instruction],
        Some(&funder.pubkey()),
        &[&funder, &order],
        context.last_blockhash,
    );

    context.banks_client.process_transaction(tx).await.unwrap();

    let escrow_account = context
        .banks_client
        .get_account(escrow)
        .await
        .unwrap()
        .unwrap();

    let rent = context.banks_client.get_rent().await.unwrap();
    let escrow_rent = rent.minimum_balance(p2swap::utils::ORDER_ESCROW_NATIVE_SIZE);
    assert_eq!(escrow_account.lamports, base_amount + escrow_rent);

    let order_account = context
        .banks_client
        .get_account(order.pubkey())
        .await
        .unwrap()
        .unwrap();

    let order = try_from_slice_unchecked::<p2swap::state::Order>(&order_account.data[8..]).unwrap();
    assert_eq!(order.base_amount, base_amount);
    assert_eq!(order.quote_amount, quote_amount);
    assert_eq!(order.funder, funder.pubkey());
    assert_eq!(order.recipient, recipient.pubkey());
    assert_eq!(order.escrow, escrow);
    assert_eq!(order.quote_token_account, funder.pubkey());
    assert!(order.start_date.is_none());
    assert!(order.is_base_native);
    assert!(order.is_quote_native);
    assert_eq!(order.expire_date, expire_date);
    assert_eq!(order.status, p2swap::state::OrderStatus::Created);
}

#[tokio::test]
async fn success_spl_token() {
    let mut context = utils::setup_test_context().await;

    let order = Keypair::new();
    let funder = Keypair::new();
    let recipient = Keypair::new();

    let base_amount = 10 * 10u64.pow(9);
    let quote_amount = 11 * 10u64.pow(9);
    let expire_date = 9999999999;

    utils::airdrop(&mut context, &funder.pubkey(), base_amount).await;
    utils::airdrop(&mut context, &recipient.pubkey(), quote_amount).await;

    let funder_token_mint = Keypair::new();
    let funder_token_account = Keypair::new();

    utils::create_mint(&mut context, &funder_token_mint, &funder.pubkey(), 9).await;
    utils::create_token_account(
        &mut context,
        &funder_token_account,
        &funder_token_mint.pubkey(),
        &funder.pubkey(),
    )
    .await;
    utils::mint_to(
        &mut context,
        &funder_token_mint.pubkey(),
        &funder_token_account.pubkey(),
        &funder,
        base_amount,
    )
    .await;

    let (escrow, escrow_bump) =
        p2swap::utils::find_order_escrow_address(&funder.pubkey(), &order.pubkey());

    let accounts = p2swap::accounts::CreateOrder {
        order: order.pubkey(),
        funder: funder.pubkey(),
        recipient: recipient.pubkey(),
        funder_token_account: funder_token_account.pubkey(),
        escrow,
        quote_token_account: funder.pubkey(),
        escrow_mint: funder_token_mint.pubkey(),
        quote_mint: System::id(),
        rent_sysvar: sysvar::rent::id(),
        clock_sysvar: sysvar::clock::id(),
        token_program: spl_token::id(),
        system_program: System::id(),
    }
    .to_account_metas(None);

    let data = p2swap::instruction::CreateOrder {
        escrow_bump,
        base_amount,
        quote_amount,
        start_date: None,
        expire_date,
    }
    .data();

    let instruction = Instruction {
        program_id: p2swap::id(),
        data,
        accounts,
    };

    let tx = Transaction::new_signed_with_payer(
        &[instruction],
        Some(&funder.pubkey()),
        &[&funder, &order],
        context.last_blockhash,
    );

    context.banks_client.process_transaction(tx).await.unwrap();

    let escrow_account = context
        .banks_client
        .get_account(escrow)
        .await
        .unwrap()
        .unwrap();

    let escrow_state =
        token::TokenAccount::try_deserialize_unchecked(&mut escrow_account.data.as_ref()).unwrap();
    assert_eq!(escrow_state.amount, base_amount);

    let order_account = context
        .banks_client
        .get_account(order.pubkey())
        .await
        .unwrap()
        .unwrap();

    let order = try_from_slice_unchecked::<p2swap::state::Order>(&order_account.data[8..]).unwrap();
    assert_eq!(order.base_amount, base_amount);
    assert_eq!(order.quote_amount, quote_amount);
    assert_eq!(order.funder, funder.pubkey());
    assert_eq!(order.recipient, recipient.pubkey());
    assert_eq!(order.escrow, escrow);
    assert_eq!(order.quote_token_account, funder.pubkey());
    assert!(order.start_date.is_none());
    assert!(!order.is_base_native);
    assert!(order.is_quote_native);
    assert_eq!(order.expire_date, expire_date);
    assert_eq!(order.status, p2swap::state::OrderStatus::Created);
}

#[tokio::test]
async fn fail_funder_account_wallet_mismatch() {
    let mut context = utils::setup_test_context().await;

    let order = Keypair::new();
    let funder = Keypair::new();
    let recipient = Keypair::new();

    let base_amount = 10 * 10u64.pow(9);
    let quote_amount = 11 * 10u64.pow(9);
    let expire_date = 9999999999;

    utils::airdrop(&mut context, &funder.pubkey(), base_amount * 2).await;
    utils::airdrop(&mut context, &recipient.pubkey(), quote_amount * 2).await;

    let (escrow, escrow_bump) =
        p2swap::utils::find_order_escrow_address(&funder.pubkey(), &order.pubkey());

    let accounts = p2swap::accounts::CreateOrder {
        order: order.pubkey(),
        funder: funder.pubkey(),
        recipient: recipient.pubkey(),
        funder_token_account: recipient.pubkey(),
        escrow,
        quote_token_account: funder.pubkey(),
        escrow_mint: System::id(),
        quote_mint: System::id(),
        rent_sysvar: sysvar::rent::id(),
        clock_sysvar: sysvar::clock::id(),
        token_program: spl_token::id(),
        system_program: System::id(),
    }
    .to_account_metas(None);

    let data = p2swap::instruction::CreateOrder {
        escrow_bump,
        base_amount,
        quote_amount,
        start_date: None,
        expire_date,
    }
    .data();

    let instruction = Instruction {
        program_id: p2swap::id(),
        data,
        accounts,
    };

    let tx = Transaction::new_signed_with_payer(
        &[instruction],
        Some(&funder.pubkey()),
        &[&funder, &order],
        context.last_blockhash,
    );

    let error = context
        .banks_client
        .process_transaction(tx)
        .await
        .unwrap_err();

    let error_code = utils::get_error_code(error);
    if let Some(error_code) = error_code {
        if error_code == 6001 {
            assert_eq!(true, true);
            return;
        }
    }
    assert_eq!(true, false);
}

#[tokio::test]
async fn fail_start_date_in_the_past() {
    let mut context = utils::setup_test_context().await;

    let order = Keypair::new();
    let funder = Keypair::new();
    let recipient = Keypair::new();

    let base_amount = 10 * 10u64.pow(9);
    let quote_amount = 11 * 10u64.pow(9);
    let start_date = 0;
    let expire_date = 9999999999;

    utils::airdrop(&mut context, &funder.pubkey(), base_amount * 2).await;
    utils::airdrop(&mut context, &recipient.pubkey(), quote_amount * 2).await;

    let (escrow, escrow_bump) =
        p2swap::utils::find_order_escrow_address(&funder.pubkey(), &order.pubkey());

    let accounts = p2swap::accounts::CreateOrder {
        order: order.pubkey(),
        funder: funder.pubkey(),
        recipient: recipient.pubkey(),
        funder_token_account: funder.pubkey(),
        escrow,
        quote_token_account: funder.pubkey(),
        escrow_mint: System::id(),
        quote_mint: System::id(),
        rent_sysvar: sysvar::rent::id(),
        clock_sysvar: sysvar::clock::id(),
        token_program: spl_token::id(),
        system_program: System::id(),
    }
    .to_account_metas(None);

    let data = p2swap::instruction::CreateOrder {
        escrow_bump,
        base_amount,
        quote_amount,
        start_date: Some(start_date),
        expire_date,
    }
    .data();

    let instruction = Instruction {
        program_id: p2swap::id(),
        data,
        accounts,
    };

    let tx = Transaction::new_signed_with_payer(
        &[instruction],
        Some(&funder.pubkey()),
        &[&funder, &order],
        context.last_blockhash,
    );

    let error = context
        .banks_client
        .process_transaction(tx)
        .await
        .unwrap_err();

    let error_code = utils::get_error_code(error);
    if let Some(error_code) = error_code {
        if error_code == 6007 {
            assert_eq!(true, true);
            return;
        }
    }
    assert_eq!(true, false);
}

#[tokio::test]
async fn fail_quote_account_wallet_mismatch() {
    let mut context = utils::setup_test_context().await;

    let order = Keypair::new();
    let funder = Keypair::new();
    let recipient = Keypair::new();

    let base_amount = 10 * 10u64.pow(9);
    let quote_amount = 11 * 10u64.pow(9);
    let expire_date = 9999999999;

    utils::airdrop(&mut context, &funder.pubkey(), base_amount * 2).await;
    utils::airdrop(&mut context, &recipient.pubkey(), quote_amount * 2).await;

    let (escrow, escrow_bump) =
        p2swap::utils::find_order_escrow_address(&funder.pubkey(), &order.pubkey());

    let accounts = p2swap::accounts::CreateOrder {
        order: order.pubkey(),
        funder: funder.pubkey(),
        recipient: recipient.pubkey(),
        funder_token_account: funder.pubkey(),
        escrow,
        quote_token_account: recipient.pubkey(),
        escrow_mint: System::id(),
        quote_mint: System::id(),
        rent_sysvar: sysvar::rent::id(),
        clock_sysvar: sysvar::clock::id(),
        token_program: spl_token::id(),
        system_program: System::id(),
    }
    .to_account_metas(None);

    let data = p2swap::instruction::CreateOrder {
        escrow_bump,
        base_amount,
        quote_amount,
        start_date: None,
        expire_date,
    }
    .data();

    let instruction = Instruction {
        program_id: p2swap::id(),
        data,
        accounts,
    };

    let tx = Transaction::new_signed_with_payer(
        &[instruction],
        Some(&funder.pubkey()),
        &[&funder, &order],
        context.last_blockhash,
    );

    let error = context
        .banks_client
        .process_transaction(tx)
        .await
        .unwrap_err();

    let error_code = utils::get_error_code(error);
    if let Some(error_code) = error_code {
        if error_code == 6000 {
            assert_eq!(true, true);
            return;
        }
    }
    assert_eq!(true, false);
}

#[tokio::test]
async fn fail_expire_date_in_the_past() {
    let mut context = utils::setup_test_context().await;

    let order = Keypair::new();
    let funder = Keypair::new();
    let recipient = Keypair::new();

    let base_amount = 10 * 10u64.pow(9);
    let quote_amount = 11 * 10u64.pow(9);
    let expire_date = 0;

    utils::airdrop(&mut context, &funder.pubkey(), base_amount * 2).await;
    utils::airdrop(&mut context, &recipient.pubkey(), quote_amount * 2).await;

    let (escrow, escrow_bump) =
        p2swap::utils::find_order_escrow_address(&funder.pubkey(), &order.pubkey());

    let accounts = p2swap::accounts::CreateOrder {
        order: order.pubkey(),
        funder: funder.pubkey(),
        recipient: recipient.pubkey(),
        funder_token_account: funder.pubkey(),
        escrow,
        quote_token_account: funder.pubkey(),
        escrow_mint: System::id(),
        quote_mint: System::id(),
        rent_sysvar: sysvar::rent::id(),
        clock_sysvar: sysvar::clock::id(),
        token_program: spl_token::id(),
        system_program: System::id(),
    }
    .to_account_metas(None);

    let data = p2swap::instruction::CreateOrder {
        escrow_bump,
        base_amount,
        quote_amount,
        start_date: None,
        expire_date,
    }
    .data();

    let instruction = Instruction {
        program_id: p2swap::id(),
        data,
        accounts,
    };

    let tx = Transaction::new_signed_with_payer(
        &[instruction],
        Some(&funder.pubkey()),
        &[&funder, &order],
        context.last_blockhash,
    );

    let error = context
        .banks_client
        .process_transaction(tx)
        .await
        .unwrap_err();

    let error_code = utils::get_error_code(error);
    if let Some(error_code) = error_code {
        if error_code == 6006 {
            assert_eq!(true, true);
            return;
        }
    }
    assert_eq!(true, false);
}
