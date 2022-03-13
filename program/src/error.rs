use anchor_lang::prelude::*;

#[error_code]
pub enum ErrorCode {
    /// 6000.
    #[msg("Quote account mismatch funder wallet")]
    QuoteAccountWalletMismatch,

    /// 6001.
    #[msg("Funder account mismatch wallet")]
    FunderAccountWalletMismatch,

    /// 6002.
    #[msg("Recipient account mismatch wallet")]
    RecipientAccountWalletMismatch,

    /// 6003.
    #[msg("Recipient receive account mismatch wallet")]
    RecipientReceiveAccountWalletMismatch,

    /// 6004.
    #[msg("Order is expired")]
    OrderIsExpired,

    /// 6005.
    #[msg("Order is not started")]
    OrderIsNotStarted,

    /// 6006.
    #[msg("Expire date in the past")]
    ExpireDateInThePast,

    /// 6007.
    #[msg("Start date in the past")]
    StartDateInThePast,

    /// 6008.
    #[msg("Invalid order status")]
    InvalidOrderStatus,
}
