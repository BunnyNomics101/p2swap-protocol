use anchor_client::anchor_lang::{Id, InstructionData, System, ToAccountMetas};
use solana_client::rpc_client::RpcClient;
use solana_sdk::{
    instruction::Instruction,
    pubkey::Pubkey,
    signature::{Keypair, Signer},
    transaction::Transaction,
};
use std::error;

pub fn cancel_order(
    client: &RpcClient,
    wallet: &Keypair,
    order: &Pubkey,
    token_account: &Pubkey,
) -> Result<(), Box<dyn error::Error>> {
    let (escrow, escrow_bump) = p2swap::utils::find_order_escrow_address(&wallet.pubkey(), order);

    let accounts = p2swap::accounts::CancelOrder {
        order: order.clone(),
        funder: wallet.pubkey(),
        funder_token_account: token_account.clone(),
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

    let last_blockhash = client.get_latest_blockhash()?;

    let tx = Transaction::new_signed_with_payer(
        &[instruction],
        Some(&wallet.pubkey()),
        &[wallet],
        last_blockhash,
    );

    client.send_and_confirm_transaction(&tx)?;

    Ok(())
}
