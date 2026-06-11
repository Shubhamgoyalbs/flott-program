use anchor_lang::{
  prelude::*,
  system_program::{transfer, Transfer},
};
use crate::state::*;
use crate::constants::*;
use crate::error::ErrorCode;

#[derive(Accounts)]
pub struct WithdrawFromVault<'info> {
  #[account(mut)]
  pub owner: Signer<'info>,
  
  pub authority: SystemAccount<'info>,
  
  #[account(
    mut,
    seeds = [
      "api".as_ref(),
      "user".as_ref(),
      "vault".as_ref(),
      api_user.key().as_ref(),
    ],
    bump = api_user.vault_bump
  )]
  pub vault: SystemAccount<'info>,
  
  #[account(
    seeds = [
      "api".as_ref(),
      "user".as_ref(),
      owner.key().as_ref()
    ],
    bump = api_user.bump,
    has_one = owner @ ErrorCode::OwnerMismatch
  )]
  pub api_user: Account<'info, ApiUser>,
  
  pub system_program: Program<'info, System>,
}

impl<'info> WithdrawFromVault<'info> {
  pub fn handler(ctx: Context<WithdrawFromVault>, amount: u64) -> Result<()> {
    ctx.accounts.api_user.verify_authority(&ctx.accounts.authority.key())?;
    
    let balance_after = ctx.accounts.vault.lamports()
      .checked_sub(amount)
      .ok_or(ErrorCode::Underflow)?;
    
    require!(
      balance_after > API_USER_MIN_BALANCE,
      ErrorCode::InsufficientDeposit
    );
    
    let api_user_key = ctx.accounts.api_user.key();
    let vault_bump    = [ctx.accounts.api_user.vault_bump];
    
    let vault_seeds: &[&[u8]] = &[
      b"api",
      b"user",
      b"vault",
      api_user_key.as_ref(),
      &vault_bump,
    ];
    
    transfer(
      CpiContext::new_with_signer(
        ctx.accounts.system_program.key(),
        Transfer {
          from:   ctx.accounts.vault.to_account_info(),
          to: ctx.accounts.owner.to_account_info(),
        },
        &[vault_seeds],
      ),
      amount,
    )?;
    
    Ok(())
  }
}