//! Module provide application utils.

use crate::error;
use solana_client::rpc_client::RpcClient;
use solana_sdk::{borsh::try_from_slice_unchecked, program_pack::Pack, pubkey::Pubkey};

/// Return p2swap `Order` state.
pub fn get_order(client: &RpcClient, order: &Pubkey) -> Result<p2swap::state::Order, error::Error> {
    let data = client.get_account_data(order)?;

    let order = try_from_slice_unchecked::<p2swap::state::Order>(&data[8..])?;

    Ok(order)
}

/// Return `spl_token` `Mint` state.
pub fn get_mint(client: &RpcClient, mint: &Pubkey) -> Result<spl_token::state::Mint, error::Error> {
    let data = client.get_account_data(mint)?;
    Ok(spl_token::state::Mint::unpack(&data)?)
}
