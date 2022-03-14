use anchor_client::anchor_lang::{Id, InstructionData, System, ToAccountMetas};
use solana_client::rpc_client::RpcClient;
use solana_sdk::{
    instruction::Instruction,
    pubkey::Pubkey,
    signature::{Keypair, Signer},
    sysvar,
    transaction::Transaction,
};
use std::error;

pub fn create_order(
    client: &RpcClient,
    wallet: &Keypair,
    recipient: &Pubkey,
    token_account: &Pubkey,
    quote_token_account: &Pubkey,
    base_mint: &Pubkey,
    quote_mint: &Pubkey,
    base_amount: u64,
    quote_amount: u64,
    start_date: Option<i64>,
    expire_date: i64,
) -> Result<Pubkey, Box<dyn error::Error>> {
    let order = Keypair::new();

    let (escrow, escrow_bump) =
        p2swap::utils::find_order_escrow_address(&wallet.pubkey(), &order.pubkey());

    let accounts = p2swap::accounts::CreateOrder {
        order: order.pubkey().clone(),
        recipient: recipient.clone(),
        quote_token_account: quote_token_account.clone(),
        escrow_mint: base_mint.clone(),
        quote_mint: quote_mint.clone(),
        rent_sysvar: sysvar::rent::id(),
        clock_sysvar: sysvar::clock::id(),
        funder: wallet.pubkey(),
        funder_token_account: token_account.clone(),
        escrow,
        token_program: spl_token::id(),
        system_program: System::id(),
    }
    .to_account_metas(None);

    let data = p2swap::instruction::CreateOrder {
        escrow_bump,
        base_amount,
        quote_amount,
        start_date,
        expire_date,
    }
    .data();

    let instruction = Instruction {
        program_id: p2swap::id(),
        data,
        accounts,
    };

    let last_blockhash = client.get_latest_blockhash()?;

    let tx = Transaction::new_signed_with_payer(
        &[instruction],
        Some(&wallet.pubkey()),
        &[wallet, &order],
        last_blockhash,
    );

    client.send_and_confirm_transaction(&tx)?;

    Ok(order.pubkey())
}
