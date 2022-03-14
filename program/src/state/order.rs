use anchor_lang::{prelude::*, solana_program::clock::UnixTimestamp};

#[derive(AnchorSerialize, AnchorDeserialize, Clone, Debug, PartialEq, Eq)]
pub enum OrderStatus {
    Created,
    Canceled,
    Completed,
}

#[account]
#[derive(Debug)]
pub struct Order {
    /// Current `Order` status.
    pub status: OrderStatus,

    /// Amount, that will be sended by `funder`.
    pub base_amount: u64,

    /// Amount, that will be sended by `recipient`.
    pub quote_amount: u64,

    /// Indicate `escrow` mint.
    pub base_mint: Pubkey,

    /// Indicate `quote_token_account` mint.
    pub quote_mint: Pubkey,

    /// Creator, swap initiator.
    pub funder: Pubkey,

    /// Participant, that swap tokens with `funder`.
    pub recipient: Pubkey,

    /// Guarantee pool, that hold `funder` tokens and send them to `recipient`.
    pub escrow: Pubkey,

    /// Token account (owned by `funder`), that will receive tokens from `recipient`.
    pub quote_token_account: Pubkey,

    /// Start date.
    /// If `None`, then `Order` starts immediately after creation.
    pub start_date: Option<UnixTimestamp>,

    /// Expire date.
    pub expire_date: UnixTimestamp,
}

impl Order {
    pub const LEN: usize = 8 + 1 + 8 + 8 + 32 + 32 + 32 + 32 + 32 + 32 + 9 + 8;

    pub fn is_base_native(&self) -> bool {
        self.base_mint == System::id()
    }

    pub fn is_quote_native(&self) -> bool {
        self.quote_mint == System::id()
    }
}
