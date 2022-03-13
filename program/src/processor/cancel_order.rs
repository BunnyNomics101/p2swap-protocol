use crate::{error, state, utils, CancelOrder};
use anchor_lang::prelude::*;
use anchor_spl::token;

impl<'info> CancelOrder<'info> {
    pub fn process(&mut self, escrow_bump: u8) -> Result<()> {
        if self.order.status != state::OrderStatus::Created {
            return Err(error::ErrorCode::InvalidOrderStatus.into());
        }

        self.order.status = state::OrderStatus::Canceled;

        // Delete `escrow` account
        if self.order.is_base_native {
            if self.funder_token_account.key != self.funder.key {
                return Err(error::ErrorCode::FunderAccountWalletMismatch.into());
            }

            utils::delete_account(
                &self.escrow.to_account_info(),
                &self.funder.to_account_info(),
            )?;
        } else {
            // Transfer `funder` tokens from `escrow`
            let escrow_token_account = token::TokenAccount::try_deserialize_unchecked(
                &mut self.escrow.data.borrow().as_ref(),
            )?;

            let order_key = self.order.key();

            let signer_seeds: &[&[&[u8]]] = &[&[
                utils::ORDER_ESCROW_PREFIX.as_bytes(),
                self.funder.key.as_ref(),
                order_key.as_ref(),
                &[escrow_bump],
            ]];

            let cpi_program = self.token_program.to_account_info();
            let cpi_accounts = token::Transfer {
                from: self.escrow.to_account_info(),
                to: self.funder_token_account.to_account_info(),
                authority: self.escrow.to_account_info(),
            };
            let cpi_ctx = CpiContext::new_with_signer(cpi_program, cpi_accounts, signer_seeds);
            token::transfer(cpi_ctx, escrow_token_account.amount)?;

            // Close `escrow` account
            let cpi_program = self.token_program.to_account_info();
            let cpi_accounts = token::CloseAccount {
                account: self.escrow.to_account_info(),
                destination: self.funder.to_account_info(),
                authority: self.escrow.to_account_info(),
            };
            let cpi_ctx = CpiContext::new_with_signer(cpi_program, cpi_accounts, signer_seeds);
            token::close_account(cpi_ctx)?;
        }

        Ok(())
    }
}
