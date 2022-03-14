mod args;
mod processor;
mod utils;

use anchor_client::anchor_lang::{Id, System};
use chrono::Utc;
use clap::Parser;
use solana_client::rpc_client::RpcClient;
use solana_sdk::{signature::read_keypair_file, signer::Signer};
use std::{env, error};

fn main() -> Result<(), Box<dyn error::Error>> {
    let args = args::Args::parse();

    let wallet = read_keypair_file(args.wallet.unwrap_or(format!(
        "{}/.config/solana/id.json",
        env::var("HOME").unwrap()
    )))?;

    let client = RpcClient::new(
        args.url
            .unwrap_or(String::from("https://api.devnet.solana.com")),
    );

    match args.command {
        args::Commands::GetOrder { order } => {
            let order = utils::get_order(&client, &order)?;
            println!("{:#?}", order);
        }
        args::Commands::CreateOrder {
            recipient,
            token_account,
            quote_token_account,
            base_mint,
            quote_mint,
            base_amount,
            quote_amount,
            start_date,
            expire_date,
        } => {
            let base_decimals = if base_mint.is_none() {
                9
            } else {
                let mint = utils::get_mint(&client, &base_mint.unwrap())?;
                mint.decimals
            };

            let quote_decimals = if quote_mint.is_none() {
                9
            } else {
                let mint = utils::get_mint(&client, &quote_mint.unwrap())?;
                mint.decimals
            };

            let last_time = Utc::now();

            let order = processor::create_order(
                &client,
                &wallet,
                &recipient,
                &token_account.unwrap_or(wallet.pubkey()),
                &quote_token_account.unwrap_or(wallet.pubkey()),
                &base_mint.unwrap_or(System::id()),
                &quote_mint.unwrap_or(System::id()),
                spl_token::ui_amount_to_amount(base_amount, base_decimals),
                spl_token::ui_amount_to_amount(quote_amount, quote_decimals),
                start_date,
                expire_date.unwrap_or(
                    last_time
                        .checked_add_signed(chrono::Duration::hours(1))
                        .unwrap()
                        .timestamp(),
                ),
            )?;

            println!("[+] New order: {}", order);
        }
        args::Commands::CancelOrder {
            order,
            token_account,
        } => {
            processor::cancel_order(
                &client,
                &wallet,
                &order,
                &token_account.unwrap_or(wallet.pubkey()),
            )?;

            println!("[+] Order canceled");
        }
        args::Commands::ExecuteOrder {
            order,
            token_account,
            receive_token_account,
        } => {
            let order_state = utils::get_order(&client, &order)?;

            processor::execute_order(
                &client,
                &wallet,
                &order,
                &order_state.funder,
                &token_account.unwrap_or(wallet.pubkey()),
                &receive_token_account.unwrap_or(wallet.pubkey()),
                &order_state.quote_token_account,
            )?;

            println!("[+] Order executed");
        }
    }

    Ok(())
}
