use clap::{Parser, Subcommand};
use solana_sdk::pubkey::Pubkey;

#[derive(Subcommand, Debug)]
pub enum Commands {
    CreateOrder {
        #[clap(long)]
        recipient: Pubkey,

        #[clap(short, long)]
        token_account: Option<Pubkey>,

        #[clap(short, long)]
        quote_token_account: Option<Pubkey>,

        #[clap(short, long)]
        base_mint: Option<Pubkey>,

        #[clap(long)]
        quote_mint: Option<Pubkey>,

        #[clap(long)]
        base_amount: f64,

        #[clap(long)]
        quote_amount: f64,

        #[clap(long)]
        start_date: Option<i64>,

        #[clap(long)]
        expire_date: Option<i64>,
    },
    CancelOrder {
        #[clap(short, long)]
        order: Pubkey,

        #[clap(short, long)]
        token_account: Option<Pubkey>,
    },
    ExecuteOrder {
        #[clap(short, long)]
        order: Pubkey,

        #[clap(short, long)]
        token_account: Option<Pubkey>,

        #[clap(short, long)]
        receive_token_account: Option<Pubkey>,
    },
    GetOrder {
        #[clap(short, long)]
        order: Pubkey,
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
