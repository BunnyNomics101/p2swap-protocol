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

    let funder_account = context
        .banks_client
        .get_account(funder.pubkey())
        .await
        .unwrap()
        .unwrap();

    let funder_balance_before = funder_account.lamports;

    context.warp_to_slot(3).unwrap();

    let accounts = p2swap::accounts::CancelOrder {
        order: order.pubkey(),
        funder: funder.pubkey(),
        funder_token_account: funder.pubkey(),
        escrow,
        token_program: spl_token::id(),
        system_program: System::id(),
    }
    .to_account_metas(None);

    let data = p2swap::instruction::CancelOrder { escrow_bump }.data();

    let instruction = Instruction {
        program_id: p2swap::id(),
        data,
        accounts,
    };

    let tx = Transaction::new_signed_with_payer(
        &[instruction],
        Some(&funder.pubkey()),
        &[&funder],
        context.last_blockhash,
    );

    context.banks_client.process_transaction(tx).await.unwrap();

    let escrow_account = context.banks_client.get_account(escrow).await.unwrap();
    assert!(escrow_account.is_none());

    let order_account = context
        .banks_client
        .get_account(order.pubkey())
        .await
        .unwrap()
        .unwrap();

    let funder_account = context
        .banks_client
        .get_account(funder.pubkey())
        .await
        .unwrap()
        .unwrap();
    assert!(funder_account.lamports > funder_balance_before);

    let order = try_from_slice_unchecked::<p2swap::state::Order>(&order_account.data[8..]).unwrap();
    assert_eq!(order.status, p2swap::state::OrderStatus::Canceled);
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

    utils::airdrop(&mut context, &funder.pubkey(), base_amount * 2).await;
    utils::airdrop(&mut context, &recipient.pubkey(), quote_amount * 2).await;

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

    let funder_token_account_state = context
        .banks_client
        .get_account(funder_token_account.pubkey())
        .await
        .unwrap()
        .unwrap();

    let funder_token_account_balance_before = token::TokenAccount::try_deserialize_unchecked(
        &mut funder_token_account_state.data.as_ref(),
    )
    .unwrap()
    .amount;

    context.warp_to_slot(3).unwrap();

    let accounts = p2swap::accounts::CancelOrder {
        order: order.pubkey(),
        funder: funder.pubkey(),
        funder_token_account: funder_token_account.pubkey(),
        escrow,
        token_program: spl_token::id(),
        system_program: System::id(),
    }
    .to_account_metas(None);

    let data = p2swap::instruction::CancelOrder { escrow_bump }.data();

    let instruction = Instruction {
        program_id: p2swap::id(),
        data,
        accounts,
    };

    let tx = Transaction::new_signed_with_payer(
        &[instruction],
        Some(&funder.pubkey()),
        &[&funder],
        context.last_blockhash,
    );

    context.banks_client.process_transaction(tx).await.unwrap();

    let escrow_account = context.banks_client.get_account(escrow).await.unwrap();
    assert!(escrow_account.is_none());

    let order_account = context
        .banks_client
        .get_account(order.pubkey())
        .await
        .unwrap()
        .unwrap();

    let funder_token_account_state = context
        .banks_client
        .get_account(funder_token_account.pubkey())
        .await
        .unwrap()
        .unwrap();

    let funder_token_account_balance_after = token::TokenAccount::try_deserialize_unchecked(
        &mut funder_token_account_state.data.as_ref(),
    )
    .unwrap()
    .amount;

    assert!(funder_token_account_balance_after > funder_token_account_balance_before);

    let order = try_from_slice_unchecked::<p2swap::state::Order>(&order_account.data[8..]).unwrap();
    assert_eq!(order.status, p2swap::state::OrderStatus::Canceled);
}

#[tokio::test]
async fn fail_invalid_order_status() {
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

    context.warp_to_slot(3).unwrap();

    let accounts = p2swap::accounts::CancelOrder {
        order: order.pubkey(),
        funder: funder.pubkey(),
        funder_token_account: funder.pubkey(),
        escrow,
        token_program: spl_token::id(),
        system_program: System::id(),
    }
    .to_account_metas(None);

    let data = p2swap::instruction::CancelOrder { escrow_bump }.data();

    let instruction = Instruction {
        program_id: p2swap::id(),
        data,
        accounts,
    };

    let tx = Transaction::new_signed_with_payer(
        &[instruction],
        Some(&funder.pubkey()),
        &[&funder],
        context.last_blockhash,
    );

    context.banks_client.process_transaction(tx).await.unwrap();

    context.warp_to_slot(6).unwrap();

    let accounts = p2swap::accounts::CancelOrder {
        order: order.pubkey(),
        funder: funder.pubkey(),
        funder_token_account: funder.pubkey(),
        escrow,
        token_program: spl_token::id(),
        system_program: System::id(),
    }
    .to_account_metas(None);

    let data = p2swap::instruction::CancelOrder { escrow_bump }.data();

    let instruction = Instruction {
        program_id: p2swap::id(),
        data,
        accounts,
    };

    let tx = Transaction::new_signed_with_payer(
        &[instruction],
        Some(&funder.pubkey()),
        &[&funder],
        context.last_blockhash,
    );

    let error = context
        .banks_client
        .process_transaction(tx)
        .await
        .unwrap_err();
    let error_code = utils::get_error_code(error);
    if let Some(error_code) = error_code {
        if error_code == 6008 {
            assert_eq!(true, true);
            return;
        }
    }
    assert_eq!(true, false);
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

    context.warp_to_slot(3).unwrap();

    let accounts = p2swap::accounts::CancelOrder {
        order: order.pubkey(),
        funder: funder.pubkey(),
        funder_token_account: order.pubkey(),
        escrow,
        token_program: spl_token::id(),
        system_program: System::id(),
    }
    .to_account_metas(None);

    let data = p2swap::instruction::CancelOrder { escrow_bump }.data();

    let instruction = Instruction {
        program_id: p2swap::id(),
        data,
        accounts,
    };

    let tx = Transaction::new_signed_with_payer(
        &[instruction],
        Some(&funder.pubkey()),
        &[&funder],
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
