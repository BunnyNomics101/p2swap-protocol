use anchor_lang::prelude::*;

#[error_code]
pub enum ErrorCode {
    #[msg("Quote account mismatch funder wallet")]
    QuoteAccountWalletMismatch,

    #[msg("Funder account mismatch wallet")]
    FunderAccountWalletMismatch,

    #[msg("Recipient account mismatch wallet")]
    RecipientAccountWalletMismatch,

    #[msg("Recipient receive account mismatch wallet")]
    RecipientReceiveAccountWalletMismatch,

    #[msg("Order is expired")]
    OrderIsExpired,

    #[msg("Order is not started")]
    OrderIsNotStarted,

    #[msg("Expire date in the past")]
    ExpireDateInThePast,

    #[msg("Start date in the past")]
    StartDateInThePast,

    #[msg("Invalid order status")]
    InvalidOrderStatus,
}
