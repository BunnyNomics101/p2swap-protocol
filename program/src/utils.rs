use crate::id;
use anchor_lang::{
    prelude::*,
    solana_program::{program::invoke_signed, system_instruction},
};

pub const ORDER_ESCROW_PREFIX: &str = "p2s_order_escrow";
pub const ORDER_ESCROW_NATIVE_SIZE: usize = 8 + 1;

/// Return `Order` tokens escrow `Pubkey` and bump seed.
pub fn find_order_escrow_address(funder_wallet: &Pubkey, order: &Pubkey) -> (Pubkey, u8) {
    Pubkey::find_program_address(
        &[
            ORDER_ESCROW_PREFIX.as_bytes(),
            funder_wallet.as_ref(),
            order.as_ref(),
        ],
        &id(),
    )
}

/// Move lamports from `src` to `dst` account.
#[inline(always)]
pub fn move_lamports<'a>(
    src: &AccountInfo<'a>,
    dst: &AccountInfo<'a>,
    lamports: u64,
) -> Result<()> {
    let mut src_lamports = src.try_borrow_mut_lamports()?;
    let mut dst_lamports = dst.try_borrow_mut_lamports()?;

    **src_lamports -= lamports;
    **dst_lamports += lamports;

    Ok(())
}

/// Delete `target` account, transfer all lamports to `receiver`.
#[inline(always)]
pub fn delete_account<'a>(target: &AccountInfo<'a>, receiver: &AccountInfo<'a>) -> Result<()> {
    let mut target_lamports = target.try_borrow_mut_lamports()?;
    let mut receiver_lamports = receiver.try_borrow_mut_lamports()?;

    **receiver_lamports += **target_lamports;
    **target_lamports = 0;

    Ok(())
}

/// Wrapper of `transfer` instruction from `system_program` program.
#[inline(always)]
pub fn sys_transfer<'a>(
    from: &AccountInfo<'a>,
    to: &AccountInfo<'a>,
    lamports: u64,
    signer_seeds: &[&[u8]],
) -> Result<()> {
    invoke_signed(
        &system_instruction::transfer(from.key, to.key, lamports),
        &[from.clone(), to.clone()],
        &[&signer_seeds],
    )?;

    Ok(())
}

/// Wrapper of `create_account` instruction from `system_program` program.
#[inline(always)]
pub fn sys_create_account<'a>(
    from: &AccountInfo<'a>,
    to: &AccountInfo<'a>,
    lamports: u64,
    space: usize,
    owner: &Pubkey,
    signer_seeds: &[&[u8]],
) -> Result<()> {
    invoke_signed(
        &system_instruction::create_account(from.key, to.key, lamports, space as u64, owner),
        &[from.clone(), to.clone()],
        &[&signer_seeds],
    )?;

    Ok(())
}
