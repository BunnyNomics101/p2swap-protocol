pub mod error;
mod processor;
pub mod state;
pub mod utils;

use anchor_lang::{prelude::*, solana_program::clock::UnixTimestamp};
use anchor_spl::token::Token;

declare_id!("p2sSQ51hNP1yiQKtZ81sDzWBM6tkaS2ZtMrgnbBhE4v");

#[program]
mod p2swap {
    use super::*;

    pub fn create_order(
        ctx: Context<CreateOrder>,
        escrow_bump: u8,
        base_amount: u64,
        quote_amount: u64,
        start_date: Option<UnixTimestamp>,
        expire_date: UnixTimestamp,
    ) -> Result<()> {
        ctx.accounts.process(
            escrow_bump,
            base_amount,
            quote_amount,
            start_date,
            expire_date,
        )
    }

    pub fn cancel_order(ctx: Context<CancelOrder>, escrow_bump: u8) -> Result<()> {
        ctx.accounts.process(escrow_bump)
    }

    pub fn execute_order(ctx: Context<ExecuteOrder>, escrow_bump: u8) -> Result<()> {
        ctx.accounts.process(escrow_bump)
    }
}

/// Perform p2p swap `Order` creation and initialization.
///
/// `base_amount` - quantity of tokens, that `funder` will give to `recipient`.
///
/// `quote_amount` - quantity of tokens, that `recipient` will give to `funder`.
///
/// `start_date` - the date from which payments will be accepted.
///
/// `expire_date` - the date from which `Order` will be expired.
#[derive(Accounts)]
#[instruction(escrow_bump: u8, base_amount: u64, quote_amount: u64, start_date: Option<UnixTimestamp>, expire_date: UnixTimestamp)]
pub struct CreateOrder<'info> {
    #[account(init, space=state::Order::LEN, payer=funder)]
    order: Box<Account<'info, state::Order>>,

    /// Funder represent `Order` initiator(creator).
    #[account(mut)]
    funder: Signer<'info>,

    /// Recipient represent `Order` participant(who will accept offer).
    recipient: UncheckedAccount<'info>,

    /// `funder` token account.
    /// Should be `funder` wallet if `Order::is_base_native`.
    /// Otherwise `spl_token` account should be passed.
    #[account(mut)]
    funder_token_account: UncheckedAccount<'info>,

    /// Will hold `funder`'s tokens to provide guarantee(PDA, uninitialized).
    ///
    /// PDA: [ORDER_ESCROW_PREFIX, funder_pubkey, order_pubkey].
    #[account(mut, seeds = [utils::ORDER_ESCROW_PREFIX.as_bytes(), funder.key().as_ref(), order.key().as_ref()], bump=escrow_bump)]
    escrow: UncheckedAccount<'info>,

    /// Will directly accept `recipient`'s tokens and send them to `funder` (`funder`'s token account).
    #[account(mut)]
    quote_token_account: UncheckedAccount<'info>,

    /// Mint of `escrow`.
    /// If base tokens are native `SOL`'s, then this field should eq to `System::id()`.
    escrow_mint: UncheckedAccount<'info>,

    /// Mint of `quote_token_account`.
    /// If quote tokens are native `SOL`'s, then this field should eq to `System::id()`.
    quote_mint: UncheckedAccount<'info>,

    rent_sysvar: Sysvar<'info, Rent>,
    clock_sysvar: Sysvar<'info, Clock>,
    token_program: Program<'info, Token>,
    system_program: Program<'info, System>,
}

/// Perform p2p swap `Order` cancel.
#[derive(Accounts)]
#[instruction(escrow_bump: u8)]
pub struct CancelOrder<'info> {
    #[account(mut, has_one = funder, has_one = escrow)]
    order: Box<Account<'info, state::Order>>,

    /// Funder represent `Order` initiator(creator).
    #[account(mut)]
    funder: Signer<'info>,

    /// `funder` token account (will receive `escrow` tokens).
    /// Should be `funder` wallet if `Order::is_base_native`.
    /// Otherwise `spl_token` account should be passed.
    #[account(mut)]
    funder_token_account: UncheckedAccount<'info>,

    /// Will hold `funder`'s tokens to provide guarantee(PDA, uninitialized).
    ///
    /// PDA: [ORDER_ESCROW_PREFIX, funder_pubkey, order_pubkey].
    #[account(mut, seeds = [utils::ORDER_ESCROW_PREFIX.as_bytes(), funder.key().as_ref(), order.key().as_ref()], bump=escrow_bump)]
    escrow: UncheckedAccount<'info>,

    token_program: Program<'info, Token>,
    system_program: Program<'info, System>,
}

/// Perform p2p swap `Order` execute.
#[derive(Accounts)]
#[instruction(escrow_bump: u8)]
pub struct ExecuteOrder<'info> {
    #[account(mut, has_one = recipient, has_one = escrow, has_one = quote_token_account, has_one = funder)]
    order: Box<Account<'info, state::Order>>,

    /// Funder represent `Order` initiator(creator).
    #[account(mut)]
    funder: UncheckedAccount<'info>,

    /// Recipient represent `Order` participant(who will accept offer).
    #[account(mut)]
    recipient: Signer<'info>,

    /// `recipient` token account (tokens input).
    /// Should be `recipient` wallet if `Order::is_quote_native`.
    /// Otherwise `spl_token` account should be passed.
    #[account(mut)]
    recipient_token_account: UncheckedAccount<'info>,

    /// `recipient` token account for receiving from `escrow` (tokens output).
    /// Should be `recipient` wallet if `Order::is_base_native`.
    /// Otherwise `spl_token` account should be passed.
    #[account(mut)]
    recipient_receive_token_account: UncheckedAccount<'info>,

    /// Will hold `funder`'s tokens to provide guarantee(PDA, uninitialized).
    ///
    /// PDA: [ORDER_ESCROW_PREFIX, funder_pubkey, order_pubkey].
    #[account(mut, seeds = [utils::ORDER_ESCROW_PREFIX.as_bytes(), order.funder.as_ref(), order.key().as_ref()], bump=escrow_bump)]
    escrow: UncheckedAccount<'info>,

    /// Will directly accept `recipient`'s tokens and send them to `funder` (`funder`'s token account).
    /// Should be `funder` wallet if `Order::is_quote_native`.
    /// Otherwise `spl_token` account should be passed.
    #[account(mut)]
    quote_token_account: UncheckedAccount<'info>,

    clock_sysvar: Sysvar<'info, Clock>,
    token_program: Program<'info, Token>,
    system_program: Program<'info, System>,
}
