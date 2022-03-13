use anchor_lang::{prelude::*, solana_program::clock::UnixTimestamp};

#[derive(AnchorSerialize, AnchorDeserialize, Clone, Debug, PartialEq, Eq)]
pub enum OrderStatus {
    Created,
    Canceled,
    Completed,
}

/// TODO: Provide docs.

#[account]
pub struct Order {
    pub status: OrderStatus,
    pub base_amount: u64,
    pub quote_amount: u64,
    pub is_base_native: bool,
    pub is_quote_native: bool,
    pub funder: Pubkey,
    pub recipient: Pubkey,
    pub escrow: Pubkey,
    pub quote_token_account: Pubkey,
    pub start_date: Option<UnixTimestamp>,
    pub expire_date: UnixTimestamp,
}

impl Order {
    pub const LEN: usize = 8 + 1 + 8 + 8 + 1 + 1 + 32 + 32 + 32 + 32 + 9 + 8;
}
