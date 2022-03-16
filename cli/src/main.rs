mod args;
mod error;
mod processor;
mod utils;

use anchor_client::anchor_lang::{Id, System};
use chrono::Utc;
use clap::Parser;
use indicatif;
use solana_client::rpc_client::RpcClient;
use solana_sdk::{signature::read_keypair_file, signer::Signer};
use std::{env, time::Duration};

fn main() -> Result<(), error::Error> {
    let args = args::Args::parse();

    let wallet = read_keypair_file(args.wallet.unwrap_or(format!(
        "{}/.config/solana/id.json",
        env::var("HOME").unwrap()
    )))
    .unwrap();

    let client = RpcClient::new(
        args.url
            .unwrap_or(String::from("https://api.devnet.solana.com")),
    );

    match args.command {
        args::Commands::GetOrder { order } => {
            let pb = indicatif::ProgressBar::new_spinner();
            pb.enable_steady_tick(Duration::from_millis(120).as_millis() as u64);
            pb.set_message("Obtaining order..");

            let order_pubkey = order;
            let order = utils::get_order(&client, &order)?;

            pb.finish_and_clear();

            utils::print_order(&client, &order_pubkey, &order)?;
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

            let pb = indicatif::ProgressBar::new_spinner();
            pb.enable_steady_tick(Duration::from_millis(120).as_millis() as u64);
            pb.set_message("Creating order..");

            let (order_pubkey, tx) = processor::create_order(
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
                    Utc::now()
                        .checked_add_signed(chrono::Duration::hours(1))
                        .unwrap()
                        .timestamp(),
                ),
            )?;

            pb.finish_and_clear();

            println!("[+] Order created: {}, tx: {}", order_pubkey, tx);
        }
        args::Commands::CancelOrder {
            order,
            token_account,
        } => {
            let pb = indicatif::ProgressBar::new_spinner();
            pb.enable_steady_tick(Duration::from_millis(120).as_millis() as u64);
            pb.set_message("Canceling order..");

            let tx = processor::cancel_order(
                &client,
                &wallet,
                &order,
                &token_account.unwrap_or(wallet.pubkey()),
            )?;

            pb.finish_and_clear();

            println!("[+] Order canceled, tx: {}", tx);
        }
        args::Commands::ExecuteOrder {
            order,
            token_account,
            receive_token_account,
        } => {
            let pb = indicatif::ProgressBar::new_spinner();
            pb.enable_steady_tick(Duration::from_millis(120).as_millis() as u64);
            pb.set_message("Executing order..");

            let order_state = utils::get_order(&client, &order)?;

            let tx = processor::execute_order(
                &client,
                &wallet,
                &order,
                &order_state.funder,
                &token_account.unwrap_or(wallet.pubkey()),
                &receive_token_account.unwrap_or(wallet.pubkey()),
                &order_state.quote_token_account,
            )?;

            pb.finish_and_clear();

            println!("[+] Order executed, tx: {}", tx);
        }
        args::Commands::GetOrdersHistory { funder, status } => {
            let pb = indicatif::ProgressBar::new_spinner();
            pb.enable_steady_tick(Duration::from_millis(120).as_millis() as u64);
            pb.set_message("Obtaining orders history..");

            let order_status = if let Some(status) = status {
                match status {
                    args::OrderStatusArg::Created => Some(p2swap::state::OrderStatus::Created),
                    args::OrderStatusArg::Canceled => Some(p2swap::state::OrderStatus::Canceled),
                    args::OrderStatusArg::Completed => Some(p2swap::state::OrderStatus::Created),
                }
            } else {
                None
            };

            let orders = utils::get_orders_history(
                &client,
                &funder.unwrap_or(wallet.pubkey()),
                order_status,
            )?;

            pb.finish_and_clear();

            for (order_pubkey, order) in orders {
                utils::print_order(&client, &order_pubkey, &order)?;
                println!();
            }
        }
    }

    Ok(())
}
