//! Module provide CLI arguments parser.

use clap::{ArgEnum, Parser, Subcommand};
use solana_sdk::pubkey::Pubkey;

#[derive(Debug, ArgEnum, Clone)]
pub enum OrderStatusArg {
    Created,
    Canceled,
    Completed,
}

#[derive(Subcommand, Debug)]
pub enum Commands {
    CreateOrder {
        #[clap(long, help = "recipient address")]
        recipient: Pubkey,

        #[clap(short, long, help = "funder token account address")]
        token_account: Option<Pubkey>,

        #[clap(short, long, help = "funder token account address for receiving")]
        quote_token_account: Option<Pubkey>,

        #[clap(short, long, help = "mint for funder tokens")]
        base_mint: Option<Pubkey>,

        #[clap(long, help = "mint for recipient tokens")]
        quote_mint: Option<Pubkey>,

        #[clap(long, help = "funder tokens amount")]
        base_amount: f64,

        #[clap(long, help = "recipient tokens amount")]
        quote_amount: f64,

        #[clap(long, help = "order start date")]
        start_date: Option<i64>,

        #[clap(long, help = "order expiration date")]
        expire_date: Option<i64>,
    },
    CancelOrder {
        #[clap(short, long, help = "order address")]
        order: Pubkey,

        #[clap(
            short,
            long,
            help = "funder token account address for escrow tokens receiving"
        )]
        token_account: Option<Pubkey>,
    },
    ExecuteOrder {
        #[clap(short, long, help = "order address")]
        order: Pubkey,

        #[clap(short, long, help = "signer token account address")]
        token_account: Option<Pubkey>,

        #[clap(short, long, help = "signer token account address for receive")]
        receive_token_account: Option<Pubkey>,
    },
    GetOrder {
        #[clap(short, long, help = "order address")]
        order: Pubkey,
    },
    GetOrdersHistory {
        #[clap(short, long, help = "funder address")]
        funder: Option<Pubkey>,

        #[clap(short, long, arg_enum, help = "order status")]
        status: Option<OrderStatusArg>,
    },
}

#[derive(Parser, Debug)]
#[clap(author = "b3zrazli4n0")]
#[clap(about = "CLI utility for p2swap program")]
#[clap(version, long_about = None)]
pub struct Args {
    #[clap(short, long, help = "Specifies cluster url")]
    pub url: Option<String>,

    #[clap(short, long, help = "Local wallet path")]
    pub wallet: Option<String>,

    #[clap(subcommand)]
    pub command: Commands,
}
