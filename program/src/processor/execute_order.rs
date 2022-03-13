use crate::{error, state, utils, ExecuteOrder};
use anchor_lang::prelude::*;
use anchor_spl::token;

impl<'info> ExecuteOrder<'info> {
    pub fn process(&mut self, escrow_bump: u8) -> Result<()> {
        if self.order.status != state::OrderStatus::Created {
            return Err(error::ErrorCode::InvalidOrderStatus.into());
        }

        self.order.status = state::OrderStatus::Completed;

        // Here `quote_token_account` is not checked for `funder` wallet
        // when `Order::is_quote_native`, because check was occur in `CreateOrder`
        // instruction

        // Transfer funds from `recipient` to `funder` (`quote_token_account`).
        if self.order.is_quote_native {
            if self.recipient_token_account.key != self.recipient.key {
                return Err(error::ErrorCode::RecipientAccountWalletMismatch.into());
            }

            utils::sys_transfer(
                &self.recipient_token_account.to_account_info(),
                &self.quote_token_account.to_account_info(),
                self.order.quote_amount,
                &[],
            )?;
        } else {
            let cpi_program = self.token_program.to_account_info();
            let cpi_accounts = token::Transfer {
                from: self.recipient_token_account.to_account_info(),
                to: self.quote_token_account.to_account_info(),
                authority: self.recipient.to_account_info(),
            };
            let cpi_ctx = CpiContext::new_with_signer(cpi_program, cpi_accounts, &[]);
            token::transfer(cpi_ctx, self.order.quote_amount)?;
        }

        // Transfer funds from `escrow` to `recipient` (`recipient_receive_token_account`).
        if self.order.is_base_native {
            if self.recipient_receive_token_account.key != self.recipient.key {
                return Err(error::ErrorCode::RecipientReceiveAccountWalletMismatch.into());
            }

            utils::move_lamports(
                &self.escrow.to_account_info(),
                &self.recipient_receive_token_account.to_account_info(),
                self.order.base_amount,
            )?;
        } else {
            let order_key = self.order.key();

            let signer_seeds: &[&[&[u8]]] = &[&[
                utils::ORDER_ESCROW_PREFIX.as_bytes(),
                self.order.funder.as_ref(),
                order_key.as_ref(),
                &[escrow_bump],
            ]];

            let cpi_program = self.token_program.to_account_info();
            let cpi_accounts = token::Transfer {
                from: self.escrow.to_account_info(),
                to: self.recipient_receive_token_account.to_account_info(),
                authority: self.escrow.to_account_info(),
            };
            let cpi_ctx = CpiContext::new_with_signer(cpi_program, cpi_accounts, signer_seeds);
            token::transfer(cpi_ctx, self.order.base_amount)?;
        }

        // Delete `escrow` account
        if self.order.is_base_native {
            utils::delete_account(
                &self.escrow.to_account_info(),
                &self.funder.to_account_info(),
            )?;
        } else {
            let order_key = self.order.key();

            let signer_seeds: &[&[&[u8]]] = &[&[
                utils::ORDER_ESCROW_PREFIX.as_bytes(),
                self.order.funder.as_ref(),
                order_key.as_ref(),
                &[escrow_bump],
            ]];

            let cpi_program = self.token_program.to_account_info();
            let cpi_accounts = token::CloseAccount {
                account: self.escrow.to_account_info(),
                destination: self.funder.to_account_info(),
                authority: self.escrow.to_account_info(),
            };
            let cpi_ctx = CpiContext::new_with_signer(cpi_program, cpi_accounts, signer_seeds);
            token::close_account(cpi_ctx)?;
        }

        if self.clock_sysvar.unix_timestamp > self.order.expire_date {
            return Err(error::ErrorCode::OrderIsExpired.into());
        }

        if let Some(start_date) = self.order.start_date {
            if start_date > self.clock_sysvar.unix_timestamp {
                return Err(error::ErrorCode::OrderIsNotStarted.into());
            }
        }

        Ok(())
    }
}
