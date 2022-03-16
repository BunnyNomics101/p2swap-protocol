//! Module provide application utils.

use crate::error;
use solana_account_decoder::{UiAccountEncoding, UiDataSliceConfig};
use solana_client::{
    rpc_client::RpcClient,
    rpc_config::{RpcAccountInfoConfig, RpcProgramAccountsConfig},
    rpc_filter::{Memcmp, MemcmpEncodedBytes, RpcFilterType},
};
use solana_sdk::{borsh::try_from_slice_unchecked, program_pack::Pack, pubkey::Pubkey};

/// Return p2swap `Order` state.
pub fn get_order(client: &RpcClient, order: &Pubkey) -> Result<p2swap::state::Order, error::Error> {
    let data = client.get_account_data(order)?;

    let order = try_from_slice_unchecked::<p2swap::state::Order>(&data[8..])?;

    Ok(order)
}

/// Return p2swap `Order` history for specific `funder` and `order_status`.
pub fn get_orders_history(
    client: &RpcClient,
    funder: &Pubkey,
    order_status: Option<p2swap::state::OrderStatus>,
) -> Result<Vec<(Pubkey, p2swap::state::Order)>, error::Error> {
    let mut filters = vec![
        RpcFilterType::DataSize(p2swap::state::Order::LEN as u64),
        RpcFilterType::Memcmp(Memcmp {
            offset: 89,
            bytes: MemcmpEncodedBytes::Base58(bs58::encode(funder).into_string()),
            encoding: None,
        }),
    ];

    if let Some(order_status) = order_status {
        filters.push(RpcFilterType::Memcmp(Memcmp {
            offset: 8,
            bytes: MemcmpEncodedBytes::Base58(bs58::encode(vec![order_status as u8]).into_string()),
            encoding: None,
        }));
    }

    let accounts = client.get_program_accounts_with_config(
        &p2swap::id(),
        RpcProgramAccountsConfig {
            filters: Some(filters),
            account_config: RpcAccountInfoConfig {
                encoding: Some(UiAccountEncoding::Base64),
                data_slice: Some(UiDataSliceConfig {
                    offset: 0,
                    length: p2swap::state::Order::LEN,
                }),
                commitment: None,
            },
            with_context: None,
        },
    )?;

    Ok(accounts
        .iter()
        .map(|(account_pubkey, account)| {
            (
                account_pubkey.clone(),
                try_from_slice_unchecked::<p2swap::state::Order>(&account.data[8..]).unwrap(),
            )
        })
        .collect())
}

/// Print order.
pub fn print_order(
    client: &RpcClient,
    order_pubkey: &Pubkey,
    order: &p2swap::state::Order,
) -> Result<(), error::Error> {
    let base_decimals = if order.is_base_native() {
        9
    } else {
        get_mint(&client, &order.base_mint)?.decimals
    };
    let quote_decimals = if order.is_quote_native() {
        9
    } else {
        get_mint(&client, &order.quote_mint)?.decimals
    };

    println!("pubkey: {}", order_pubkey);
    println!("status: {:?}", order.status);
    println!(
        "base_amount: {}",
        spl_token::amount_to_ui_amount(order.base_amount, base_decimals)
    );
    println!(
        "quote_amount: {}",
        spl_token::amount_to_ui_amount(order.quote_amount, quote_decimals)
    );
    println!("base_mint: {}", order.base_mint);
    println!("quote_mint: {}", order.quote_mint);
    println!("funder: {}", order.funder);
    println!("recipient: {}", order.recipient);
    println!("escrow: {}", order.escrow);
    println!("quote_token_account: {}", order.quote_token_account);
    println!("start_date: {:?}", order.start_date);
    println!("expire_date: {}", order.expire_date);

    Ok(())
}

/// Return `spl_token` `Mint` state.
pub fn get_mint(client: &RpcClient, mint: &Pubkey) -> Result<spl_token::state::Mint, error::Error> {
    let data = client.get_account_data(mint)?;
    Ok(spl_token::state::Mint::unpack(&data)?)
}
