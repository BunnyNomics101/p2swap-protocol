//! Module provide `ExecuteOrder` instruction handler.

use crate::error;
use anchor_client::anchor_lang::{Id, InstructionData, System, ToAccountMetas};
use solana_client::rpc_client::RpcClient;
use solana_sdk::{
    instruction::Instruction,
    pubkey::Pubkey,
    signature::Signature,
    signature::{Keypair, Signer},
    sysvar,
    transaction::Transaction,
};

/// Handler.
pub fn execute_order(
    client: &RpcClient,
    wallet: &Keypair,
    order: &Pubkey,
    funder: &Pubkey,
    token_account: &Pubkey,
    receive_token_account: &Pubkey,
    quote_token_account: &Pubkey,
) -> Result<Signature, error::Error> {
    let (escrow, escrow_bump) = p2swap::utils::find_order_escrow_address(funder, order);

    let accounts = p2swap::accounts::ExecuteOrder {
        order: order.clone(),
        recipient: wallet.pubkey(),
        recipient_token_account: token_account.clone(),
        recipient_receive_token_account: receive_token_account.clone(),
        quote_token_account: quote_token_account.clone(),
        clock_sysvar: sysvar::clock::id(),
        funder: funder.clone(),
        escrow,
        token_program: spl_token::id(),
        system_program: System::id(),
    }
    .to_account_metas(None);

    let data = p2swap::instruction::ExecuteOrder { escrow_bump }.data();

    let instruction = Instruction {
        program_id: p2swap::id(),
        data,
        accounts,
    };

    let last_blockhash = client.get_latest_blockhash()?;

    let tx = Transaction::new_signed_with_payer(
        &[instruction],
        Some(&wallet.pubkey()),
        &[wallet],
        last_blockhash,
    );

    Ok(client.send_and_confirm_transaction(&tx)?)
}
