use crate::{error, id, state, utils, CreateOrder};
use anchor_lang::{prelude::*, solana_program::clock::UnixTimestamp};
use anchor_spl::token;

impl<'info> CreateOrder<'info> {
    pub fn process(
        &mut self,
        escrow_bump: u8,
        base_amount: u64,
        quote_amount: u64,
        start_date: Option<UnixTimestamp>,
        expire_date: UnixTimestamp,
    ) -> Result<()> {
        self.order.status = state::OrderStatus::Created;

        self.order.base_amount = base_amount;
        self.order.quote_amount = quote_amount;

        self.order.is_base_native = self.escrow_mint.key == &System::id();
        self.order.is_quote_native = self.quote_mint.key == &System::id();

        self.order.funder = self.funder.key.clone();
        self.order.recipient = self.recipient.key.clone();

        self.order.escrow = self.escrow.key().clone();
        self.order.quote_token_account = self.quote_token_account.key.clone();

        self.order.start_date = start_date;
        self.order.expire_date = expire_date;

        // Check if escrow is denominated in native `SOL`'s
        if self.order.is_base_native {
            if self.funder_token_account.key != self.funder.key {
                return Err(error::ErrorCode::FunderAccountWalletMismatch.into());
            }

            // Create native escrow account to hold `funder`'s native `SOL`'s
            utils::sys_create_account(
                &self.funder.to_account_info(),
                &self.escrow.to_account_info(),
                self.rent_sysvar
                    .minimum_balance(utils::ORDER_ESCROW_NATIVE_SIZE),
                utils::ORDER_ESCROW_NATIVE_SIZE,
                &id(),
                &[
                    utils::ORDER_ESCROW_PREFIX.as_bytes(),
                    self.funder.key.as_ref(),
                    self.order.key().as_ref(),
                    &[escrow_bump],
                ],
            )?;

            // Transfer native `SOL`'s to escrow
            utils::sys_transfer(
                &self.funder_token_account.to_account_info(),
                &self.escrow.to_account_info(),
                base_amount,
                &[],
            )?;
        } else {
            // Create `spl` escrow account to hold `funder`'s `spl_token`'s
            utils::sys_create_account(
                &self.funder.to_account_info(),
                &self.escrow.to_account_info(),
                self.rent_sysvar.minimum_balance(token::TokenAccount::LEN),
                token::TokenAccount::LEN,
                &token::Token::id(),
                &[
                    utils::ORDER_ESCROW_PREFIX.as_bytes(),
                    self.funder.key.as_ref(),
                    self.order.key().as_ref(),
                    &[escrow_bump],
                ],
            )?;

            // Initialize escrow `spl_token` account
            let cpi_program = self.token_program.to_account_info();
            let cpi_accounts = token::InitializeAccount {
                account: self.escrow.to_account_info(),
                mint: self.escrow_mint.to_account_info(),
                authority: self.escrow.to_account_info(),
                rent: self.rent_sysvar.to_account_info(),
            };
            let cpi_ctx = CpiContext::new_with_signer(cpi_program, cpi_accounts, &[]);
            token::initialize_account(cpi_ctx)?;

            // Transfer base amount to `spl_token` escrow
            let cpi_program = self.token_program.to_account_info();
            let cpi_accounts = token::Transfer {
                from: self.funder_token_account.to_account_info(),
                to: self.escrow.to_account_info(),
                authority: self.funder.to_account_info(),
            };
            let cpi_ctx = CpiContext::new_with_signer(cpi_program, cpi_accounts, &[]);
            token::transfer(cpi_ctx, base_amount)?;
        }

        // Check if quote token account is valid in native `SOL` context
        if self.order.is_quote_native && self.quote_token_account.key() != self.funder.key() {
            return Err(error::ErrorCode::QuoteAccountWalletMismatch.into());
        }

        // Check expire date
        if self.clock_sysvar.unix_timestamp >= self.order.expire_date {
            return Err(error::ErrorCode::ExpireDateInThePast.into());
        }

        // Check start date
        if let Some(start_date) = self.order.start_date {
            if self.clock_sysvar.unix_timestamp > start_date {
                return Err(error::ErrorCode::StartDateInThePast.into());
            }

            if start_date >= self.order.expire_date {
                return Err(error::ErrorCode::ExpireDateInThePast.into());
            }
        }

        Ok(())
    }
}
